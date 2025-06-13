use crate::data::dynamodb::Db;
use crate::data::error::Result;
use crate::data::migrations::{Migratable, Migrator};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Builder, Clone, Debug)]
pub struct DataRequest {
    /// `DataRequest` - Global partition key for all data requests
    #[builder(setter(custom))]
    pub pk: String,
    /// `ChatThread#<thread_id>#DataRequest#<request_id>`
    #[builder(setter(custom))]
    pub sk: String,

    /// The name of the data that is unavailable
    pub name: String,

    /// An explanation of why the data would be relevant to the user
    pub explanation: String,

    /// When the request was created
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_ts: DateTime<Utc>,

    /// The status of the request (e.g., "pending", "approved", "rejected")
    #[builder(default = "\"pending\".to_string()")]
    pub status: String,
}

impl DataRequestBuilder {
    /// Custom setter for `thread_id` and `request_id` that sets `sk` automatically.
    pub fn thread_and_request_ids(&mut self, thread_id: &str, request_id: &str) -> &mut Self {
        self.pk = Some("DataRequest".to_string());
        self.sk = Some(format!(
            "ChatThread#{}#DataRequest#{}",
            thread_id, request_id
        ));
        self
    }
}

impl DataRequest {
    /// Get the thread ID this request is associated with
    pub fn thread_id(&self) -> &str {
        self.sk.split('#').nth(1).unwrap()
    }

    pub fn id(&self) -> &str {
        self.sk.split('#').nth(3).unwrap()
    }

    /// Get all data requests across all threads
    pub async fn get_all_requests(db: &Db) -> Result<Vec<Self>> {
        // Build the query to get all data requests
        let query_builder = db
            .client
            .query()
            .table_name(&db.table_name)
            .key_condition_expression("#pk = :pk")
            .expression_attribute_names("#pk", "pk")
            .expression_attribute_values(":pk", AttributeValue::S("DataRequest".to_string()));

        // Execute the query
        let items = db.query_all(query_builder, None).await?;

        // Convert DynamoDB items to DataRequest structs
        let requests = items
            .into_iter()
            .map(serde_dynamo::from_item)
            .collect::<std::result::Result<Vec<Self>, _>>()?;

        Ok(requests)
    }
}

pub struct DataRequestMigrator;

#[async_trait]
impl Migrator for DataRequestMigrator {
    async fn migrate(
        _db: &Db,
        item: HashMap<String, AttributeValue>,
    ) -> Result<HashMap<String, AttributeValue>> {
        // No migrations needed for the initial version
        Ok(item)
    }
}

// Explicitly associate the Migrator with DataRequest
#[async_trait]
impl Migratable for DataRequest {
    type Migrator = DataRequestMigrator;
}
