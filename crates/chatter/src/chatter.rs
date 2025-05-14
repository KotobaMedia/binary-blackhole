use crate::{
    chatter_context::ChatterContext,
    chatter_message::ChatterMessage,
    error::{ChatterError, Result},
    functions::{ExecutionContext, ExecutionContextBuilder},
    geom::GeometryWrapper,
};
use async_openai::types::{
    ChatCompletionMessageToolCall, ChatCompletionRequestMessage, ChatCompletionResponseMessage,
    CreateChatCompletionRequestArgs,
};
use async_stream::try_stream;
use futures::Stream;
use geo_types::Geometry;
use rust_decimal::Decimal;
use std::{env, sync::Arc};
use tokio_postgres::NoTls;

pub struct QueryResultRow {
    pub geom: Geometry,
    pub properties: serde_json::Value,
}

#[derive(Clone)]
pub struct Chatter {
    pub context: ChatterContext,
    pub client: async_openai::Client<async_openai::config::OpenAIConfig>,
    pub ddb_client: Arc<crate::data::dynamodb::Db>,
    pub pg_client: Arc<tokio_postgres::Client>,

    func_ctx: ExecutionContext,
}

impl Chatter {
    pub async fn new() -> Result<Self> {
        let config = env::var("POSTGRES_CONN_STR")?;
        let (pg_client, connection) = tokio_postgres::connect(&config, NoTls).await?;
        let pg_client = Arc::new(pg_client);

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                panic!("Postgres connection error: {}", e);
            }
        });

        pg_client
            .batch_execute("SET statement_timeout = 10000")
            .await?;

        let ddb_client = Arc::new(crate::data::dynamodb::Db::new().await);

        let func_ctx = ExecutionContextBuilder::default()
            .pg(pg_client.clone())
            .ddb(ddb_client.clone())
            .build()?;

        Ok(Self {
            context: ChatterContext::new(&pg_client).await?,
            client: async_openai::Client::new(),
            pg_client,
            ddb_client,
            func_ctx,
        })
    }

    /// Create a new context with default parameters. The Chatter's internal context
    /// will be replaced with the new context.
    pub async fn new_context(&mut self) -> Result<()> {
        self.context = ChatterContext::new(&self.pg_client).await?;
        Ok(())
    }

    /// Switch the internal context with an already instantiated ChatterContext.
    pub fn switch_context(&mut self, context: ChatterContext) {
        self.context = context;
    }

    pub fn execute_stream(mut self) -> impl Stream<Item = Result<ChatterMessage>> {
        try_stream! {
            let last_message = self.context.messages.last().cloned();
            if let Some(last_message) = last_message {
                yield last_message;
            }

            loop {
                let message = self.create_and_send_request().await?;

                // Add the AI response to the context
                let cmessage: ChatterMessage = message.clone().try_into()?;
                self.context.add_message(cmessage.clone());
                yield cmessage;

                if let Some(tool_calls) = message.tool_calls {
                    // Iterate over all tool calls and process each one
                    for tool_call in tool_calls {
                        let tool_response = self.execute_tool_call(tool_call).await?;
                        self.context.add_message(tool_response.clone());
                        yield tool_response;
                    }
                    // Continue the loop to process the next message
                } else {
                    // No tool call, that means that the assistant has finished.
                    break;
                }
            }
        }
    }

    /// Creates and sends a chat completion request, then returns the message from the response.
    async fn create_and_send_request(&mut self) -> Result<ChatCompletionResponseMessage> {
        // Create the chat completion request
        let request = CreateChatCompletionRequestArgs::default()
            .max_completion_tokens(2048u32)
            .model(&self.context.model)
            .messages(
                self.context
                    .messages
                    .iter()
                    .map(|m| m.clone().try_into())
                    .collect::<Result<Vec<ChatCompletionRequestMessage>>>()?,
            )
            .tools(self.context.tools.clone())
            // The following two options are supported by gpt-4o, but not o3-mini
            // .temperature(0.2)
            // .parallel_tool_calls(false) // We only want to run one tool at a time
            .build()?;

        // Send the request and get the response
        let response = self.client.chat().create(request).await?;
        let choice = response.choices[0].clone();

        Ok(choice.message)
    }

    /// Executes a tool call and returns the response message
    async fn execute_tool_call(
        &mut self,
        tool_call: ChatCompletionMessageToolCall,
    ) -> Result<ChatterMessage> {
        let call = tool_call.function;
        let id = tool_call.id;
        match call.name.as_str() {
            "describe_tables" => {
                let args = serde_json::from_str(&call.arguments)?;
                let response = self.func_ctx.describe_tables(&id, args).await?;
                Ok(response.into())
            }
            "query_database" => {
                let args = serde_json::from_str(&call.arguments)?;
                let response = self.func_ctx.query_database(&id, args).await?;
                Ok(response.into())
            }
            other => Err(crate::error::ChatterError::UnknownToolCall(
                other.to_string(),
            )),
        }
    }

    /// Execute a SQL query and return the result. Used by the API to execute queries.
    /// TODO: This area requires a lot of refactoring -- the query_database tool
    /// should actually run the query, store the result somewhere, then return the ID of
    /// the execution, rendering this function obsolete. This is used in the meantime.
    pub async fn execute_query(&mut self, query: &str) -> Result<Vec<QueryResultRow>> {
        // Execute the provided query directly.
        let rows = self.pg_client.query(query, &[]).await?;
        let mut results = Vec::with_capacity(rows.len());

        for row in rows {
            let mut geom: Option<Geometry> = None;
            let mut properties = serde_json::Map::new();

            // Iterate over each column in the row.
            for (i, column) in row.columns().iter().enumerate() {
                // Check if this column is of the geometry type.
                // (Adjust the condition if your type detection is different.)
                if column.type_().name() == "geometry" {
                    // Convert using your GeometryWrapper.
                    // This assumes that GeometryWrapper implements FromSql.
                    geom = Some(row.get::<_, GeometryWrapper>(i).0);
                } else {
                    let value = convert_column_value(&row, i, column);
                    properties.insert(column.name().to_string(), value);
                }
            }

            // Return an error if no geometry column was found.
            let geom = geom.ok_or_else(|| ChatterError::GeometryNotFound)?;
            results.push(QueryResultRow {
                geom,
                properties: serde_json::Value::Object(properties),
            });
        }
        Ok(results)
    }

    /// Execute a SQL query for a given XYZ tile and return the result as a MVT binary.
    /// Note: the query's geometry column must be named "geom" and the ID column must be named "ogc_fid".
    pub async fn get_tile(&mut self, query: &str, z: i32, x: i32, y: i32) -> Result<Vec<u8>> {
        let stmt = self.pg_client.prepare(query).await?;
        let columns = stmt.columns();
        // get the first column from the query -- that will be our ID column
        let id_column = columns.get(0).ok_or_else(|| {
            ChatterError::QueryError("No columns found in the query result.".to_string())
        })?;
        let id_column_name = id_column.name();
        let geom_column = columns
            .iter()
            .find(|col| col.type_().name() == "geometry")
            .ok_or_else(|| {
                ChatterError::QueryError(
                    "No geometry column found in the query result.".to_string(),
                )
            })?;
        let geom_column_name = geom_column.name();

        let query = format!(
            r#"
                WITH
                -- 1) tile coords + both envelopes
                params AS (
                SELECT
                    $1::int   AS z,
                    $2::int   AS x,
                    $3::int   AS y,
                    -- WebMercator envelope for MVT‐packing
                    ST_TileEnvelope($1, $2, $3)                            AS env_3857,
                    -- tile envelope reprojected once into 4326 for indexed intersection
                    ST_Transform(
                    ST_TileEnvelope($1, $2, $3),
                    4326
                    )                                                       AS env_4326
                ),

                -- 2) your auto‐generated subquery goes here
                source AS (
                    {query}
                ),

                -- 3) only intersect in 4326 (uses index on source.geom), then reproject+clip
                tile_raw AS (
                SELECT
                    "{id_column_name}",
                    ST_AsMVTGeom(
                        ST_Transform("source"."{geom_column_name}", 3857),
                        params.env_3857,
                        4096,
                        256,
                        TRUE
                    ) AS geom
                FROM source
                CROSS JOIN params
                WHERE
                    source."{geom_column_name}" && params.env_4326
                )

                -- 4) pack into an MVT blob
                SELECT
                ST_AsMVT(
                    tile,
                    'data',
                    4096,
                    'geom',
                    '{id_column_name}'
                ) AS mvt_tile
                FROM (
                    SELECT * FROM tile_raw
                ) AS tile;
            "#,
        );

        let result = self.pg_client.query_one(&query, &[&z, &x, &y]).await?;
        let mvt_tile: Option<Vec<u8>> = result.get(0);

        mvt_tile.ok_or_else(|| {
            ChatterError::QueryError("No MVT tile found for the given query.".to_string())
        })
    }

    pub async fn get_query_bbox(&mut self, input_query: &str) -> Result<[f64; 4]> {
        let query = format!(
            r#"
                WITH
                source AS (
                    {input_query}
                )
                SELECT
                    ST_XMin(extent) AS minx,
                    ST_YMin(extent) AS miny,
                    ST_XMax(extent) AS maxx,
                    ST_YMax(extent) AS maxy
                FROM (
                    SELECT ST_Extent(source.geom) AS extent FROM source
                ) AS agg;
            "#,
        );
        let result = self.pg_client.query_one(&query, &[]).await?;
        let minx: f64 = result.get(0);
        let miny: f64 = result.get(1);
        let maxx: f64 = result.get(2);
        let maxy: f64 = result.get(3);

        Ok([minx, miny, maxx, maxy])
    }
}

// Helper function to convert a column value to serde_json::Value based on its type.
fn convert_column_value(
    row: &tokio_postgres::Row,
    index: usize,
    column: &tokio_postgres::Column,
) -> serde_json::Value {
    match column.type_().name() {
        // Convert text types to JSON string.
        "varchar" | "text" => {
            let s: Option<String> = row.get(index);
            s.map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null)
        }
        // Convert integer types.
        "int4" => {
            let v: Option<i32> = row.get(index);
            v.map(|v| serde_json::Value::Number(v.into()))
                .unwrap_or(serde_json::Value::Null)
        }
        "int8" => {
            let v: Option<i64> = row.get(index);
            v.map(|v| serde_json::Value::Number(v.into()))
                .unwrap_or(serde_json::Value::Null)
        }
        // Convert floating point types.
        "float4" => {
            let v: Option<f32> = row.get(index);
            v.and_then(|v| serde_json::Number::from_f64(v as f64))
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        "float8" => {
            let v: Option<f64> = row.get(index);
            v.and_then(serde_json::Number::from_f64)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        // Convert boolean types.
        "bool" => {
            let v: Option<bool> = row.get(index);
            v.map(serde_json::Value::Bool)
                .unwrap_or(serde_json::Value::Null)
        }
        "numeric" => {
            let v: Option<Decimal> = row.get(index);
            v.map(|v| serde_json::Value::String(v.to_string()))
                .unwrap_or(serde_json::Value::Null)
        }
        // If the column is already in JSON format.
        "json" | "jsonb" => {
            let v: Option<serde_json::Value> = row.get(index);
            v.unwrap_or(serde_json::Value::Null)
        }
        // Fallback: attempt to get a string representation.
        _col_type_name => {
            let s: Option<String> = row.try_get(index).ok().flatten();
            s.map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::Point;
    use serde_json::Value;

    #[tokio::test]
    async fn test_chatter() -> Result<()> {
        Chatter::new().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_query() -> Result<()> {
        let mut chatter = Chatter::new().await?;
        // some data we just create for the test
        let rows = chatter
            .execute_query(
                r#"
                SELECT
                    'hello' as "name",
                    ST_Point(35, 135, 4326) as "geom"
                "#,
            )
            .await?;
        assert!(!rows.is_empty());
        let row = &rows[0];
        assert_eq!(
            row.properties.get("name"),
            Some(&Value::String("hello".to_string()))
        );
        assert_eq!(row.geom, Point::new(35.0, 135.0).into());
        Ok(())
    }
}
