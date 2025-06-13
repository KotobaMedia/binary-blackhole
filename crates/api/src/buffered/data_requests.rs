use crate::error::Result;
use crate::state::AppState;
use axum::{Json, Router, extract::State, routing::get};
use chatter::data::types::data_request::DataRequest;
use serde::Serialize;

#[derive(Serialize)]
pub struct DataRequestView {
    pub id: String,
    pub thread_id: String,
    pub name: String,
    pub explanation: String,
    pub created_ts: chrono::DateTime<chrono::Utc>,
    pub status: String,
}

impl From<DataRequest> for DataRequestView {
    fn from(request: DataRequest) -> Self {
        Self {
            id: request.id().to_string(),
            thread_id: request.thread_id().to_string(),
            name: request.name,
            explanation: request.explanation,
            created_ts: request.created_ts,
            status: request.status,
        }
    }
}

#[derive(Serialize)]
pub struct DataRequestList {
    pub requests: Vec<DataRequestView>,
}

async fn get_data_requests_handler(State(state): State<AppState>) -> Result<Json<DataRequestList>> {
    let requests = DataRequest::get_all_requests(&state.ddb).await?;
    let requests: Vec<DataRequestView> = requests.into_iter().map(Into::into).collect();
    Ok(Json(DataRequestList { requests }))
}

pub fn data_requests_routes() -> Router<AppState> {
    Router::new().route("/data-requests", get(get_data_requests_handler))
}
