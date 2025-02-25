use std::{env, sync::Arc};

use crate::{
    chatter_context::ChatterContext,
    error::Result,
    functions::{ExecutionContext, ExecutionContextBuilder},
};
use async_openai::types::{
    ChatCompletionMessageToolCall, ChatCompletionRequestMessage, ChatCompletionResponseMessage,
    CreateChatCompletionRequestArgs,
};
use tokio_postgres::NoTls;

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

    pub fn switch_context(&mut self, context: ChatterContext) {
        self.context = context;
    }

    pub async fn execute(&mut self) -> Result<ChatCompletionResponseMessage> {
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
            .parallel_tool_calls(false) // We only want to run one tool at a time
            .build()?;
        // println!("Sending request: {}", serde_json::to_string(&request)?);
        let response = self.client.chat().create(request).await?;
        let choice = response.choices[0].clone();
        self.context.add_message(choice.message.clone().try_into()?);
        let message = choice.message;
        // If the message is a tool call, we need to execute the tool and re-run the model.
        // Because we have parallel set as false, we know there is only one tool call.
        if let Some(tool_calls) = message.tool_calls {
            self.execute_tool_call(tool_calls[0].clone()).await?;
            Box::pin(self.execute()).await
        } else {
            Ok(message)
        }
    }

    async fn execute_tool_call(&mut self, tool_call: ChatCompletionMessageToolCall) -> Result<()> {
        let call = tool_call.function;
        let id = tool_call.id;
        let response = match call.name.as_str() {
            "describe_tables" => {
                let args = serde_json::from_str(&call.arguments)?;
                let response = self.func_ctx.describe_tables(&id, args).await?;
                response.into()
            }
            "query_database" => {
                let args = serde_json::from_str(&call.arguments)?;
                let response = self.func_ctx.query_database(&id, args).await?;
                response.into()
            }
            other => {
                return Err(crate::error::ChatterError::UnknownToolCall(
                    other.to_string(),
                ));
            }
        };
        self.context.add_message(response);
        Ok(())
    }
}
