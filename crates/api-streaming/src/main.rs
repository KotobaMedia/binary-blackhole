use axum::{
    Router,
    http::Method,
    response::{
        IntoResponse,
        sse::{Event, Sse},
    },
    routing::get,
};
use futures::stream::{self, Stream};
use lambda_http::tracing;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt as _;
use tower_http::cors::{Any, CorsLayer};

async fn regular_handler() -> impl IntoResponse {
    "Buffered hello world!"
}

async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::repeat_with(|| Event::default().data("Streamed hello world!"))
        .map(Ok)
        .throttle(Duration::from_secs(1))
        .take(5);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let cors = CorsLayer::new()
        .allow_headers([axum::http::header::CONTENT_TYPE])
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let app = Router::new()
        .route("/a", get(regular_handler))
        .route("/b", get(sse_handler))
        .layer(cors);

    lambda_http::run_with_streaming_response(app).await
}
