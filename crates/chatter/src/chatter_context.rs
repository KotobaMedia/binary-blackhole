use crate::chatter_message::ChatterMessageSidecar;
use async_openai::types::{ChatCompletionTool, Role};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{chatter_message::ChatterMessage, functions::ExecutionContext};

/// A context is a collection of messages and tools that are used to interact with the LLM.
/// It is used to track the conversation history and the tools that are available to the LLM.
/// The context is created when a user starts a new conversation.
///
/// TODO: Refactor this so instead of taking in a client, the system message is passed in as a parameter, decoupling it from this struct.
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
    /// Use this when a user starts a new conversation.
    pub fn new() -> Self {
        Self::new_with_stored(Ulid::new().to_string(), vec![])
    }

    /// Instantiate a new context with stored messages.
    /// This is used when a user returns to a previous conversation.
    /// Note that these messages should not include the system message.
    pub fn new_with_stored(id: String, messages: Vec<ChatterMessage>) -> Self {
        Self {
            id,
            messages,
            // model: "gpt-4o".to_string(),
            model: "gpt-4.1".to_string(),
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
