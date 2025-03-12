use chatter::chatter_message::{ChatterMessage, ChatterMessageSidecar, Role};
use data::types::chat_message::ChatMessage;
use data::types::chat_thread::ChatThread;
use serde::Serialize;

#[derive(Serialize)]
pub struct Thread {
    pub id: String,
    pub title: String,
}
impl From<ChatThread> for Thread {
    fn from(thread: ChatThread) -> Self {
        Self {
            id: thread.id().to_string(),
            title: thread.title,
        }
    }
}

#[derive(Serialize)]
pub struct ThreadList {
    pub threads: Vec<Thread>,
}

#[derive(Serialize)]
pub struct ChatterMessageView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub message: Option<String>,

    pub role: Role,

    #[serde(skip_serializing_if = "ChatterMessageSidecar::is_none")]
    #[serde(default)]
    pub sidecar: ChatterMessageSidecar,
}

impl From<ChatterMessage> for ChatterMessageView {
    fn from(message: ChatterMessage) -> Self {
        let mut msg = message.message;
        if message.role == Role::Tool {
            // The message from the tool is directed at the LLM, not the user.
            // The contents we want to show to the user are in the sidecar.
            msg = None;
        }
        Self {
            message: msg,
            role: message.role,
            sidecar: message.sidecar.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct MessageView {
    pub id: u32,
    pub content: ChatterMessageView,
}
impl From<ChatMessage> for MessageView {
    fn from(message: ChatMessage) -> Self {
        Self {
            id: message.id(),
            content: message.msg.into(),
        }
    }
}

#[derive(Serialize)]
pub struct ThreadDetails {
    pub id: String,
    pub title: String,
    pub archived: Option<bool>,
    pub messages: Vec<MessageView>,
}
