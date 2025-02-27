use data::dynamodb::Db;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    db: Arc<Db>,
}

impl AppState {
    pub async fn new() -> Self {
        let db = Db::new().await;
        Self { db: Arc::new(db) }
    }
}
