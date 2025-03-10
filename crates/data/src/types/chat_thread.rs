use crate::error::Result;
use crate::migrations::{Migratable, Migrator};
use crate::{dynamodb::Db, error::DataError};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnValue};
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Builder)]
pub struct ChatThread {
    /// `User#<user_id>`
    #[builder(setter(custom))]
    pub pk: String,
    /// `ChatThread#<thread_id>`
    #[builder(setter(custom))]
    pub sk: String,

    pub title: String,

    /// The timestamp this thread was last modified.
    /// Note that this is not the same as the last message timestamp.
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub modified_ts: DateTime<Utc>,

    /// If `true`, this thread is archived and new messages can no longer be sent to it.
    /// `false` is equivalent to `None`.
    #[builder(default = "None")]
    pub archived: Option<bool>,
}
impl ChatThreadBuilder {
    /// Custom setter for `user_id` that sets `pk` automatically.
    pub fn user_id(&mut self, user_id: String) -> &mut Self {
        self.pk = Some(format!("User#{}", user_id));
        self
    }
    /// Custom setter for `thread_id` that sets `sk` automatically.
    pub fn id(&mut self, thread_id: String) -> &mut Self {
        self.sk = Some(format!("ChatThread#{}", thread_id));
        self
    }
}

impl ChatThread {
    pub fn user_id(&self) -> &str {
        self.pk.trim_start_matches("User#")
    }

    pub fn id(&self) -> &str {
        self.sk.trim_start_matches("ChatThread#")
    }

    pub async fn get_thread(db: &Db, user_id: &str, thread_id: &str) -> Result<Self> {
        let item = db
            .client
            .get_item()
            .table_name(&db.table_name)
            .key("pk", AttributeValue::S(format!("User#{}", user_id)))
            .key("sk", AttributeValue::S(format!("ChatThread#{}", thread_id)))
            .send()
            .await?
            .item;

        if let Some(item) = item {
            let thread = db.from_item(item).await?;
            Ok(thread)
        } else {
            Err(DataError::DocumentNotFound)
        }
    }

    pub async fn get_all_user_threads(db: &Db, user_id: &str) -> Result<Vec<Self>> {
        // Build the query
        let query_builder = db
            .client
            .query()
            .scan_index_forward(false)
            .key_condition_expression("#pk = :pk AND begins_with(#sk, :sk)")
            .expression_attribute_names("#pk", "pk")
            .expression_attribute_names("#sk", "sk")
            .expression_attribute_values(":pk", AttributeValue::S(format!("User#{}", user_id)))
            .expression_attribute_values(":sk", AttributeValue::S("ChatThread#".to_string()));

        // Use the query_all method with our prepared query builder
        let items = db.query_all(query_builder, None).await?;

        // Convert DynamoDB items to ChatThread structs
        let threads = try_join_all(items.into_iter().map(|item| db.from_item(item))).await?;

        Ok(threads)
    }
}

pub struct ChatThreadMigrator;

#[async_trait]
impl Migrator for ChatThreadMigrator {
    async fn migrate(
        db: &Db,
        item: HashMap<String, AttributeValue>,
    ) -> Result<HashMap<String, AttributeValue>> {
        let version = item
            .get("schema_version")
            .and_then(|v| v.as_n().ok())
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(1);

        if version == 1 {
            let pk = item.get("pk").and_then(|v| v.as_s().ok()).unwrap();
            let sk = item.get("sk").and_then(|v| v.as_s().ok()).unwrap();

            let updated_item = db
                .client
                .update_item()
                .table_name(&db.table_name)
                .key("pk", AttributeValue::S(pk.to_string()))
                .key("sk", AttributeValue::S(sk.to_string()))
                .update_expression(
                    "SET #modified_ts = :modified_ts, #schema_version = :schema_version",
                )
                .expression_attribute_names("#schema_version", "schema_version")
                .expression_attribute_names("#modified_ts", "modified_ts")
                .expression_attribute_values(
                    ":modified_ts",
                    AttributeValue::N(Utc::now().timestamp_millis().to_string()),
                )
                .expression_attribute_values(":schema_version", AttributeValue::N("2".into()))
                .return_values(ReturnValue::AllNew)
                .send()
                .await?
                .attributes
                .unwrap_or(item);

            Ok(updated_item)
        } else {
            Ok(item) // no migration needed
        }
    }
}

// Explicitly associate the Migrator with ChatThread
#[async_trait]
impl Migratable for ChatThread {
    type Migrator = ChatThreadMigrator;
}
