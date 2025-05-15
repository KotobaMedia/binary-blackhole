use crate::error::{ChatterError, Result};
use crate::rows_to_tsv::has_geometry_column;
use tokio_postgres::Row;

pub async fn check_query(
    client: &tokio_postgres::Client,
    query: &str,
    sample_size: usize,
) -> Result<Vec<Row>> {
    let sample_query = format!(
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
