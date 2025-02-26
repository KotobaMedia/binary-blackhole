use axum::Router;
use axum::http::Method;
use axum::response::Redirect;
use axum::routing::get;
use lambda_http::{Error, run, tracing};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize, Serialize)]
struct Params {
    first: Option<String>,
    second: Option<String>,
}

async fn root() -> Redirect {
    Redirect::temporary("https://bblackhole.com/")
}
async fn health() -> String {
    "OK".to_string()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/__health", get(health))
        .layer(cors);

    run(app).await
}
