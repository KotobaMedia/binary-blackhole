use crate::data::threads::ThreadDetailsFull;
use crate::state::AppState;
use crate::{
    data::threads::{MessageView, Thread, ThreadDetails, ThreadList},
    error::Result,
};
use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chatter::data::types::chat_message::ChatMessage;
use chatter::data::types::chat_thread::{ChatThread, ChatThreadBuilder};
use chrono::Utc;
use serde::Serialize;
use ulid::Ulid;

async fn get_threads_handler(State(state): State<AppState>) -> Result<Json<ThreadList>> {
    let threads = ChatThread::get_all_user_threads(&state.ddb, "demo_user").await?;
    let threads: Vec<Thread> = threads.into_iter().map(Into::into).collect();
    Ok(ThreadList { threads }.into())
}

async fn get_thread_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ThreadDetails>> {
    let thread_f = ChatThread::get_thread(&state.ddb, "demo_user", &id);
    let messages_f = ChatMessage::get_all_thread_messages(&state.ddb, "demo_user", &id);
    let (thread, messages) = tokio::try_join!(thread_f, messages_f)?;

    Ok(ThreadDetails {
        id: thread.id().to_string(),
        title: thread.title,
        archived: thread.archived,
        messages: messages
            .into_iter()
            .map(Into::into)
            .filter(|m: &MessageView| {
                m.content.role != chatter::chatter_message::Role::System
                // && (!m.content.sidecar.is_none() || !m.content.message.is_none())
            })
            .collect(),
    }
    .into())
}

async fn get_thread_full_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ThreadDetailsFull>> {
    let thread_f = ChatThread::get_thread(&state.ddb, "demo_user", &id);
    let messages_f = ChatMessage::get_all_thread_messages(&state.ddb, "demo_user", &id);
    let (thread, messages) = tokio::try_join!(thread_f, messages_f)?;

    Ok(ThreadDetailsFull {
        id: thread.id().to_string(),
        title: thread.title,
        archived: thread.archived,
        messages: messages.into_iter().map(|m| m.msg).collect(),
    }
    .into())
}

#[derive(Serialize)]
struct CreateNewThreadResponse {
    thread_id: String,
}

async fn create_new_thread_handler(State(state): State<AppState>) -> Result<Response> {
    let thread_id = Ulid::new();
    let thread = ChatThreadBuilder::default()
        .id(thread_id.to_string())
        .user_id("demo_user".to_string())
        .title(thread_id.to_string())
        .modified_ts(Utc::now())
        .build()?;
    state.ddb.put_item_excl(&thread).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateNewThreadResponse {
            thread_id: thread_id.to_string(),
        }),
    )
        .into_response())
}

async fn archive_thread_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response> {
    let thread_id = Ulid::from_string(&id).context("Invalid thread ID")?;
    let mut thread =
        ChatThread::get_thread(&state.ddb, "demo_user", &thread_id.to_string()).await?;
    let ts = thread.modified_ts;
    thread.archived = Some(true);
    thread.modified_ts = Utc::now();

    state.ddb.put_item_lock(&thread, "modified_ts", &ts).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

pub fn threads_routes() -> Router<AppState> {
    Router::new()
        .route("/threads", get(get_threads_handler))
        .route("/threads", post(create_new_thread_handler))
        .route("/threads/{id}", get(get_thread_handler))
        .route("/threads/{id}/_full", get(get_thread_full_handler))
        .route("/threads/{id}/archive", post(archive_thread_handler))
}
