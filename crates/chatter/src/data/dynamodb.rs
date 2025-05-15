use crate::data::error::{DataError, Result};
use crate::data::migrations::Migratable;
use aws_config::Region;
use aws_sdk_dynamodb::operation::query::builders::QueryFluentBuilder;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_dynamo::aws_sdk_dynamodb_1::to_item;
use std::collections::HashMap;
use std::env;

#[cfg(test)]
const LOCAL_ENDPOINT_URL: Option<&str> = Some("http://localhost:9001");
#[cfg(not(test))]
const LOCAL_ENDPOINT_URL: Option<&str> = None;

fn get_endpoint_url() -> Option<String> {
    let endpoint_from_env = env::var("DYNAMODB_ENDPOINT_URL").ok();
    LOCAL_ENDPOINT_URL
        .or(endpoint_from_env.as_deref())
        .map(|s| s.to_string())
}

pub struct Db {
    pub client: aws_sdk_dynamodb::Client,
    pub table_name: String,
}

impl Db {
    async fn get_config() -> aws_config::SdkConfig {
        // Run this in dev/test, but not in release builds.
        #[cfg(any(debug_assertions, test))]
        if let Some(endpoint_url) = get_endpoint_url() {
            let config = aws_config::from_env()
                .endpoint_url(endpoint_url)
                .region(Region::new("us-east-1"))
                .test_credentials()
                .load()
                .await;
            return config;
        }
        
        aws_config::load_from_env().await
    }

    /// Creates a new `Db` by loading AWS config from the environment and reading TABLE_NAME.
    pub async fn new() -> Self {
        let config = Self::get_config().await;
        let client = aws_sdk_dynamodb::Client::new(&config);
        let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set in the environment");

        let db = Self { client, table_name };

        // Initialize the database if in dev/test environments and the endpoint override is set. When we're connecting to AWS DynamoDB, we don't want to initialize the database, but we do if we're connecting to a local DynamoDB instance.
        #[cfg(any(debug_assertions, test))]
        if get_endpoint_url().is_some() {
            db.init_schema().await;
        }

        db
    }

    /// Initialize the database. Only avaliable in dev/test environments.
    #[cfg(any(debug_assertions, test))]
    pub async fn init_schema(&self) {
        use crate::data::dynamodb_schema::create_table_if_not_exists;
        create_table_if_not_exists(self).await;
    }

    /// Put an item into DynamoDB.
    pub async fn put_item<T: Serialize>(&self, input: &T) -> Result<()> {
        let item = to_item(input)?;
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await?;
        Ok(())
    }

    /// Put an item into DynamoDB using an optimistic locking strategy.
    /// if the timestamp in `expected` is not equal to the timestamp in the item on the `ts_field` field,
    /// the put will fail.
    pub async fn put_item_lock<T: Serialize>(
        &self,
        input: &T,
        ts_field: &str,
        expected: &DateTime<Utc>,
    ) -> Result<()> {
        let item = to_item(input)?;
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .condition_expression("attribute_exists(#field) AND #field = :expected")
            .expression_attribute_values(
                ":expected",
                AttributeValue::N(expected.timestamp_millis().to_string()),
            )
            .expression_attribute_names("#field", ts_field)
            .send()
            .await
            .map_err(|err| {
                if let Some(true) = err
                    .as_service_error().map(|se| se.is_conditional_check_failed_exception())
                {
                    return DataError::OptimisticLockFailed;
                }
                DataError::DynamoPutItemError(err)
            })?;
        Ok(())
    }

    /// Exclusively put an item into DynamoDB.
    /// This will fail if a record with the same key attributes already exist.
    pub async fn put_item_excl<T: Serialize>(&self, input: &T) -> Result<()> {
        let item = to_item(input)?;
        self.client
            .put_item()
            .table_name(&self.table_name)
            .condition_expression("attribute_not_exists(pk) AND attribute_not_exists(sk)")
            .set_item(Some(item))
            .send()
            .await?;
        Ok(())
    }

    /// Execute a DynamoDB query with pagination, collecting all results.
    /// Takes a pre-built query builder and handles pagination.
    pub async fn query_all(
        &self,
        query_builder: QueryFluentBuilder,
        limit: Option<usize>,
    ) -> Result<Vec<HashMap<String, AttributeValue>>> {
        let mut items = Vec::new();
        let mut exclusive_start_key: Option<AttributeValue> = None;

        loop {
            // Clone the builder for each pagination request
            let mut query = query_builder.clone().table_name(&self.table_name);

            // Set limit if provided
            if let Some(limit_val) = limit {
                query = query.limit(limit_val as i32);
            }

            // Add exclusive start key for pagination if we have one
            if let Some(key) = &exclusive_start_key {
                query = query.exclusive_start_key("sk", key.clone());
            }

            let result = query.send().await?;

            if let Some(result_items) = result.items {
                items.extend(result_items);
            }

            exclusive_start_key = result.last_evaluated_key.and_then(|v| v.get("sk").cloned());

            // If there's no last evaluated key, we're done
            if exclusive_start_key.is_none() {
                if let Some(limit_val) = limit.filter(|&v| v < items.len()) {
                    items.truncate(limit_val);
                }
                break;
            }

            // If we have a limit and we've reached it, we're done
            if let Some(limit_val) = limit.filter(|&v| v < items.len()) {
                items.truncate(limit_val);
                break;
            }
        }

        Ok(items)
    }

    pub async fn from_item<T>(&self, item: HashMap<String, AttributeValue>) -> Result<T>
    where
        T: Migratable + std::marker::Send,
    {
        T::migrate_and_parse(self, item).await
    }
}

#[cfg(test)]
mod tests {
    use crate::data::types::{
        chat_message::{ChatMessage, ChatMessageBuilder},
        chat_thread::ChatThreadBuilder,
    };

    use super::*;
    /// A test that creates a table, puts a ChatMessage item, and retrieves it.
    #[tokio::test]
    async fn test_put_and_get_chat_message() {
        let db = Db::new().await;

        // Build a ChatMessage item using the builder.
        // Note: The custom setters automatically set the partition and sort keys.
        let chat_message = ChatMessageBuilder::default()
            .user_id("user123".to_string())
            .thread_message_ids("thread456".to_string(), 1)
            .msg(crate::chatter_message::ChatterMessage {
                role: crate::chatter_message::Role::User,
                tool_calls: None,
                tool_call_id: None,
                message: Some("Hello, world!".to_string()),
                sidecar: crate::chatter_message::ChatterMessageSidecar::None,
            })
            .build()
            .expect("Failed building ChatMessage");

        // Put the item into DynamoDB.
        db.put_item(&chat_message)
            .await
            .expect("Failed to put ChatMessage item");

        // Retrieve the item using our get_all_thread_messages method.
        let retrieved = ChatMessage::get_all_thread_messages(&db, "user123", "thread456")
            .await
            .expect("Get call failed");

        // Verify that we found records and that contents are correct.
        assert!(
            !retrieved.is_empty(),
            "Expected to find at least one chat message"
        );
        let first_message = &retrieved[0];
        assert_eq!(first_message.user_id(), "user123");
        assert_eq!(first_message.thread_id(), "thread456");
        assert_eq!(first_message.id(), 1);
        assert_eq!(first_message.msg.message.as_ref().unwrap(), "Hello, world!");
    }

    /// A test to ensure that getting non-existent chat messages returns an empty vector.
    #[tokio::test]
    async fn test_get_non_existent_chat_message() {
        let db = Db::new().await;

        // Try to get chat messages for a user/thread combination that doesn't exist.
        let retrieved =
            ChatMessage::get_all_thread_messages(&db, "nonexistent_user", "nonexistent_thread")
                .await
                .expect("Get call failed");

        assert!(
            retrieved.is_empty(),
            "Expected empty vector for a missing chat message record"
        );
    }

    /// A test to ensure optimistic locking works.
    #[tokio::test]
    async fn test_put_optimistic_lock() {
        let db = Db::new().await;

        // Build a ChatThread item using the builder.
        let original_ts = Utc::now();
        let chat_thread = ChatThreadBuilder::default()
            .user_id("user123".to_string())
            .id("thread456".to_string())
            .title("Test Thread".to_string())
            .modified_ts(original_ts)
            .archived(Some(false))
            .build()
            .expect("Failed building ChatThread");

        // Put the item into DynamoDB.
        db.put_item(&chat_thread)
            .await
            .expect("Failed to put ChatMessage item");

        // Attempt to update the item with an incorrect timestamp.
        let result = db
            .put_item_lock(
                &chat_thread,
                "modified_ts",
                &(original_ts - chrono::Duration::seconds(10)),
            )
            .await;

        assert!(result.is_err(), "Expected optimistic lock to fail");
        if let Err(DataError::OptimisticLockFailed) = result {
            // Expected error
        } else {
            panic!("Expected optimistic lock to fail");
        }
        // Now update the item with the correct timestamp.
        let result = db
            .put_item_lock(&chat_thread, "modified_ts", &original_ts)
            .await;
        assert!(result.is_ok(), "Expected optimistic lock to succeed");
        // Verify that the item was updated successfully.
        let retrieved = ChatMessage::get_all_thread_messages(&db, "user123", "thread456")
            .await
            .expect("Get call failed");
        assert!(
            !retrieved.is_empty(),
            "Expected to find at least one chat message after optimistic lock"
        );
        let first_message = &retrieved[0];
        assert_eq!(first_message.user_id(), "user123");
        assert_eq!(first_message.thread_id(), "thread456");
    }
}
