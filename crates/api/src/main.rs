use lambda_http::{Error, run, tracing};

mod buffered;
mod data;
mod error;
mod sentry;
mod state;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    // Initialize Sentry if SENTRY_DSN is set
    let _sentry_guard = sentry::init_sentry_guard();

    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let api = buffered::api::create_api().await;
    run(api).await
}
