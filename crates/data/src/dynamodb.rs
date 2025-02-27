use crate::error::Result;
use aws_config::Region;
use aws_sdk_dynamodb::types::AttributeValue;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_dynamo::aws_sdk_dynamodb_1::to_item;
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
        self.sk = Some(format!("ChatMessage#{}#{}", thread_id, message_id));
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

    pub fn message_id(&self) -> u32 {
        self.sk.split('#').nth(2).unwrap().parse().unwrap()
    }

    /// Retrieve all messages for a given thread.
    pub async fn get_all_thread_messages(
        db: &Db,
        user_id: &str,
        thread_id: &str,
    ) -> Result<Option<Self>> {
        let query = db
            .client
            .query()
            .table_name(&db.table_name)
            .key_condition_expression("#pk = :pk AND begins_with(#sk, :sk)")
            .expression_attribute_names("#pk", "pk")
            .expression_attribute_names("#sk", "sk")
            .expression_attribute_values(":pk", AttributeValue::S(format!("User#{}", user_id)))
            .expression_attribute_values(
                ":sk",
                AttributeValue::S(format!("ChatMessage#{}#", thread_id)),
            )
            .send()
            .await?;
        let items = query.items.unwrap_or_default();
        let item = items.into_iter().next();
        let item = match item {
            Some(item) => item,
            None => return Ok(None),
        };
        let parsed_item = serde_dynamo::from_item(item)?;
        Ok(Some(parsed_item))
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

        // Verify that we found a record and that its contents are correct.
        assert!(
            retrieved.is_some(),
            "Expected to find a chat message record"
        );
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.user_id(), "user123");
        assert_eq!(retrieved.thread_id(), "thread456");
        assert_eq!(retrieved.message_id(), 1);
        assert_eq!(retrieved.msg.message.unwrap(), "Hello, world!");
    }

    /// A test to ensure that getting a non-existent chat message returns `None`.
    #[tokio::test]
    async fn test_get_non_existent_chat_message() {
        let db = Db::new().await;

        // Try to get a chat message for a user/thread combination that doesn't exist.
        let retrieved =
            ChatMessage::get_all_thread_messages(&db, "nonexistent_user", "nonexistent_thread")
                .await
                .expect("Get call failed");

        assert!(
            retrieved.is_none(),
            "Expected `None` for a missing chat message record"
        );
    }
}
