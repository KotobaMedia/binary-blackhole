use std::collections::HashMap;

use crate::dynamodb::Db;
use crate::error::Result;
use crate::migrations::{Migratable, Migrator};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Builder, Clone, Debug)]
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

pub struct ChatMessageMigrator;

#[async_trait]
impl Migrator for ChatMessageMigrator {
    async fn migrate(
        _db: &Db,
        item: HashMap<String, AttributeValue>,
    ) -> Result<HashMap<String, AttributeValue>> {
        let _version = item
            .get("schema_version")
            .and_then(|v| v.as_n().ok())
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(1);

        // Perform migration based on version
        // No migrations yet!

        Ok(item)
    }
}

// Explicitly associate the Migrator with ChatMessage
#[async_trait]
impl Migratable for ChatMessage {
    type Migrator = ChatMessageMigrator;
}
