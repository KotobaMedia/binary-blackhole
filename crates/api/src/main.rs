use crate::error::Result as AppResult;
use axum::http::Method;
use axum::response::Redirect;
use axum::routing::get;
use axum::{Router, extract::State};
use lambda_http::{Error, run, tracing};
use state::AppState;
use tower_http::cors::{Any, CorsLayer};

mod error;
mod query;
mod state;
mod threads;

async fn root() -> Redirect {
    Redirect::temporary("https://www.bblackhole.com/")
}
async fn health(State(state): State<AppState>) -> AppResult<String> {
    let mut chatter = state.chatter.lock().await;
    let rows = chatter
        .execute_query(
            r#"
            SELECT
                'hello' as "name",
                ST_Point(35, 135, 4326) as "geom"
        "#,
        )
        .await?;
    assert!(!rows.is_empty());
    let row = &rows[0];
    assert_eq!(
        row.properties.get("name"),
        Some(&serde_json::Value::String("hello".to_string()))
    );
    assert_eq!(row.geom, geo::Point::new(35.0, 135.0).into());
    Ok("OK".to_string())
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
        .merge(query::query_routes())
        .layer(cors)
        .with_state(app_state);

    run(app).await
}
