use crate::chatter_message::ChatterMessage;
use crate::data::types::data_request::DataRequestBuilder;
use crate::error::{ChatterError, Result};
use crate::functions::{LlmFunction, LlmFunctionExecutor, SharedResources};
use async_openai::types::Role;
use async_trait::async_trait;
use chrono::Utc;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct RequestUnavailableDataParams {
    /// The name of the data that is unavailable.
    name: String,
    /// An explanation of why the data would be relevant to the user.
    explanation: String,
}

/// Implementation of LlmFunction for requesting unavailable data
pub struct RequestUnavailableDataFunction;

impl LlmFunction for RequestUnavailableDataFunction {
    fn name(&self) -> &'static str {
        "request_unavailable_data"
    }

    fn description(&self) -> &'static str {
        "Puts in a request for data that is currently unavailable."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!(schema_for!(RequestUnavailableDataParams))
    }
}

#[async_trait]
impl LlmFunctionExecutor for RequestUnavailableDataFunction {
    async fn execute(
        &self,
        resources: &SharedResources,
        tool_call_id: String,
        params: serde_json::Value,
    ) -> Result<ChatterMessage> {
        let params: RequestUnavailableDataParams = serde_json::from_value(params)?;

        // Get the thread ID from the context
        let thread_id = {
            let chatter_context = resources.chatter_context.lock().unwrap();
            chatter_context.id.clone()
        };

        // Create a new data request
        let request = DataRequestBuilder::default()
            .thread_and_request_ids(&thread_id, &ulid::Ulid::new().to_string())
            .name(params.name.clone())
            .explanation(params.explanation)
            .created_ts(Utc::now())
            .status("pending".to_string())
            .build()
            .map_err(|e| ChatterError::DataRequestCreationError(e.to_string()))?;

        // Store the request in DynamoDB
        resources
            .ddb
            .put_item(&request)
            .await
            .map_err(|e| ChatterError::DataRequestCreationError(e.to_string()))?;

        Ok(ChatterMessage {
            message: Some(format!(
                "I've submitted a request for the data '{}'. The data team will review this request and get back to you. Request ID: {}",
                params.name,
                request.id()
            )),
            role: Role::Tool,
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
            sidecar: crate::chatter_message::ChatterMessageSidecar::None,
        })
    }
}
