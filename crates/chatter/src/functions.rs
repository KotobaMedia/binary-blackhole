//! LLM functions that will be called by the LLM runtime.

use async_openai::types::FunctionObject;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, JsonSchema)]
struct DescribeTablesParams {
    table_names: Vec<String>,
}

pub fn describe_tables_definition() -> FunctionObject {
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

#[derive(Serialize, Deserialize, JsonSchema)]
struct QueryDatabaseParams {
    query: String,
}

pub fn query_database() -> FunctionObject {
    let parameters_schema = json!(schema_for!(QueryDatabaseParams));
    FunctionObject {
        name: "query_database".into(),
        description: Some("Query the database with the provided SQL query.".into()),
        parameters: Some(parameters_schema),
        strict: Some(true),
    }
}
