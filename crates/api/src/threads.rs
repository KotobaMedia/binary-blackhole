use axum::{
    Router,
    extract::State,
    routing::{get, post},
};

use crate::state::AppState;

async fn get_threads_handler(State(state): State<AppState>) -> String {
    "Get threads".to_string()
}

async fn get_thread_handler() -> String {
    "Get thread".to_string()
}

async fn create_post_handler() -> String {
    "Create post".to_string()
}

pub fn threads_routes() -> Router<AppState> {
    Router::new()
        .route("/threads", get(get_threads_handler))
        .route("/threads/{id}", get(get_thread_handler))
        .route("/threads/{id}/post", post(create_post_handler))
}
