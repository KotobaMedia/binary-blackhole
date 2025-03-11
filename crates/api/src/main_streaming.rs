use lambda_http::tracing;

mod data;
mod error;
mod state;
mod streaming;

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let app = streaming::api::create_api().await;
    lambda_http::run_with_streaming_response(app).await
}
