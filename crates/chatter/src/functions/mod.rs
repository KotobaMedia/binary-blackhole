//! LLM functions that will be called by the LLM runtime.

use crate::chatter_context::ChatterContext;
use crate::chatter_message::ChatterMessage;
use crate::data::dynamodb::Db;
use crate::error::Result;
use async_openai::types::{ChatCompletionTool, ChatCompletionToolType, FunctionObject};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod impls;
mod utils;

pub use impls::describe_tables::DescribeTablesFunction;
pub use impls::query_database::QueryDatabaseFunction;
pub use impls::request_unavailable_data::RequestUnavailableDataFunction;
pub use utils::format_column;

/// Shared resources needed by functions
#[derive(Clone)]
pub struct SharedResources {
    pub chatter_context: Arc<Mutex<ChatterContext>>,
    pub pg: Arc<deadpool_postgres::Client>,
    pub ddb: Arc<Db>,
}

/// Trait defining the interface for LLM functions
pub trait LlmFunction: Send + Sync {
    /// Get the function name
    fn name(&self) -> &'static str;

    /// Get the function description
    fn description(&self) -> &'static str;

    /// Get the function parameters schema
    fn parameters_schema(&self) -> serde_json::Value;

    /// Get the tool definition
    fn tool(&self) -> ChatCompletionTool {
        ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: FunctionObject {
                name: self.name().into(),
                description: Some(self.description().into()),
                parameters: Some(self.parameters_schema()),
                strict: Some(true),
            },
        }
    }
}

/// Trait for executing LLM functions
#[async_trait]
pub trait LlmFunctionExecutor: Send + Sync {
    /// Execute the function with the given parameters
    async fn execute(
        &self,
        resources: &SharedResources,
        tool_call_id: String,
        params: serde_json::Value,
    ) -> Result<ChatterMessage>;
}

/// Combined trait for LLM functions that can be used as a trait object
pub trait LlmFunctionTrait: LlmFunction + LlmFunctionExecutor {}
impl<T: LlmFunction + LlmFunctionExecutor> LlmFunctionTrait for T {}

/// Registry for managing and dispatching LLM functions
pub struct FunctionRegistry {
    functions: HashMap<String, Arc<dyn LlmFunctionTrait>>,
}

impl FunctionRegistry {
    /// Create a new function registry
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Register a function with the registry
    pub fn register<F: LlmFunctionTrait + 'static>(&mut self, function: F) {
        self.functions
            .insert(function.name().to_string(), Arc::new(function));
    }

    /// Get all registered functions as tools
    pub fn get_tools(&self) -> Vec<ChatCompletionTool> {
        self.functions.values().map(|f| f.tool()).collect()
    }

    /// Execute a function by name
    pub async fn execute(
        &self,
        resources: &SharedResources,
        function_name: &str,
        tool_call_id: String,
        params: serde_json::Value,
    ) -> Result<ChatterMessage> {
        let function = self.functions.get(function_name).ok_or_else(|| {
            crate::error::ChatterError::FunctionNotFound(function_name.to_string())
        })?;

        function.execute(resources, tool_call_id, params).await
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
