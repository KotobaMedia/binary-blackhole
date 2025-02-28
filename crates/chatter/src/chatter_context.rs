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
    /// Create a new context with default parameters.
    pub async fn new(client: &tokio_postgres::Client) -> Result<Self> {
        let mut tables: String = String::new();
        let rows = client
            .query(
                r#"
                    SELECT
                        "table_name",
                        "metadata"->'data_item'->>'name' AS "name"
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

        Ok(Self {
            id: Ulid::new().to_string(),
            messages: vec![ChatterMessage {
                message: Some(
                    format!(include_str!("../data/system_prompt_01.txt"), tables).to_string(),
                ),
                role: Role::System,
                tool_calls: None,
                tool_call_id: None,
                sidecar: ChatterMessageSidecar::None,
            }],
            model: "gpt-4o".to_string(),
            tools: vec![
                ExecutionContext::describe_tables_tool(),
                ExecutionContext::query_database_tool(),
            ],
        })
    }

    /// Instantiate a new context with stored messages.
    /// This is used when a user returns to a previous conversation.
    pub fn new_with_stored(id: String, messages: Vec<ChatterMessage>) -> Self {
        Self {
            id,
            messages,
            model: "gpt-4o".to_string(),
            tools: vec![
                ExecutionContext::describe_tables_tool(),
                ExecutionContext::query_database_tool(),
            ],
        }
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
