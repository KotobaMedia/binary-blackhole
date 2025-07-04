use super::{data_requests, datasets, query, threads};
use crate::error::Result as AppResult;
use crate::state::AppState;
use axum::http::Method;
use axum::response::Redirect;
use axum::routing::get;
use axum::{Router, extract::State};
use chatter::chatter::Chatter;
use tower_http::cors::{Any, CorsLayer};

async fn root() -> Redirect {
    Redirect::temporary("https://www.bblackhole.com/")
}
async fn health(State(state): State<AppState>) -> AppResult<String> {
    let pg = state.postgres_pool.get().await?;
    let mut chatter = Chatter::new(pg).await?;
    let rows = chatter
        .execute_raw_query(
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

pub async fn create_api() -> Router {
    let app_state = AppState::new().await;

    let cors = CorsLayer::new()
        .allow_headers([axum::http::header::CONTENT_TYPE])
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    Router::new()
        .route("/", get(root))
        .route("/__health", get(health))
        .merge(datasets::routes())
        .merge(threads::threads_routes())
        .merge(query::query_routes())
        .merge(data_requests::data_requests_routes())
        .layer(cors)
        .with_state(app_state)
}
