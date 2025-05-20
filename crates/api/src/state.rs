use crate::error::Result;
use chatter::data::dynamodb::Db;
use deadpool_postgres::{Config, ManagerConfig, Pool, PoolConfig, RecyclingMethod, Runtime};
use std::{env, sync::Arc};
use tokio_postgres::NoTls;

/// Application state shared across all requests.
/// This is a singleton. However, this runs in Lambda, so theoretically it only services
/// one request at a time. We're using Arc just to satisfy the borrow checker.
/// Maybe that could be improved later.
#[derive(Clone)]
pub struct AppState {
    pub ddb: Arc<Db>,
    pub postgres_pool: Pool,
}

impl AppState {
    pub async fn new() -> Self {
        let db = Db::new().await;
        Self {
            ddb: Arc::new(db),
            postgres_pool: Self::get_postgres_pool().unwrap(),
        }
    }

    fn get_postgres_pool() -> Result<Pool> {
        let mut cfg = Config::new();
        let config = env::var("POSTGRES_CONN_STR")?;
        cfg.url = Some(config);
        cfg.pool = Some(PoolConfig {
            max_size: 1,
            ..Default::default()
        });
        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
        Ok(pool)
    }
}
