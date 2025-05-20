use crate::error::Result;
use crate::state::AppState;
use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use chatter::chatter::Chatter;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct GetTileQuery {
    q: String,
}

async fn get_tile_metadata_handler(
    headers: HeaderMap,
    Query(query): Query<GetTileQuery>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let pg = state.postgres_pool.get().await?;
    let mut chatter = Chatter::new(pg).await?;

    let bbox = chatter
        .get_query_bbox(&query.q)
        .await
        .with_context(|| format!("when executing query: {}", &query.q))?;

    let escaped_q = urlencoding::encode(&query.q);

    // Extract host and create base URL
    let host = headers
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:9000");

    // Determine protocol (use https if X-Forwarded-Proto indicates it)
    let protocol = headers
        .get("X-Forwarded-Proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("http");

    let base_url = format!("{}://{}", protocol, host);

    Ok(Json(json!({
        "tilejson": "3.0.0",
        "scheme": "xyz",
        "tiles": [
            format!("{}/tile/{{z}}/{{x}}/{{y}}?q={}", base_url, escaped_q),
        ],
        "bounds": bbox,
        "minzoom": 0,
        "maxzoom": 18,
    })))
}

async fn get_tile_handler(
    Path((z, x, y)): Path<(i32, i32, i32)>,
    Query(query): Query<GetTileQuery>,
    State(state): State<AppState>,
) -> Result<Response> {
    let pg = state.postgres_pool.get().await?;
    let mut chatter = Chatter::new(pg).await?;

    let tile = chatter
        .get_tile(&query.q, z, x, y)
        .await
        .with_context(|| format!("when getting tile: z={}, x={}, y={}", z, x, y))?;

    // Create a response with the appropriate content type
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/x-protobuf".parse().unwrap(),
    );

    Ok((StatusCode::OK, headers, tile).into_response())
}

pub fn query_routes() -> Router<AppState> {
    Router::new()
        .route("/tile.json", get(get_tile_metadata_handler))
        .route("/tile/{z}/{x}/{y}", get(get_tile_handler))
}
