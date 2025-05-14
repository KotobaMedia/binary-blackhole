use crate::dynamodb::Db;
use crate::error::{DataError, Result};
use crate::migrations::{Migratable, Migrator};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Serialize, Deserialize, Builder, Clone, Debug)]
pub struct SqlQuery {
    /// `ChatThread#<thread_id>`
    #[builder(setter(custom))]
    pub pk: String,
    /// `SqlQuery#<query_id>`
    #[builder(setter(custom))]
    pub sk: String,

    /// The ID of the thread this query is associated with
    pub thread_id: String,

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

    /// [TTL] When the query will be deleted
    /// By default, this is set to 24 hours from the last access time.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub ttl: DateTime<Utc>,
}

impl SqlQueryBuilder {
    /// Custom setter for `thread_id` that sets `pk` automatically.
    pub fn thread_id(&mut self, thread_id: String) -> &mut Self {
        self.pk = Some(format!("ChatThread#{}", thread_id));
        self.thread_id = Some(thread_id);
        self
    }

    /// Custom setter for `query_id` that sets `sk` automatically.
    pub fn query_id(&mut self, query_id: String) -> &mut Self {
        self.sk = Some(format!("SqlQuery#{}", query_id));
        self
    }

    /// Generate a new query with default timestamps and a unique ID
    pub fn new_query(thread_id: String, name: String, content: String) -> Self {
        let now = Utc::now();
        let ttl = now + chrono::Duration::hours(24);
        let query_id = Ulid::new().to_string();

        let mut builder = SqlQueryBuilder::default();
        builder
            .thread_id(thread_id)
            .query_id(query_id)
            .query_name(name)
            .query_content(content)
            .created_ts(now)
            .modified_ts(now)
            .accessed_ts(now)
            .ttl(ttl);

        builder
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

    /// Create a new SQL query
    pub async fn create_query(
        db: &Db,
        thread_id: &str,
        query_name: &str,
        query_content: &str,
    ) -> Result<Self> {
        let query = SqlQueryBuilder::new_query(
            thread_id.to_string(),
            query_name.to_string(),
            query_content.to_string(),
        )
        .build()?;

        db.put_item(&query).await?;
        Ok(query)
    }

    /// Update an existing SQL query
    pub async fn update_query(
        db: &Db,
        thread_id: &str,
        query_id: &str,
        query_name: Option<&str>,
        query_content: Option<&str>,
    ) -> Result<Self> {
        // First retrieve the existing query
        let mut query = Self::get_query(db, thread_id, query_id).await?;

        // Update the fields if provided
        if let Some(name) = query_name {
            query.query_name = name.to_string();
        }

        if let Some(content) = query_content {
            query.query_content = content.to_string();
        }

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
