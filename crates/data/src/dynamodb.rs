use crate::error::Result;
use aws_sdk_dynamodb::types::AttributeValue;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_dynamo::aws_sdk_dynamodb_1::to_item;

pub struct Db {
    pub client: aws_sdk_dynamodb::Client,
    pub table_name: String,
}

impl Db {
    /// Creates a new `Db` by loading AWS config from the environment and reading TABLE_NAME.
    #[cfg(not(test))]
    pub async fn new() -> Self {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_dynamodb::Client::new(&config);
        let table_name =
            std::env::var("TABLE_NAME").expect("TABLE_NAME must be set in the environment");
        Self { client, table_name }
    }

    #[cfg(test)]
    pub async fn new() -> Self {
        use aws_config::Region;

        let config = aws_config::from_env()
            .endpoint_url("http://localhost:8000")
            .region(Region::new("us-east-1"))
            .test_credentials()
            .load()
            .await;
        let client = aws_sdk_dynamodb::Client::new(&config);
        let table_name = "TenantSettingsTestTable".into();
        Self { client, table_name }
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
    use aws_sdk_dynamodb::types::{
        AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
    };

    /// A helper function to create the test table if it does not exist.
    async fn create_table_if_not_exists(db: &Db) {
        let table_name = &db.table_name;
        let table = db
            .client
            .create_table()
            .table_name(table_name)
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("pk")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .unwrap(),
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("sk")
                    .attribute_type(ScalarAttributeType::S)
                    .build()
                    .unwrap(),
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("pk")
                    .key_type(KeyType::Hash)
                    .build()
                    .unwrap(),
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("sk")
                    .key_type(KeyType::Range)
                    .build()
                    .unwrap(),
            )
            .provisioned_throughput(
                ProvisionedThroughput::builder()
                    .read_capacity_units(5)
                    .write_capacity_units(5)
                    .build()
                    .unwrap(),
            )
            .send()
            .await;
        match table {
            Ok(_) => println!("Created table: {}", table_name),
            Err(_) => println!("Table probably exists: {}", table_name),
        }
    }

    /// A test that creates a table, puts a ChatMessage item, and retrieves it.
    #[tokio::test]
    async fn test_put_and_get_chat_message() {
        let db = Db::new().await;

        // Create the table if needed.
        create_table_if_not_exists(&db).await;

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

        // Make sure the table exists (idempotent).
        create_table_if_not_exists(&db).await;

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
