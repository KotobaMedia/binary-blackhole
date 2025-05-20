use crate::data::threads::MessageView;
use crate::error::AppError;
use crate::state::AppState;
use anyhow::Context;
use axum::body::{Body, Bytes};
use axum::http::header;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use chatter::chatter::Chatter;
use chatter::chatter_context::ChatterContext;
use chatter::chatter_message::Role;
use chatter::data::types::chat_message::{ChatMessage, ChatMessageBuilder};
use chatter::data::types::chat_thread::ChatThread;
use futures::{StreamExt, future};
use serde::Deserialize;
use serde_json::json;
use std::convert::Infallible;
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

    let thread = ChatThread::get_thread(&state.ddb, "demo_user", &thread_id.to_string()).await?;
    if thread.archived.unwrap_or(false) == true {
        return Err(AppError::Conflict("thread_archived".to_string()));
    }

    // Get the messages for the thread so we can re-instantiate the context
    let messages =
        ChatMessage::get_all_thread_messages(&state.ddb, "demo_user", &thread_id.to_string())
            .await?;
    let thread_message_count = messages.len() as u32;

    let stream = {
        let pg = state.postgres_pool.get().await?;
        let mut chatter = Chatter::new(pg).await?;
        if !messages.is_empty() {
            let ctx = ChatterContext::new_with_stored(
                thread_id.to_string(),
                messages.into_iter().map(|m| m.msg).collect(),
            );
            chatter.switch_context(ctx).await?;
        } else {
            chatter.new_context().await?;
        }
        chatter.add_user_message(&payload.content)?;
        // let messages = &chatter.context.messages;
        // let msg = messages.last().unwrap();
        // let mut binding = ChatMessageBuilder::default();
        // binding
        //     .thread_message_ids(thread_id.to_string(), messages.len() as u32 - 1)
        //     .user_id("demo_user".to_string())
        //     .msg(msg.clone());
        // let message = binding.build()?;
        // state.ddb.put_item_excl(&message).await?;

        chatter.execute_stream()
    };

    let db = state.ddb.clone();
    let stream = stream
        .enumerate()
        .map(move |(i, m)| {
            let message = m?;
            let mut binding = ChatMessageBuilder::default();
            let builder = binding
                .thread_message_ids(thread_id.to_string(), thread_message_count + i as u32)
                .user_id("demo_user".to_string())
                .msg(message.clone());
            let db_message = builder.build()?;
            Ok::<_, AppError>(db_message)
        })
        .then(move |m| {
            let value = db.clone();
            async move {
                let db_message = m?;
                value.put_item_excl(&db_message).await?;
                Ok::<_, AppError>(db_message)
            }
        })
        .filter(|m| {
            // Filter out system messages.
            let resp = if let Ok(db_message) = m {
                db_message.msg.role != Role::System
            } else {
                // Keep errors in the stream.
                true
            };
            future::ready(resp)
        })
        .map(|m| {
            let message = m?;

            let message_view: MessageView = message.into();
            let message_json = serde_json::to_string(&message_view)?;
            println!("sending message to user: {}", &message_json);
            Ok::<_, AppError>(Bytes::from(message_json + "\n"))
        })
        .map(|res| {
            Ok::<_, Infallible>(res.unwrap_or_else(|err| {
                let value = json!({
                    "error": err.to_string(),
                });
                let error_json = serde_json::to_string(&value).unwrap();
                Bytes::from(error_json + "\n")
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
