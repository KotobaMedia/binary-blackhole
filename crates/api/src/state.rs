use chatter::chatter::Chatter;
use chatter::data::dynamodb::Db;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Application state shared across all requests.
/// This is a singleton. However, this runs in Lambda, so theoretically it only services
/// one request at a time. We're using Arc and Mutex just to satisfy the borrow checker.
/// Maybe that could be improved later.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub chatter: Arc<Mutex<Chatter>>,
}

impl AppState {
    pub async fn new() -> Self {
        let db = Db::new().await;
        let chatter = Chatter::new().await.unwrap();
        Self {
            db: Arc::new(db),
            chatter: Arc::new(Mutex::new(chatter)),
        }
    }
}
