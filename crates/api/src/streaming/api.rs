use super::threads;
use crate::state::AppState;
use axum::Router;
use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

pub async fn create_api() -> Router {
    let app_state = AppState::new().await;

    let cors = CorsLayer::new()
        .allow_headers([axum::http::header::CONTENT_TYPE])
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    Router::new()
        .merge(threads::threads_routes())
        .layer(cors)
        .with_state(app_state)
}
