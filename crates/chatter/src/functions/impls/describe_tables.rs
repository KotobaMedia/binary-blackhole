use crate::chatter_message::ChatterMessage;
use crate::error::Result;
use crate::functions::{LlmFunction, LlmFunctionExecutor, SharedResources, format_column};
use async_openai::types::Role;
use async_trait::async_trait;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct DescribeTablesParams {
    table_names: Vec<String>,
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

#[async_trait]
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
            sidecar: crate::chatter_message::ChatterMessageSidecar::DatabaseLookup,
        })
    }
}
