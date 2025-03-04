use crate::error::Result;
use crate::state::AppState;
use anyhow::Context;
use axum::{Json, Router, extract::State, routing::post};
use geo::{BoundingRect, Geometry, GeometryCollection};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
        geometries.push(Geometry::from(row.geom));
        if let Value::Object(props) = row.properties {
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

pub fn query_routes() -> Router<AppState> {
    Router::new().route("/query", post(post_query_handler))
}
