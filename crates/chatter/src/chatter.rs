use std::{env, sync::Arc};

use crate::{
    chatter_context::ChatterContext,
    chatter_message::{self, ChatterMessage},
    error::Result,
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
use tokio_postgres::NoTls;

pub struct QueryResultRow {
    pub geom: Geometry,
    pub properties: serde_json::Value,
}

#[derive(Clone)]
pub struct Chatter {
    pub context: ChatterContext,
    pub client: async_openai::Client<async_openai::config::OpenAIConfig>,
    pub pg_client: Arc<tokio_postgres::Client>,

    func_ctx: ExecutionContext,
}

impl Chatter {
    pub async fn new() -> Result<Self> {
        let config = env::var("POSTGRES_CONN_STR")?;
        let (client, connection) = tokio_postgres::connect(&config, NoTls).await?;
        let client = Arc::new(client);

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                panic!("Postgres connection error: {}", e);
            }
        });

        let func_ctx = ExecutionContextBuilder::default()
            .client(client.clone())
            .build()?;

        Ok(Self {
            context: ChatterContext::new(&client).await?,
            client: async_openai::Client::new(),
            pg_client: client,
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

    #[deprecated]
    pub async fn execute(&mut self) -> Result<ChatCompletionResponseMessage> {
        loop {
            let message = self.create_and_send_request().await?;

            // Add the AI response to the context
            self.context.add_message(message.clone().try_into()?);

            if let Some(tool_calls) = message.tool_calls {
                // Execute the tool call and get the response message
                let tool_response = self.execute_tool_call(tool_calls[0].clone()).await?;

                // Add the tool response to the context
                self.context.add_message(tool_response);

                // Continue the loop to process the next message
            } else {
                // No tool call, we're done
                return Ok(message);
            }
        }
    }

    pub fn execute_stream(mut self) -> impl Stream<Item = Result<ChatterMessage>> {
        try_stream! {
            loop {
                let message = self.create_and_send_request().await?;

                // Add the AI response to the context
                let cmessage: ChatterMessage = message.clone().try_into()?;
                self.context.add_message(cmessage.clone());
                yield cmessage;

                if let Some(tool_calls) = message.tool_calls {
                    // Execute the tool call and get the response message
                    let tool_response = self.execute_tool_call(tool_calls[0].clone()).await?;

                    // Add the tool response to the context
                    self.context.add_message(tool_response.clone());
                    yield tool_response;
                } else {
                    // No tool call, we're done
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
            .temperature(0.2)
            .model(&self.context.model)
            .messages(
                self.context
                    .messages
                    .iter()
                    .map(|m| m.clone().try_into())
                    .collect::<Result<Vec<ChatCompletionRequestMessage>>>()?,
            )
            .tools(self.context.tools.clone())
            .parallel_tool_calls(false) // We only want to run one tool at a time
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
    ) -> Result<chatter_message::ChatterMessage> {
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
        // we wrap the query so we get the geometry and attributes in the correct formats
        let internal_query = format!(
            r#"
            SELECT
                "bbh_internal_query"."geom",
                to_jsonb("bbh_internal_query") - 'geom' AS "properties"
            FROM (
                {}
            ) AS "bbh_internal_query"
        "#,
            query
        );
        let rows = self
            .pg_client
            .query(&internal_query, &[])
            .await?
            .iter()
            .map(|row| {
                let geom: Geometry = row.get::<_, GeometryWrapper>("geom").0;
                let properties: serde_json::Value = row.get("properties");
                QueryResultRow { geom, properties }
            })
            .collect();
        Ok(rows)
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
