//! LLM functions that will be called by the LLM runtime.

use std::{collections::HashMap, fmt::Display, sync::Arc};

use crate::rows_to_tsv::{has_geometry_column, rows_to_tsv};
use crate::{
    chatter_message::{ChatterMessage, ChatterMessageSidecar},
    error::Result,
};
use async_openai::types::{ChatCompletionTool, ChatCompletionToolType, FunctionObject, Role};
use derive_builder::Builder;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_postgres::types::Json;

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct ExecutionContext {
    client: Arc<tokio_postgres::Client>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct DescribeTablesParams {
    table_names: Vec<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct QueryDatabaseParams {
    /// The name this query will be referred to as. This will be shown to the user. It must be short and descriptive.
    name: String,

    /// The SQL query to execute.
    query: String,
}

#[derive(Deserialize, Debug)]
struct DescribeTableAttribute {
    attr_name: String,
    attr_description: String,
    attr_type: String,
    attr_ref: Option<RefType>,
}

#[derive(Debug, Deserialize)]
enum RefType {
    Enum(Vec<String>),
    Code(HashMap<String, String>),
}

impl Display for DescribeTableAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "`{}`: {} ({})",
            self.attr_name, self.attr_description, self.attr_type
        )?;
        if let Some(ref_type) = &self.attr_ref {
            match ref_type {
                RefType::Enum(values) => {
                    write!(f, " possible values: {}", values.join(", "))?;
                }
                RefType::Code(code_map) => {
                    write!(
                        f,
                        " coded values: {}",
                        code_map
                            .iter()
                            .map(|(k, v)| format!("`{}`: {}", k, v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )?;
                }
            }
        }
        Ok(())
    }
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
        let rows = self.client.query(r#"
            SELECT
                "table_name",
                "metadata"->'data_item'->>'name' AS "name",
                "metadata"->'data_page'->'metadata'->'fundamental'->>'内容' AS "description",
                "metadata"->'data_page'->'metadata'->'fundamental'->>'データ形状' AS "data_shape",
                jsonb_agg(
                    jsonb_build_object(
                        'attr_name', attr.value->>'name',
                        'attr_description', attr.value->>'description',
                        'attr_type', attr.value->>'attr_type',
                        'attr_ref', attr.value->'ref'
                    )
                ) AS attributes
            FROM datasets
            CROSS JOIN LATERAL jsonb_each("metadata"->'data_page'->'metadata'->'attribute') AS attr(key, value)
            WHERE "table_name" = ANY($1)
            GROUP BY
                "table_name",
                "metadata"->'data_item'->>'name',
                "metadata"->'data_page'->'metadata'->'fundamental'->>'内容',
                "metadata"->'data_page'->'metadata'->'fundamental'->>'データ形状';
        "#, &[&params.table_names]).await?;

        let mut out = "".to_string();
        for row in rows {
            let table_name: String = row.get("table_name");
            let name: String = row.get("name");
            let description: String = row.get("description");
            let data_shape: String = row.get("data_shape");
            let mut table = format!(
                "Table: `{}` (geom shape: {})\nDescription: {}\n{}\nAttributes:",
                table_name, data_shape, name, description
            )
            .to_string();
            let attributes: Json<Vec<DescribeTableAttribute>> = row.get("attributes");
            if attributes.0.is_empty() {
                table.push_str("\n- No attributes found. This table is empty. Do not use this table in your queries.");
            }
            for attr in attributes.0 {
                table.push_str(&format!("\n- {}", attr));
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
            description: Some("Query the database and show results to the user. You will have access to a limited subset of the output.\nIf the query is not correct, an error message will be returned.\nIf an error is returned, rewrite the query and try again.\nIf the result set is empty, try again.".into()),
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

        let sample_size = 5;
        let explain_query = format!(
            r#"
            WITH numbered AS (
                SELECT row_number() OVER () AS __rn, t.*
                FROM ({}) AS t
            ), total AS (
                SELECT count(*) AS cnt FROM numbered
            ), random_indices AS (
                SELECT floor(random() * cnt)::int + 1 as __rn
                FROM total, generate_series(1, {})
            )
            SELECT *
            FROM numbered
            WHERE __rn IN (
                SELECT __rn FROM random_indices
            )
            ORDER BY __rn;
            "#,
            query, sample_size,
        );
        let result = self.client.query(&explain_query, &[]).await;

        match result {
            Ok(rows) => {
                if rows.len() == 0 {
                    return Ok(ChatterMessage {
                        message: Some(
                            "Failed to execute query: The result set is empty. Try again."
                                .to_string(),
                        ),
                        role: Role::Tool,
                        tool_calls: None,
                        tool_call_id: Some(tool_call_id.into()),
                        sidecar: ChatterMessageSidecar::SQLExecutionError,
                    });
                }

                if !has_geometry_column(&rows[0]) {
                    return Ok(ChatterMessage {
                        message: Some(
                            "Failed to execute query: `geom` column was not in the result set."
                                .to_string(),
                        ),
                        role: Role::Tool,
                        tool_calls: None,
                        tool_call_id: Some(tool_call_id.into()),
                        sidecar: ChatterMessageSidecar::SQLExecutionError,
                    });
                }

                let tsv = rows_to_tsv(&rows);
                println!("SQL [{}]: {}", params.name, &query);
                return Ok(ChatterMessage {
                    message: Some(format!("
                        Result: \n{}\n
                        Note: This is a random sample of the result set.\nDon't reveal it to the user, but you may use it to help followup questions.",
                        tsv)),
                    role: Role::Tool,
                    tool_calls: None,
                    tool_call_id: Some(tool_call_id.into()),
                    sidecar: ChatterMessageSidecar::SQLExecution((params.name, query.to_string())),
                });
            }
            Err(e) => {
                let message = if let Some(db_error) = e.as_db_error() {
                    format!(
                        "Failed to execute query: {}{}{}",
                        db_error.message(),
                        db_error
                            .where_()
                            .map(|where_| format!(", where: {}", where_))
                            .unwrap_or_default(),
                        db_error
                            .hint()
                            .map(|hint| format!(", hint: {}", hint))
                            .unwrap_or_default()
                    )
                } else {
                    format!("Failed to execute query: {}", e)
                };
                return Ok(ChatterMessage {
                    message: Some(message),
                    role: Role::Tool,
                    tool_calls: None,
                    tool_call_id: Some(tool_call_id.into()),
                    sidecar: ChatterMessageSidecar::SQLExecutionError,
                });
            }
        }
    }
}
