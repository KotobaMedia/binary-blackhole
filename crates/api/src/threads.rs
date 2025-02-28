use crate::error::{AppError, Result};
use crate::state::AppState;
use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chatter::chatter_message::{ChatterMessage, ChatterMessageSidecar, Role};
use data::dynamodb::{ChatMessage, ChatMessageBuilder, ChatThread, ChatThreadBuilder};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Serialize)]
struct Thread {
    id: String,
    title: String,
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
struct ThreadList {
    threads: Vec<Thread>,
}

async fn get_threads_handler(State(state): State<AppState>) -> Result<Json<ThreadList>> {
    let threads = ChatThread::get_all_user_threads(&state.db, "demo_user").await?;
    let threads: Vec<Thread> = threads.into_iter().map(Into::into).collect();
    Ok(ThreadList { threads }.into())
}

#[derive(Serialize)]
struct ChatterMessageView {
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
struct Message {
    id: u32,
    content: ChatterMessageView,
}
impl From<ChatMessage> for Message {
    fn from(message: ChatMessage) -> Self {
        Self {
            id: message.id(),
            content: message.msg.into(),
        }
    }
}

#[derive(Serialize)]
struct ThreadDetails {
    id: String,
    title: String,
    messages: Vec<Message>,
}

async fn get_thread_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ThreadDetails>> {
    let thread_f = ChatThread::get_thread(&state.db, "demo_user", &id);
    let messages_f = ChatMessage::get_all_thread_messages(&state.db, "demo_user", &id);
    let (thread, messages) = tokio::try_join!(thread_f, messages_f)?;

    Ok(ThreadDetails {
        id: thread.id().to_string(),
        title: thread.title,
        messages: messages
            .into_iter()
            .filter(|m| m.msg.role != chatter::chatter_message::Role::System)
            .map(Into::into)
            .collect(),
    }
    .into())
}

#[derive(Deserialize)]
struct CreateNewThreadRequest {
    content: String,
}

#[derive(Serialize)]
struct CreateNewThreadResponse {
    thread_id: String,
}

async fn create_new_thread_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateNewThreadRequest>,
) -> Result<Response> {
    let thread_id = Ulid::new();
    let thread = ChatThreadBuilder::default()
        .id(thread_id.to_string())
        .user_id("demo_user".to_string())
        .title(thread_id.to_string())
        .build()?;
    state.db.put_item_excl(thread).await?;

    // TODO: async
    let mut chatter = state.chatter.lock().await;
    chatter.new_context().await?;
    chatter.context.add_user_message(&payload.content);
    chatter.execute().await?;

    // translate the messages in to the database format
    let messages = chatter
        .context
        .messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let mut binding = ChatMessageBuilder::default();
            let builder = binding
                .thread_message_ids(thread_id.to_string(), i as u32)
                .user_id("demo_user".to_string())
                .msg(m.clone());
            builder.build().map_err(AppError::from)
        })
        .collect::<Result<Vec<ChatMessage>>>()?;
    for message in messages {
        state.db.put_item_excl(message).await?;
    }

    Ok((
        StatusCode::CREATED,
        Json(CreateNewThreadResponse {
            thread_id: thread_id.to_string(),
        }),
    )
        .into_response())
}

async fn create_thread_message_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateNewThreadRequest>,
) -> Result<Response> {
    let thread_id = Ulid::from_string(&id).context("Invalid thread ID")?;

    // Get the messages for the thread so we can re-instantiate the context
    let messages =
        ChatMessage::get_all_thread_messages(&state.db, "demo_user", &thread_id.to_string())
            .await?;
    let message_count = messages.len();

    let context = chatter::chatter_context::ChatterContext::new_with_stored(
        thread_id.to_string(),
        messages.into_iter().map(|m| m.msg).collect(),
    );

    // TODO: async
    let mut chatter = state.chatter.lock().await;
    chatter.switch_context(context);
    chatter.context.add_user_message(&payload.content);
    chatter.execute().await?;

    // translate the messages in to the database format
    let messages = chatter
        .context
        .messages
        .iter()
        .enumerate()
        .filter(|(i, _)| i >= &message_count)
        .map(|(i, m)| {
            let mut binding = ChatMessageBuilder::default();
            let builder = binding
                .thread_message_ids(thread_id.to_string(), i as u32)
                .user_id("demo_user".to_string())
                .msg(m.clone());
            builder.build().map_err(AppError::from)
        })
        .collect::<Result<Vec<ChatMessage>>>()?;
    for message in messages {
        state.db.put_item_excl(message).await?;
    }

    Ok((
        StatusCode::CREATED,
        Json(CreateNewThreadResponse {
            thread_id: thread_id.to_string(),
        }),
    )
        .into_response())
}

pub fn threads_routes() -> Router<AppState> {
    Router::new()
        .route("/threads", get(get_threads_handler))
        .route("/threads", post(create_new_thread_handler))
        .route("/threads/{id}", get(get_thread_handler))
        .route("/threads/{id}/message", post(create_thread_message_handler))
}
