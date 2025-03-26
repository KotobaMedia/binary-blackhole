use crate::{chatter_message::ChatterMessageSidecar, error::Result};
use async_openai::types::{ChatCompletionTool, Role};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{chatter_message::ChatterMessage, functions::ExecutionContext};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatterContext {
    /// A unique identifier for this context. Use this to track context and conversation history.
    pub id: String,
    pub messages: Vec<ChatterMessage>,
    pub model: String,
    pub tools: Vec<ChatCompletionTool>,
}

impl ChatterContext {
    async fn create_system_message(client: &tokio_postgres::Client) -> Result<ChatterMessage> {
        let mut tables: String = String::new();
        let rows = client
            .query(
                r#"
                    SELECT
                        "table_name",
                        "metadata"->>'name' AS "name"
                    FROM "datasets";
                "#,
                &[],
            )
            .await?;
        for row in rows {
            let table_name: String = row.get(0);
            let name: String = row.get(1);
            tables.push_str(&format!("- `{}`: {}\n", table_name, name));
        }

        Ok(ChatterMessage {
            message: Some(
                format!(include_str!("../data/system_prompt_01.txt"), tables).to_string(),
            ),
            role: Role::System,
            tool_calls: None,
            tool_call_id: None,
            sidecar: ChatterMessageSidecar::None,
        })
    }

    /// Create a new context with default parameters.
    pub async fn new(client: &tokio_postgres::Client) -> Result<Self> {
        Ok(Self {
            id: Ulid::new().to_string(),
            messages: vec![Self::create_system_message(client).await?],
            model: "gpt-4o".to_string(),
            tools: vec![
                ExecutionContext::describe_tables_tool(),
                ExecutionContext::query_database_tool(),
            ],
        })
    }

    /// Instantiate a new context with stored messages.
    /// This is used when a user returns to a previous conversation.
    /// Note that the system message isn't included, so it is recreated and added.
    pub async fn new_with_stored(
        client: &tokio_postgres::Client,
        id: String,
        mut messages: Vec<ChatterMessage>,
    ) -> Result<Self> {
        let system_msg = Self::create_system_message(client).await?;
        messages.insert(0, system_msg);
        Ok(Self {
            id,
            messages,
            model: "gpt-4o".to_string(),
            tools: vec![
                ExecutionContext::describe_tables_tool(),
                ExecutionContext::query_database_tool(),
            ],
        })
    }

    pub fn add_message(&mut self, message: ChatterMessage) {
        self.messages.push(message);
    }

    pub fn add_user_message(&mut self, message: &str) {
        self.add_message(ChatterMessage {
            message: Some(message.to_string()),
            tool_calls: None,
            role: Role::User,
            tool_call_id: None,
            sidecar: ChatterMessageSidecar::None,
        });
    }
}
