use axum::{
    Router,
    body::{Body, Bytes},
    http::{Method, StatusCode, header},
    response::{
        IntoResponse,
        sse::{Event, Sse},
    },
    routing::get,
};
use futures::stream::{self, Stream};
use lambda_http::tracing;
use std::{convert::Infallible, time::Duration};
use tokio::time::sleep;
use tokio_stream::StreamExt as _;
use tower_http::cors::{Any, CorsLayer};

mod data;
mod error;
mod state;
mod streaming;

// async fn ndjson_stream() -> impl IntoResponse {
//     // Hardcoded JSON objects for demo purposes
//     let json_lines = vec![
//         r#"{"id":"001","message":"Hello, NDJSON!","timestamp":1679680000}"#,
//         r#"{"id":"002","message":"Another line of data","timestamp":1679680005}"#,
//         r#"{"id":"003","message":"And one more line","timestamp":1679680010}"#,
//     ];

//     // Create a stream of JSON lines, each followed by a newline.
//     // Insert an artificial delay to simulate "streaming."
//     let stream = stream::iter(json_lines).then(|line| async move {
//         // Simulate some work or delayed generation
//         sleep(Duration::from_secs(1)).await;
//         // Convert each JSON string into bytes + newline
//         Ok::<_, Infallible>(Bytes::from(format!("{}\n", line)))
//     });

//     // Wrap our stream in a StreamBody, setting the Content-Type to NDJSON
//     let body = Body::from_stream(stream);

//     (
//         StatusCode::OK,
//         [(header::CONTENT_TYPE, "application/x-ndjson")],
//         body,
//     )
// }

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let app = streaming::api::create_api().await;
    lambda_http::run_with_streaming_response(app).await
}
