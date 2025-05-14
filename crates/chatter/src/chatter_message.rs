use crate::error::{ChatterError, Result};
use async_openai::types::{
    ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionResponseMessage, Role as OpenAIRole,
};
use serde::{Deserialize, Serialize};

pub type Role = OpenAIRole;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SQLExecutionDetails {
    pub id: String,
    pub name: String,
    pub sql: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum ChatterMessageSidecar {
    #[default]
    None,

    /// Execute some SQL. (Query ID, name, SQL query)
    SQLExecution(SQLExecutionDetails),
    /// A failed SQL execution.
    SQLExecutionError,

    /// A database lookup.
    DatabaseLookup,
}

impl ChatterMessageSidecar {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatterMessage {
    pub message: Option<String>,
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub tool_calls: Option<Vec<ChatCompletionMessageToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "ChatterMessageSidecar::is_none")]
    #[serde(default)]
    pub sidecar: ChatterMessageSidecar,
}

impl TryFrom<ChatCompletionResponseMessage> for ChatterMessage {
    type Error = ChatterError;

    fn try_from(message: ChatCompletionResponseMessage) -> Result<Self> {
        Ok(Self {
            message: message.content,
            role: message.role,
            tool_calls: message.tool_calls,
            tool_call_id: None,
            sidecar: ChatterMessageSidecar::None,
        })
    }
}

impl TryFrom<ChatterMessage> for ChatCompletionRequestMessage {
    type Error = ChatterError;

    fn try_from(message: ChatterMessage) -> Result<Self> {
        let out: ChatCompletionRequestMessage = match message.role {
            Role::User => {
                let mut msg = ChatCompletionRequestUserMessageArgs::default();
                let msg = if let Some(message) = message.message {
                    msg.content(message)
                } else {
                    &mut msg
                };
                msg.build()?.into()
            }
            Role::System => {
                let mut msg = ChatCompletionRequestSystemMessageArgs::default();
                let msg = if let Some(message) = message.message {
                    msg.content(message)
                } else {
                    &mut msg
                };
                msg.build()?.into()
            }
            Role::Assistant => {
                let mut msg = ChatCompletionRequestAssistantMessageArgs::default();
                let mut msg = if let Some(message) = message.message {
                    msg.content(message)
                } else {
                    &mut msg
                };
                let msg = if let Some(tool_calls) = message.tool_calls {
                    msg.tool_calls(tool_calls)
                } else {
                    &mut msg
                };
                msg.build()?.into()
            }
            Role::Tool => {
                let mut msg = ChatCompletionRequestToolMessageArgs::default();
                let mut msg = if let Some(message) = message.message {
                    msg.content(message)
                } else {
                    &mut msg
                };
                let msg = if let Some(tool_call_id) = message.tool_call_id {
                    msg.tool_call_id(tool_call_id)
                } else {
                    &mut msg
                };
                msg.build()?.into()
            }
            role => Err(ChatterError::UnknownRole(role.to_string()))?,
        };
        Ok(out)
    }
}
