//! LLM functions that will be called by the LLM runtime.

use std::{fmt::Display, sync::Arc};

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

#[derive(Builder)]
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
    /// The name this query will be referred to as. This will be shown to the user. It should be short and descriptive.
    name: String,

    /// The SQL query to execute.
    query: String,
}

#[derive(Deserialize, Debug)]
struct DescribeTableAttribute {
    attr_name: String,
    attr_description: String,
    attr_type: String,
}

impl Display for DescribeTableAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "`{}`: {} ({})",
            self.attr_name, self.attr_description, self.attr_type
        )
    }
}

impl ExecutionContext {
    fn describe_tables_definition() -> FunctionObject {
        let parameters_schema = json!(schema_for!(DescribeTablesParams));
        FunctionObject {
            name: "describe_tables".into(),
            description: Some(
                "Query the database to get detailed information about the requested tables.".into(),
            ),
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
                        'attr_type', attr.value->>'attr_type'
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
            sidecar: ChatterMessageSidecar::None,
        })
    }

    fn query_database_definition() -> FunctionObject {
        let parameters_schema = json!(schema_for!(QueryDatabaseParams));
        FunctionObject {
            name: "query_database".into(),
            description: Some("Query the database with the provided SQL query.".into()),
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
        let message = format!("Database was queried successfully, and results shown to user.");
        println!("SQL [{}]: {}", params.name, params.query);

        Ok(ChatterMessage {
            message: Some(message),
            role: Role::Tool,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
            sidecar: ChatterMessageSidecar::SQLExecution((params.name, params.query)),
        })
    }
}
