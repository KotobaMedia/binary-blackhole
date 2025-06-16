//! LLM functions that will be called by the LLM runtime.

use crate::chatter_context::ChatterContext;
use crate::chatter_message::SQLExecutionDetails;
use crate::data::dynamodb::Db;
use crate::data::types::data_request::DataRequestBuilder;
use crate::data::types::sql_query::SqlQueryBuilder;
use crate::error::ChatterError;
use crate::pg_helpers::{check_query, validate_query_rows};
use crate::rows_to_tsv::rows_to_tsv;
use crate::{
    chatter_message::{ChatterMessage, ChatterMessageSidecar},
    error::Result,
};
use async_openai::types::{ChatCompletionTool, ChatCompletionToolType, FunctionObject, Role};
use chrono::Utc;
use km_to_sql::metadata::ColumnMetadata;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
#[async_trait::async_trait]
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
        let function = self
            .functions
            .get(function_name)
            .ok_or_else(|| ChatterError::FunctionNotFound(function_name.to_string()))?;

        function.execute(resources, tool_call_id, params).await
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of LlmFunction for describing tables
pub struct DescribeTablesFunction;

impl LlmFunction for DescribeTablesFunction {
    fn name(&self) -> &'static str {
        "describe_tables"
    }

    fn description(&self) -> &'static str {
        "Get detailed information about the requested tables."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!(schema_for!(DescribeTablesParams))
    }
}

#[async_trait::async_trait]
impl LlmFunctionExecutor for DescribeTablesFunction {
    async fn execute(
        &self,
        resources: &SharedResources,
        tool_call_id: String,
        params: serde_json::Value,
    ) -> Result<ChatterMessage> {
        let params: DescribeTablesParams = serde_json::from_value(params)?;
        let table_names: Vec<&str> = params.table_names.iter().map(|s| s.as_str()).collect();
        let rows = km_to_sql::postgres::get(&resources.pg, &table_names).await?;

        let mut out = "".to_string();
        for (table_name, metadata) in rows {
            let mut table =
                format!("Table: `{}` (for humans: {})\n", table_name, metadata.name).to_string();
            if let Some(desc) = metadata.desc {
                table.push_str(&format!("- Description: {}\n", desc));
            }
            if let Some(pkey) = metadata.primary_key {
                table.push_str(&format!("- Primary key: {}\n", pkey));
            }
            if metadata.columns.is_empty() {
                table.push_str("- No columns found. This table is empty. Do not use this table in your queries.\n");
            } else {
                table.push_str("- Columns:\n");
                for column in metadata.columns {
                    table.push_str(&format_column(&column));
                }
                table.push('\n');
            }

            out.push_str(&table);
            out.push_str("\n\n");
        }

        Ok(ChatterMessage {
            message: Some(out),
            role: Role::Tool,
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
            sidecar: ChatterMessageSidecar::DatabaseLookup,
        })
    }
}

/// Implementation of LlmFunction for querying the database
pub struct QueryDatabaseFunction;

impl LlmFunction for QueryDatabaseFunction {
    fn name(&self) -> &'static str {
        "query_database"
    }

    fn description(&self) -> &'static str {
        "Query the database and show results to the user. You will have access to a limited subset of the output.\nIf the query is not correct, an error message will be returned.\nIf an error is returned, rewrite the query and try again.\nWhen updating previous queries, provide the `query_id` parameter with the ID of the query you are updating."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!(schema_for!(QueryDatabaseParams))
    }
}

#[async_trait::async_trait]
impl LlmFunctionExecutor for QueryDatabaseFunction {
    async fn execute(
        &self,
        resources: &SharedResources,
        tool_call_id: String,
        params: serde_json::Value,
    ) -> Result<ChatterMessage> {
        let params: QueryDatabaseParams = serde_json::from_value(params)?;
        let query = params.query.trim_end_matches(';');
        let mut query_id = params.query_id;
        if query_id.is_empty() {
            query_id = ulid::Ulid::new().to_string();
        }

        let sample_size = 5;
        // Call the helper to check the query.
        let result = check_query(&resources.pg, query, sample_size).await;

        match result {
            Ok(rows) => {
                if let Err(validation_error) = validate_query_rows(&rows) {
                    return Ok(ChatterMessage {
                        message: Some(
                            json!({
                                "query_id": query_id,
                                "error": true,
                                "message": validation_error.to_string(),
                            })
                            .to_string(),
                        ),
                        role: Role::Tool,
                        tool_calls: None,
                        tool_call_id: Some(tool_call_id),
                        sidecar: ChatterMessageSidecar::SQLExecutionError,
                    });
                }
                {
                    use chrono::Utc;
                    let now = Utc::now();
                    let mut builder = SqlQueryBuilder::default();
                    let thread_id = {
                        let chatter_context = resources.chatter_context.lock().unwrap();
                        chatter_context.id.clone()
                    };
                    builder
                        .thread_id(&thread_id)
                        .query_id(&query_id)
                        .query_name(params.name.clone())
                        .query_content(query.to_string())
                        .created_ts(now)
                        .modified_ts(now)
                        .accessed_ts(now);
                    let sql_query = builder
                        .build()
                        .map_err(|e| ChatterError::SqlQueryCreationError(e.to_string()))?;
                    resources
                        .ddb
                        .put_item(&sql_query)
                        .await
                        .map_err(|e| ChatterError::SqlQueryCreationError(e.to_string()))?;
                }

                let tsv = rows_to_tsv(&rows);
                println!("SQL [{}]: {}", params.name, &query);
                Ok(ChatterMessage {
                    message: Some(
                        json!({
                            "query_id": query_id,
                            "tsv": tsv,
                            "tsv_rows": rows.len(),
                        })
                        .to_string(),
                    ),
                    role: Role::Tool,
                    tool_calls: None,
                    tool_call_id: Some(tool_call_id),
                    sidecar: ChatterMessageSidecar::SQLExecution(SQLExecutionDetails {
                        id: query_id,
                        name: params.name,
                        sql: query.to_string(),
                    }),
                })
            }
            Err(e) => {
                let message = crate::pg_helpers::format_db_error(&e);
                Ok(ChatterMessage {
                    message: Some(
                        json!({
                            "query_id": query_id,
                            "error": true,
                            "message": message,
                        })
                        .to_string(),
                    ),
                    role: Role::Tool,
                    tool_calls: None,
                    tool_call_id: Some(tool_call_id),
                    sidecar: ChatterMessageSidecar::SQLExecutionError,
                })
            }
        }
    }
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

#[async_trait::async_trait]
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
            sidecar: ChatterMessageSidecar::None,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct DescribeTablesParams {
    table_names: Vec<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct QueryDatabaseParams {
    /// The ID of the query. When updating or revising a query, provide the ID of the query you want to update. If this is a new query, pass an empty string.
    query_id: String,

    /// The name this query will be referred to as. This will be shown to the user. It must be short and descriptive.
    name: String,

    /// The SQL query to execute.
    query: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct RequestUnavailableDataParams {
    /// The name of the data that is unavailable.
    name: String,
    /// An explanation of why the data would be relevant to the user.
    explanation: String,
}

fn format_column(column: &ColumnMetadata) -> String {
    let mut out = format!("  - `{}`", column.name);
    if let Some(ref desc) = column.desc {
        out.push_str(&format!(": {}", desc));
    }
    let mut annotations = vec![format!("type: {}", column.data_type)];
    if let Some(fk) = &column.foreign_key {
        annotations.push(format!(
            r#"foreign key: "{}"."{}""#,
            fk.foreign_table, fk.foreign_column
        ));
    }
    if let Some(enum_vs) = &column.enum_values {
        let mut enum_v_strs = vec![];
        for enum_v in enum_vs {
            let mut str = format!("`{}`", enum_v.value);
            if let Some(desc) = &enum_v.desc {
                str.push_str(&format!(": {}", desc));
            }
            enum_v_strs.push(str);
        }
        let enum_v_str = enum_v_strs.join(", ");
        annotations.push(format!("possible values: {}", enum_v_str));
    }
    out.push_str(&format!(" ({})", annotations.join(", ")));
    out.push('\n');
    out
}
