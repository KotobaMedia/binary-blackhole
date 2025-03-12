use super::threads;
use crate::error::Result as AppResult;
use crate::state::AppState;
use async_stream::stream;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, StatusCode, header};
use axum::response::IntoResponse;
use axum::{Router, routing::get};
use std::convert::Infallible;
use tower_http::cors::{Any, CorsLayer};

async fn health(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
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

    let stream = stream! {
        yield Ok::<_, Infallible>("OK".to_string());
    };
    let body = Body::from_stream(stream);
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        body,
    )
        .into_response())
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
        .route("/__health", get(health))
        .merge(threads::threads_routes())
        .layer(cors)
        .with_state(app_state)
}
