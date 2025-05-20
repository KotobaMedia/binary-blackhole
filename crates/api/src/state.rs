use chatter::data::dynamodb::Db;
use std::sync::Arc;

/// Application state shared across all requests.
/// This is a singleton. However, this runs in Lambda, so theoretically it only services
/// one request at a time. We're using Arc just to satisfy the borrow checker.
/// Maybe that could be improved later.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
}

impl AppState {
    pub async fn new() -> Self {
        let db = Db::new().await;
        Self { db: Arc::new(db) }
    }
}
