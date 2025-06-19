use lambda_http::tracing;

mod data;
mod error;
mod sentry;
mod state;
mod streaming;

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    // Initialize Sentry if SENTRY_DSN is set
    let _sentry_guard = sentry::init_sentry_guard();

    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let app = streaming::api::create_api().await;
    lambda_http::run_with_streaming_response(app).await
}
