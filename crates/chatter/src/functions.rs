//! LLM functions that will be called by the LLM runtime.

use crate::chatter_message::SQLExecutionDetails;
use crate::data::dynamodb::Db;
use crate::data::types::sql_query::SqlQueryBuilder;
use crate::pg_helpers::{check_query, validate_query_rows};
use crate::rows_to_tsv::rows_to_tsv;
use crate::{
    chatter_message::{ChatterMessage, ChatterMessageSidecar},
    error::Result,
};
use async_openai::types::{ChatCompletionTool, ChatCompletionToolType, FunctionObject, Role};
use derive_builder::Builder;
use indoc::indoc;
use km_to_sql::metadata::ColumnMetadata;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct ExecutionContext {
    pg: Arc<tokio_postgres::Client>,
    ddb: Arc<Db>,
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

impl ExecutionContext {
    fn describe_tables_definition() -> FunctionObject {
        let parameters_schema = json!(schema_for!(DescribeTablesParams));
        FunctionObject {
            name: "describe_tables".into(),
            description: Some("Get detailed information about the requested tables.".into()),
            parameters: Some(parameters_schema),
            strict: Some(true),
        }
    }

    pub fn describe_tables_tool() -> ChatCompletionTool {
        ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: Self::describe_tables_definition(),
        }
    }

    pub async fn describe_tables(
        &self,
        tool_call_id: &str,
        params: DescribeTablesParams,
    ) -> Result<ChatterMessage> {
        let table_names: Vec<&str> = params.table_names.iter().map(|s| s.as_str()).collect();
        let rows = km_to_sql::postgres::get(&self.pg, &table_names).await?;

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
            tool_call_id: Some(tool_call_id.into()),
            sidecar: ChatterMessageSidecar::DatabaseLookup,
        })
    }

    fn query_database_definition() -> FunctionObject {
        let parameters_schema = json!(schema_for!(QueryDatabaseParams));
        FunctionObject {
            name: "query_database".into(),
            description: Some("Query the database and show results to the user. You will have access to a limited subset of the output.\nIf the query is not correct, an error message will be returned.\nIf an error is returned, rewrite the query and try again.\nIf the result set is empty, try again.\nWhen updating previous queries, provide the `query_id` parameter with the ID of the query you are updating.".into()),
            parameters: Some(parameters_schema),
            strict: Some(true),
        }
    }

    pub fn query_database_tool() -> ChatCompletionTool {
        ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: Self::query_database_definition(),
        }
    }

    pub async fn query_database(
        &self,
        tool_call_id: &str,
        params: QueryDatabaseParams,
    ) -> Result<ChatterMessage> {
        let query = params.query.trim_end_matches(';');
        let mut query_id = params.query_id;
        if query_id.is_empty() {
            query_id = ulid::Ulid::new().to_string();
        }

        let sample_size = 5;
        // Call the helper to check the query.
        let result = check_query(&self.pg, query, sample_size).await;

        match result {
            Ok(rows) => {
                if let Err(validation_error) = validate_query_rows(&rows) {
                    return Ok(ChatterMessage {
                        message: Some(validation_error.to_string()),
                        role: Role::Tool,
                        tool_calls: None,
                        tool_call_id: Some(tool_call_id.into()),
                        sidecar: ChatterMessageSidecar::SQLExecutionError,
                    });
                }
                {
                    use chrono::Utc;
                    let now = Utc::now();
                    let mut builder = SqlQueryBuilder::default();
                    builder
                        .thread_id("default".to_string())
                        .query_id(ulid::Ulid::new().to_string())
                        .query_name(params.name.clone())
                        .query_content(query.to_string())
                        .created_ts(now)
                        .modified_ts(now)
                        .accessed_ts(now);
                    let sql_query = builder.build().map_err(|e| {
                        crate::error::ChatterError::SqlQueryCreationError(e.to_string())
                    })?;
                    self.ddb.put_item(&sql_query).await.map_err(|e| {
                        crate::error::ChatterError::SqlQueryCreationError(e.to_string())
                    })?;
                }

                let tsv = rows_to_tsv(&rows);
                println!("SQL [{}]: {}", params.name, &query);
                Ok(ChatterMessage {
                    message: Some(format!(
                        indoc! {"
                            First {1} rows of the result set:

                            ```tsv
                            {0}
                            ```
                        "},
                        tsv,
                        rows.len()
                    )),
                    role: Role::Tool,
                    tool_calls: None,
                    tool_call_id: Some(tool_call_id.into()),
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
                    message: Some(message),
                    role: Role::Tool,
                    tool_calls: None,
                    tool_call_id: Some(tool_call_id.into()),
                    sidecar: ChatterMessageSidecar::SQLExecutionError,
                })
            }
        }
    }
}
