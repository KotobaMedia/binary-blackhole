use crate::error::{ChatterError, Result};
use crate::rows_to_tsv::has_geometry_column;
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

pub async fn create_matview(
    client: &tokio_postgres::Client,
    matview_name: &str,
    query: &str,
) -> Result<()> {
    // Create the materialized view
    let create_query = format!("CREATE MATERIALIZED VIEW {} AS {}", matview_name, query);
    client.execute(&create_query, &[]).await?;

    // Sample one row to check for a geometry column
    let sample_query = format!("SELECT * FROM ({}) AS t LIMIT 1", query);
    let rows = client.query(&sample_query, &[]).await?;

    if !rows.is_empty() && has_geometry_column(&rows[0]) {
        // Create a spatial index on the "geom" column if geometry exists
        let index_query = format!("CREATE INDEX ON {} USING GIST (geom)", matview_name);
        client.execute(&index_query, &[]).await?;
    }

    Ok(())
}

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
