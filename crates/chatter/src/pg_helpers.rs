use crate::error::{ChatterError, Result};
use crate::rows_to_tsv::has_geometry_column;
use rust_decimal::Decimal;
use tokio_postgres::Row;

pub async fn check_query(
    client: &tokio_postgres::Client,
    query: &str,
    sample_size: usize,
) -> Result<Vec<Row>> {
    let sample_query = format!("SELECT * FROM ({}) AS t LIMIT {}", query, sample_size);
    let rows = client.query(&sample_query, &[]).await?;
    Ok(rows)
}

pub fn validate_query_rows(rows: &[tokio_postgres::Row]) -> Result<()> {
    if rows.is_empty() {
        Err(ChatterError::QueryError(
            "Failed to execute query: The result set is empty. Try again.".to_string(),
        ))
    } else if !has_geometry_column(&rows[0]) {
        Err(ChatterError::GeometryNotFound)
    } else {
        Ok(())
    }
}

// let's save this for later
// pub async fn create_matview(
//     client: &tokio_postgres::Client,
//     matview_name: &str,
//     query: &str,
// ) -> Result<()> {
//     // Create the materialized view
//     let create_query = format!("CREATE MATERIALIZED VIEW {} AS {}", matview_name, query);
//     client.execute(&create_query, &[]).await?;

//     // Sample one row to check for a geometry column
//     let sample_query = format!("SELECT * FROM ({}) AS t LIMIT 1", query);
//     let rows = client.query(&sample_query, &[]).await?;

//     if !rows.is_empty() && has_geometry_column(&rows[0]) {
//         // Create a spatial index on the "geom" column if geometry exists
//         let index_query = format!("CREATE INDEX ON {} USING GIST (geom)", matview_name);
//         client.execute(&index_query, &[]).await?;
//     }

//     Ok(())
// }

pub fn format_db_error(e: &crate::error::ChatterError) -> String {
    if let crate::error::ChatterError::PostgresError(pg_err) = e {
        if let Some(db_error) = pg_err.as_db_error() {
            format!(
                "Failed to execute query: {}{}{}",
                db_error.message(),
                db_error
                    .where_()
                    .map(|w| format!(", where: {}", w))
                    .unwrap_or_default(),
                db_error
                    .hint()
                    .map(|h| format!(", hint: {}", h))
                    .unwrap_or_default()
            )
        } else {
            format!("Failed to execute query: {}", pg_err)
        }
    } else {
        format!("Failed to execute query: {}", e)
    }
}

// Helper function to convert a column value to serde_json::Value based on its type.
pub fn convert_column_value(
    row: &tokio_postgres::Row,
    index: usize,
    column: &tokio_postgres::Column,
) -> serde_json::Value {
    match column.type_().name() {
        // Convert text types to JSON string.
        "varchar" | "text" => {
            let s: Option<String> = row.get(index);
            s.map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null)
        }
        // Convert integer types.
        "int4" => {
            let v: Option<i32> = row.get(index);
            v.map(|v| serde_json::Value::Number(v.into()))
                .unwrap_or(serde_json::Value::Null)
        }
        "int8" => {
            let v: Option<i64> = row.get(index);
            v.map(|v| serde_json::Value::Number(v.into()))
                .unwrap_or(serde_json::Value::Null)
        }
        // Convert floating point types.
        "float4" => {
            let v: Option<f32> = row.get(index);
            v.and_then(|v| serde_json::Number::from_f64(v as f64))
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        "float8" => {
            let v: Option<f64> = row.get(index);
            v.and_then(serde_json::Number::from_f64)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        // Convert boolean types.
        "bool" => {
            let v: Option<bool> = row.get(index);
            v.map(serde_json::Value::Bool)
                .unwrap_or(serde_json::Value::Null)
        }
        "numeric" => {
            let v: Option<Decimal> = row.get(index);
            v.map(|v| serde_json::Value::String(v.to_string()))
                .unwrap_or(serde_json::Value::Null)
        }
        // If the column is already in JSON format.
        "json" | "jsonb" => {
            let v: Option<serde_json::Value> = row.get(index);
            v.unwrap_or(serde_json::Value::Null)
        }
        // Fallback: attempt to get a string representation.
        _col_type_name => {
            let s: Option<String> = row.try_get(index).ok().flatten();
            s.map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null)
        }
    }
}
