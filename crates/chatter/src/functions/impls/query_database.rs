use crate::chatter_message::{ChatterMessage, ChatterMessageSidecar, SQLExecutionDetails};
use crate::data::types::sql_query::SqlQueryBuilder;
use crate::error::{ChatterError, Result};
use crate::functions::{LlmFunction, LlmFunctionExecutor, SharedResources};
use crate::pg_helpers::{check_query, validate_query_rows};
use crate::rows_to_tsv::rows_to_tsv;
use async_openai::types::Role;
use async_trait::async_trait;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;

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

#[async_trait]
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
