use crate::error::{DataError, Result};
use aws_config::Region;
use aws_sdk_dynamodb::operation::query::builders::QueryFluentBuilder;
use aws_sdk_dynamodb::types::AttributeValue;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
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
        let config = aws_config::load_from_env().await;
        config
    }

    /// Creates a new `Db` by loading AWS config from the environment and reading TABLE_NAME.
    pub async fn new() -> Self {
        let config = Self::get_config().await;
        let client = aws_sdk_dynamodb::Client::new(&config);
        let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set in the environment");

        let db = Self { client, table_name };

        // Initialize the database if in dev/test environments and the endpoint override is set. When we're connecting to AWS DynamoDB, we don't want to initialize the database, but we do if we're connecting to a local DynamoDB instance.
        #[cfg(any(debug_assertions, test))]
        if let Some(_) = get_endpoint_url() {
            db.init_schema().await;
        }

        db
    }

    /// Initialize the database. Only avaliable in dev/test environments.
    #[cfg(any(debug_assertions, test))]
    pub async fn init_schema(&self) {
        use crate::dynamodb_schema::create_table_if_not_exists;
        create_table_if_not_exists(self).await;
    }

    /// Put an item into DynamoDB.
    pub async fn put_item<T: Serialize>(&self, input: T) -> Result<()> {
        let item = to_item(input)?;
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await?;
        Ok(())
    }

    /// Exclusively put an item into DynamoDB.
    /// This will fail if a record with the same key attributes already exist.
    pub async fn put_item_excl<T: Serialize>(&self, input: T) -> Result<()> {
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
}

#[derive(Serialize, Deserialize, Builder)]
pub struct ChatThread {
    /// `User#<user_id>`
    #[builder(setter(custom))]
    pub pk: String,
    /// `ChatThread#<thread_id>`
    #[builder(setter(custom))]
    pub sk: String,

    pub title: String,
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
            let thread = serde_dynamo::from_item(item)?;
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
            .key_condition_expression("#pk = :pk AND begins_with(#sk, :sk)")
            .expression_attribute_names("#pk", "pk")
            .expression_attribute_names("#sk", "sk")
            .expression_attribute_values(":pk", AttributeValue::S(format!("User#{}", user_id)))
            .expression_attribute_values(":sk", AttributeValue::S("ChatThread#".to_string()));

        // Use the query_all method with our prepared query builder
        let items = db.query_all(query_builder, None).await?;

        // Convert DynamoDB items to ChatThread structs
        let threads = items
            .into_iter()
            .map(|item| serde_dynamo::from_item(item))
            .collect::<std::result::Result<Vec<Self>, _>>()?;

        Ok(threads)
    }
}

#[derive(Serialize, Deserialize, Builder)]
pub struct ChatMessage {
    /// `User#<user_id>`
    #[builder(setter(custom))]
    pub pk: String,
    /// `ChatMessage#<thread_id>#<message_id>`
    #[builder(setter(custom))]
    pub sk: String,

    pub msg: chatter::chatter_message::ChatterMessage,
}

impl ChatMessageBuilder {
    /// Custom setter for `user_id` that sets `pk` automatically.
    pub fn user_id(&mut self, user_id: String) -> &mut Self {
        self.pk = Some(format!("User#{}", user_id));
        self
    }

    /// Custom setter for `thread_id` and `message_id` that sets `sk` automatically.
    pub fn thread_message_ids(&mut self, thread_id: String, message_id: u32) -> &mut Self {
        // pad the message_id with zeros so DynamoDB sorts it correctly
        // we shouldn't need more than 3 digits (a 999 message thread is already way too long)
        self.sk = Some(format!("ChatMessage#{}#{:03}", thread_id, message_id));
        self
    }
}

impl ChatMessage {
    pub fn user_id(&self) -> &str {
        self.pk.trim_start_matches("User#")
    }

    pub fn thread_id(&self) -> &str {
        self.sk.split('#').nth(1).unwrap()
    }

    pub fn id(&self) -> u32 {
        self.sk.split('#').nth(2).unwrap().parse().unwrap()
    }

    /// Retrieve all messages for a given thread.
    pub async fn get_all_thread_messages(
        db: &Db,
        user_id: &str,
        thread_id: &str,
    ) -> Result<Vec<Self>> {
        // Build the query
        let query_builder = db
            .client
            .query()
            .key_condition_expression("#pk = :pk AND begins_with(#sk, :sk)")
            .expression_attribute_names("#pk", "pk")
            .expression_attribute_names("#sk", "sk")
            .expression_attribute_values(":pk", AttributeValue::S(format!("User#{}", user_id)))
            .expression_attribute_values(
                ":sk",
                AttributeValue::S(format!("ChatMessage#{}#", thread_id)),
            );

        // Use the query_all method with our prepared query builder
        let items = db.query_all(query_builder, None).await?;

        // Convert DynamoDB items to ChatMessage structs
        let messages = items
            .into_iter()
            .map(|item| serde_dynamo::from_item(item))
            .collect::<std::result::Result<Vec<Self>, _>>()?;

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
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
            .msg(chatter::chatter_message::ChatterMessage {
                role: chatter::chatter_message::Role::User,
                tool_calls: None,
                tool_call_id: None,
                message: Some("Hello, world!".to_string()),
                sidecar: chatter::chatter_message::ChatterMessageSidecar::None,
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
}
