use axum::Router;
use axum::http::Method;
use axum::response::Redirect;
use axum::routing::get;
use lambda_http::{Error, run, tracing};
use state::AppState;
use tower_http::cors::{Any, CorsLayer};

mod error;
mod state;
mod threads;

async fn root() -> Redirect {
    Redirect::temporary("https://www.bblackhole.com/")
}
async fn health() -> String {
    "OK".to_string()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let app_state = AppState::new().await;

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/__health", get(health))
        .merge(threads::threads_routes())
        .layer(cors)
        .with_state(app_state);

    run(app).await
}
