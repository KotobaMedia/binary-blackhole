use std::convert::Infallible;

use crate::error::{AppError, Result};
use crate::state::AppState;
use anyhow::Context;
use axum::Error;
use axum::body::{Body, Bytes};
use axum::http::header;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chatter::chatter_context::ChatterContext;
use chatter::chatter_message::{ChatterMessage, ChatterMessageSidecar, Role};
use chrono::Utc;
use data::types::chat_message::{ChatMessage, ChatMessageBuilder};
use data::types::chat_thread::{ChatThread, ChatThreadBuilder};
use futures::StreamExt;
use futures::stream::TryStreamExt;
use serde::Deserialize;
use serde_json::json;
use ulid::Ulid;

#[derive(Deserialize)]
struct CreateThreadMessageRequest {
    content: String,
}

async fn create_thread_message_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateThreadMessageRequest>,
) -> impl IntoResponse {
    let thread_id = Ulid::from_string(&id).context("Invalid thread ID")?;

    let thread = ChatThread::get_thread(&state.db, "demo_user", &thread_id.to_string()).await?;
    if thread.archived.unwrap_or(false) == true {
        return Err(AppError::Conflict("thread_archived".to_string()));
    }

    // Get the messages for the thread so we can re-instantiate the context
    let messages =
        ChatMessage::get_all_thread_messages(&state.db, "demo_user", &thread_id.to_string())
            .await?;

    let stream = {
        let mut chatter = state.chatter.lock().await.clone();
        if !messages.is_empty() {
            let ctx = ChatterContext::new_with_stored(
                thread_id.to_string(),
                messages.into_iter().map(|m| m.msg).collect(),
            );
            chatter.switch_context(ctx);
        } else {
            chatter.new_context().await?;
        }
        chatter.context.add_user_message(&payload.content);

        chatter.execute_stream()
    };

    let db = state.db.clone();
    let stream = stream
        .enumerate()
        .map(move |(i, m)| {
            let message = m?;
            let mut binding = ChatMessageBuilder::default();
            let builder = binding
                .thread_message_ids(thread_id.to_string(), i as u32)
                .user_id("demo_user".to_string())
                .msg(message.clone());
            let message = builder.build()?;
            Ok::<_, AppError>(message)
        })
        .then(move |m| {
            let value = db.clone();
            async move {
                let message = m?;
                value.put_item_excl(&message).await?;
                Ok::<_, AppError>(message)
            }
        })
        .map(|m| {
            let message = m?;
            let message_json = serde_json::to_string(&message)?;
            Ok::<_, AppError>(Bytes::from(message_json))
        })
        .take_while(|res| std::future::ready(res.is_ok()))
        .map(|res| {
            Ok::<_, Infallible>(res.unwrap_or_else(|err| {
                let value = json!({
                    "error": err.to_string(),
                });
                let error_json = serde_json::to_string(&value).unwrap();
                Bytes::from(error_json)
            }))
        });

    let body = Body::from_stream(stream);

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/x-ndjson")],
        body,
    )
        .into_response())
}

pub fn threads_routes() -> Router<AppState> {
    Router::new()
        // this route is is the streaming API
        .route("/threads/{id}/message", post(create_thread_message_handler))
}
