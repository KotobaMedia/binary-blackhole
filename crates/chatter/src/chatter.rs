use crate::{
    chatter_context::ChatterContext,
    chatter_message::ChatterMessage,
    data::types::sql_query::SqlQuery,
    error::{ChatterError, Result},
    functions::{ExecutionContext, ExecutionContextBuilder},
    geom::GeometryWrapper,
    pg_helpers::convert_column_value,
};
use async_openai::types::{
    ChatCompletionMessageToolCall, ChatCompletionRequestMessage, ChatCompletionResponseMessage,
    CreateChatCompletionRequestArgs,
};
use async_stream::try_stream;
use futures::Stream;
use geo_types::Geometry;
use std::sync::{Arc, Mutex};

pub struct QueryResultRow {
    pub geom: Geometry,
    pub properties: serde_json::Value,
}

#[derive(Clone)]
pub struct Chatter {
    pub context: Arc<Mutex<ChatterContext>>,
    pub client: async_openai::Client<async_openai::config::OpenAIConfig>,
    pub ddb_client: Arc<crate::data::dynamodb::Db>,
    pub pg_client: Arc<deadpool_postgres::Client>,

    func_ctx: ExecutionContext,
}

impl Chatter {
    pub async fn new(pg_client: deadpool_postgres::Client) -> Result<Self> {
        let pg_client = Arc::new(pg_client);
        let ddb_client = Arc::new(crate::data::dynamodb::Db::new().await);
        let context = Arc::new(Mutex::new(ChatterContext::new()));

        let func_ctx = ExecutionContextBuilder::default()
            .pg(pg_client.clone())
            .ddb(ddb_client.clone())
            .chatter_context(context.clone())
            .build()?;

        Ok(Self {
            context,
            client: async_openai::Client::new(),
            pg_client,
            ddb_client,
            func_ctx,
        })
    }

    /// Create a new context with default parameters. The Chatter's internal context
    /// will be replaced with the new context.
    pub async fn new_context(&mut self) -> Result<()> {
        let ctx = ChatterContext::new();
        self.switch_context(ctx).await
    }

    /// Switch the internal context with an already instantiated ChatterContext.
    /// This is used when a user returns to a previous conversation.
    /// Note that these messages should not include the system message -- it will be added in this function.
    pub async fn switch_context(&mut self, mut context: ChatterContext) -> Result<()> {
        // because the context doesn't have the system message, we will add it here.
        let system_message = ChatterMessage::create_system_message(&self.pg_client).await?;
        context.messages.insert(0, system_message);

        // Set the new context
        self.context = Arc::new(Mutex::new(context));
        // Update the function context with the new context
        self.func_ctx.update_context(self.context.clone());

        Ok(())
    }

    pub fn add_user_message(&mut self, message: &str) -> Result<()> {
        let mut context = self.context.lock().unwrap();
        context.add_user_message(message);
        Ok(())
    }

    pub fn execute_stream(mut self) -> impl Stream<Item = Result<ChatterMessage>> {
        let stream = try_stream! {
            let last_message = {
                let context = self.context.lock().unwrap();
                context.messages.last().cloned()
            };
            if let Some(last_message) = last_message {
                yield last_message;
            }

            loop {
                let message = self.create_and_send_request().await?;

                // Add the AI response to the context
                let cmessage: ChatterMessage = message.clone().try_into()?;
                {
                    let mut context = self.context.lock().unwrap();
                    context.add_message(cmessage.clone());
                };
                yield cmessage;

                if let Some(tool_calls) = message.tool_calls {
                    // Iterate over all tool calls and process each one
                    for tool_call in tool_calls {
                        let tool_response = self.execute_tool_call(tool_call).await?;
                        {
                            let mut context = self.context.lock().unwrap();
                            context.add_message(tool_response.clone());
                        };
                        yield tool_response;
                    }
                    // Continue the loop to process the next message
                } else {
                    // No tool call, that means that the assistant has finished.
                    break;
                }
            }
        };
        stream
    }

    /// Creates and sends a chat completion request, then returns the message from the response.
    async fn create_and_send_request(&mut self) -> Result<ChatCompletionResponseMessage> {
        let request = {
            let context = self.context.lock().unwrap();
            // Create the chat completion request
            CreateChatCompletionRequestArgs::default()
                .max_completion_tokens(2048u32)
                .model(&context.model)
                .messages(
                    context
                        .messages
                        .iter()
                        .map(|m| m.clone().try_into())
                        .collect::<Result<Vec<ChatCompletionRequestMessage>>>()?,
                )
                .tools(context.tools.clone())
                // The following two options are supported by gpt-4o, but not o3-mini
                // .temperature(0.2)
                // .parallel_tool_calls(false) // We only want to run one tool at a time
                .build()
        }?;
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
                Ok(response)
            }
            "query_database" => {
                let args = serde_json::from_str(&call.arguments)?;
                let response = self.func_ctx.query_database(&id, args).await?;
                Ok(response)
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
    pub async fn execute_raw_query(&mut self, query: &str) -> Result<Vec<QueryResultRow>> {
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

    pub async fn get_query_results(&mut self, query_id: &str) -> Result<Vec<QueryResultRow>> {
        let query_obj = SqlQuery::get_query(&self.ddb_client, query_id)
            .await
            .map_err(|e| ChatterError::QueryError(e.to_string()))?;
        let query_str = query_obj.query_content;
        self.execute_raw_query(&query_str).await
    }

    /// Execute a SQL query for a given XYZ tile and return the result as a MVT binary.
    /// Note: the query's geometry column must be named "geom" and the ID column must be named "ogc_fid".
    pub async fn get_tile(&mut self, query_id: &str, z: i32, x: i32, y: i32) -> Result<Vec<u8>> {
        let query_obj = SqlQuery::get_query(&self.ddb_client, query_id)
            .await
            .map_err(|e| ChatterError::QueryError(e.to_string()))?;
        let query_str = query_obj.query_content;
        let stmt = self.pg_client.prepare(&query_str).await?;
        let columns = stmt.columns();
        // get the first column from the query -- that will be our ID column
        let id_column = columns
            .iter()
            .find(|col| col.name() == "_id")
            .ok_or_else(|| ChatterError::QueryError("No ID column found".to_string()))?;
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

        // Generate a comma-separated list of columns from source, excluding id and geom columns.
        let extra_columns: Vec<String> = columns
            .iter()
            .filter(|col| col.name() != id_column_name && col.name() != geom_column_name)
            .map(|col| format!("source.\"{}\"", col.name()))
            .collect();
        let extra_columns_str = if extra_columns.is_empty() {
            "".to_string()
        } else {
            format!(", {}", extra_columns.join(", "))
        };

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
                    {query_str}
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
                    {extra_columns_str}
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

    pub async fn get_query_bbox(&mut self, query_id: &str) -> Result<[f64; 4]> {
        let query_obj = SqlQuery::get_query(&self.ddb_client, query_id)
            .await
            .map_err(|e| ChatterError::QueryError(e.to_string()))?;
        let query_str = query_obj.query_content;
        let extent_query = format!(
            r#"
                WITH
                source AS (
                    {query_str}
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
        let result = self.pg_client.query_one(&extent_query, &[]).await?;
        let minx: f64 = result.get(0);
        let miny: f64 = result.get(1);
        let maxx: f64 = result.get(2);
        let maxy: f64 = result.get(3);

        Ok([minx, miny, maxx, maxy])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_postgres::{Config, ManagerConfig, PoolConfig, RecyclingMethod, Runtime};
    use geo_types::Point;
    use serde_json::Value;
    use std::env;
    use tokio_postgres::NoTls;

    async fn setup() -> Result<Chatter> {
        let mut cfg = Config::new();
        let config = env::var("POSTGRES_CONN_STR")?;
        cfg.url = Some(config);
        cfg.pool = Some(PoolConfig {
            max_size: 1,
            ..Default::default()
        });
        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
        let pg_client = pool.get().await?;
        Chatter::new(pg_client).await
    }

    #[tokio::test]
    async fn test_chatter() -> Result<()> {
        setup().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_query() -> Result<()> {
        let mut chatter = setup().await?;
        // some data we just create for the test
        let rows = chatter
            .execute_raw_query(
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
