use lambda_http::{Error, run, tracing};

mod buffered;
mod data;
mod error;
mod state;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let api = buffered::api::create_api().await;
    run(api).await
}
