use crate::data::{dynamodb::Db, error::Result};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::Deserialize;
use std::collections::HashMap;

#[async_trait]
pub trait Migratable: Sized + for<'a> Deserialize<'a> {
    type Migrator: Migrator;

    async fn migrate_and_parse(db: &Db, item: HashMap<String, AttributeValue>) -> Result<Self> {
        // Delegate migration explicitly to associated migrator
        let migrated_item = Self::Migrator::migrate(db, item).await?;
        serde_dynamo::from_item(migrated_item).map_err(|e| e.into())
    }
}

// Define a Migrator trait for migrations
#[async_trait]
pub trait Migrator {
    async fn migrate(
        db: &Db,
        item: HashMap<String, AttributeValue>,
    ) -> Result<HashMap<String, AttributeValue>>;
}
