use crate::data::dynamodb::Db;
use crate::data::error::{DataError, Result};
use crate::data::migrations::{Migratable, Migrator};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Builder, Clone, Debug)]
pub struct SqlQuery {
    /// `ChatThread#<thread_id>`
    #[builder(setter(custom))]
    pub pk: String,
    /// `SqlQuery#<query_id>`
    #[builder(setter(custom))]
    pub sk: String,

    /// A user-friendly name for the query
    pub query_name: String,

    /// The SQL query text
    pub query_content: String,

    /// When the query was created
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_ts: DateTime<Utc>,

    /// When the query was last modified
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub modified_ts: DateTime<Utc>,

    /// When the query was last accessed
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub accessed_ts: DateTime<Utc>,
}

impl SqlQueryBuilder {
    /// Custom setter for `thread_id` that sets `pk` automatically.
    pub fn thread_id(&mut self, thread_id: &str) -> &mut Self {
        self.pk = Some(format!("ChatThread#{}", thread_id));
        self
    }

    /// Custom setter for `query_id` that sets `sk` automatically.
    pub fn query_id(&mut self, query_id: &str) -> &mut Self {
        self.sk = Some(format!("SqlQuery#{}", query_id));
        self
    }
}

impl SqlQuery {
    /// Get the thread ID this query is associated with
    pub fn thread_id(&self) -> &str {
        self.pk.trim_start_matches("ChatThread#")
    }

    pub fn id(&self) -> &str {
        self.sk.trim_start_matches("SqlQuery#")
    }

    pub fn matview_name(&self) -> String {
        format!(
            "mv{}_{}",
            self.thread_id().to_ascii_lowercase(),
            self.id().to_ascii_lowercase()
        )
    }

    /// Update an existing SQL query
    pub async fn update_query_content(
        db: &Db,
        thread_id: &str,
        query_id: &str,
        query_content: &str,
    ) -> Result<Self> {
        // First retrieve the existing query
        let mut query = Self::get_query(db, thread_id, query_id).await?;

        query.query_content = query_content.to_string();

        // Update the modified timestamp
        query.modified_ts = Utc::now();

        // Save the updated query
        db.put_item(&query).await?;
        Ok(query)
    }

    /// Get a specific SQL query by ID
    pub async fn get_query(db: &Db, thread_id: &str, query_id: &str) -> Result<Self> {
        let item = db
            .client
            .get_item()
            .table_name(&db.table_name)
            .key("pk", AttributeValue::S(format!("ChatThread#{}", thread_id)))
            .key("sk", AttributeValue::S(format!("SqlQuery#{}", query_id)))
            .send()
            .await?
            .item;

        if let Some(item) = item {
            let query = db.from_item(item).await?;
            Ok(query)
        } else {
            Err(DataError::DocumentNotFound)
        }
    }

    /// Get all SQL queries for a specific thread
    pub async fn get_thread_queries(db: &Db, thread_id: &str) -> Result<Vec<Self>> {
        // Build the query to get all SQL queries for the thread
        let query_builder = db
            .client
            .query()
            .table_name(&db.table_name)
            .key_condition_expression("#pk = :pk AND begins_with(#sk, :sk)")
            .expression_attribute_names("#pk", "pk")
            .expression_attribute_names("#sk", "sk")
            .expression_attribute_values(
                ":pk",
                AttributeValue::S(format!("ChatThread#{}", thread_id)),
            )
            .expression_attribute_values(":sk", AttributeValue::S("SqlQuery#".to_string()));

        // Execute the query
        let items = db.query_all(query_builder, None).await?;

        // Convert DynamoDB items to SqlQuery structs
        let queries = try_join_all(items.into_iter().map(|item| db.from_item(item))).await?;

        Ok(queries)
    }
}

pub struct SqlQueryMigrator;

#[async_trait]
impl Migrator for SqlQueryMigrator {
    async fn migrate(
        _db: &Db,
        item: HashMap<String, AttributeValue>,
    ) -> Result<HashMap<String, AttributeValue>> {
        // No migrations needed for the initial version
        Ok(item)
    }
}

// Explicitly associate the Migrator with SqlQuery
#[async_trait]
impl Migratable for SqlQuery {
    type Migrator = SqlQueryMigrator;
}
