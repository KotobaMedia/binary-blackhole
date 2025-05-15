use crate::error::Result;
use crate::state::AppState;
use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use geo::{BoundingRect, Geometry, GeometryCollection};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Deserialize)]
struct PostQueryRequest {
    query: String,
}

/// The response to a query. A GeoJSON Feature Collection is returned.
#[derive(Serialize)]
struct PostQueryResponse {
    data: geojson::FeatureCollection,
    bbox: Option<[f64; 4]>, // minx, miny, maxx, maxy
}

async fn post_query_handler(
    State(state): State<AppState>,
    Json(payload): Json<PostQueryRequest>,
) -> Result<Json<PostQueryResponse>> {
    let mut chatter = state.chatter.lock().await;

    let rows = chatter
        .execute_query(&payload.query)
        .await
        .with_context(|| format!("when executing query: {}", &payload.query))?;

    let mut fc = geojson::FeatureCollection::default();
    let mut geometries: Vec<Geometry<f64>> = Vec::new();

    for row in rows {
        let mut feature = geojson::Feature::default();
        feature.geometry = Some((&row.geom).into());
        geometries.push(row.geom);
        if let Value::Object(props) = row.properties {
            if let Some(id) = props.get("_id") {
                feature.id = Some(geojson::feature::Id::String(id.to_string()));
            }
            feature.properties = Some(props);
        }
        fc.features.push(feature);
    }

    let combined_geometries = GeometryCollection::from(geometries);
    let combined_bbox = combined_geometries.bounding_rect();

    // Convert the geo bounding box to the response format [minx, miny, maxx, maxy]
    let bbox = combined_bbox.map(|bbox| [bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y]);

    Ok(Json(PostQueryResponse { data: fc, bbox }))
}

#[derive(Deserialize)]
struct GetTileQuery {
    q: String,
}

async fn get_tile_metadata_handler(
    State(state): State<AppState>,
    Query(query): Query<GetTileQuery>,
) -> Result<Json<serde_json::Value>> {
    let mut chatter = state.chatter.lock().await;

    let bbox = chatter
        .get_query_bbox(&query.q)
        .await
        .with_context(|| format!("when executing query: {}", &query.q))?;

    let escaped_q = urlencoding::encode(&query.q);

    Ok(Json(json!({
        "tilejson": "3.0.0",
        "scheme": "xyz",
        "tiles": [
            format!("http://localhost:9000/tile/{{z}}/{{x}}/{{y}}?q={}", escaped_q),
        ],
        "bounds": bbox,
        "minzoom": 0,
        "maxzoom": 18,
    })))
}

async fn get_tile_handler(
    State(state): State<AppState>,
    Path((z, x, y)): Path<(i32, i32, i32)>,
    Query(query): Query<GetTileQuery>,
) -> Result<Response> {
    let mut chatter = state.chatter.lock().await;

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
        .route("/query", post(post_query_handler))
        .route("/tile.json", get(get_tile_metadata_handler))
        .route("/tile/{z}/{x}/{y}", get(get_tile_handler))
}
