use tokio_postgres::Row;

use crate::geom::GeometryWrapper;

macro_rules! try_get_as_string {
    ($row:expr, $col:expr, $ty:ty) => {
        if let Ok(val) = $row.try_get::<_, Option<$ty>>($col) {
            return val.map_or("NULL".to_string(), |v| v.to_string());
        }
    };
}

fn col_to_string(row: &tokio_postgres::Row, col: usize) -> String {
    // Try common types that implement Display/ToString.
    try_get_as_string!(row, col, String);
    try_get_as_string!(row, col, i32);
    try_get_as_string!(row, col, i64);
    try_get_as_string!(row, col, f32);
    try_get_as_string!(row, col, f64);
    try_get_as_string!(row, col, bool);
    try_get_as_string!(row, col, GeometryWrapper);

    // Fallback: use debug formatting.
    "unsupported".to_string()
}

pub fn rows_to_tsv(rows: &[Row]) -> String {
    if rows.is_empty() {
        "Empty result set.".to_string()
    } else {
        let cols = rows[0].columns();
        let headers: Vec<String> = cols
            .iter()
            .filter(|col| col.name() != "__rn")
            .map(|col| col.name().to_string())
            .collect();
        let mut tsv = headers.join("\t") + "\n";
        for row in rows {
            let mut row_values = Vec::new();
            for (i, col) in row.columns().iter().enumerate() {
                if col.name() == "__rn" {
                    continue;
                }
                // let col_type = col.type_().name().to_lowercase();
                let cell = col_to_string(row, i);
                row_values.push(cell);
            }
            tsv.push_str(&row_values.join("\t"));
            tsv.push('\n');
        }
        tsv
    }
}

pub fn has_geometry_column(row: &Row) -> bool {
    row.columns()
        .iter()
        .any(|col| col.type_().name() == "geometry")
}

#[cfg(test)]
mod tests {
    use std::env;

    // Use the real database for testing.
    use super::*;
    use tokio_postgres::{Error, NoTls};

    #[tokio::test]
    async fn test_rows_to_tsv_integration() -> Result<(), Error> {
        let connect_str = env::var("POSTGRES_CONN_STR_TEST").unwrap();
        let (client, connection) = tokio_postgres::connect(&connect_str, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        // Create a temporary table.
        client.execute("CREATE TEMP TABLE test_table (id serial PRIMARY KEY, name text, geom geometry, __rn int)", &[]).await?;
        // Insert sample data.
        client
            .execute(
                "INSERT INTO test_table (name, geom, __rn) VALUES ($1, ST_GeomFromText($2), $3)",
                &[
                    &"Alice",
                    &"POINT (130.46046626035297 30.371258529489126)",
                    &&1,
                ],
            )
            .await?;
        client
            .execute(
                "INSERT INTO test_table (name, geom, __rn) VALUES ($1, ST_GeomFromText($2), $3)",
                &[&"Bob", &None::<String>, &&2],
            )
            .await?;
        // Query rows.
        let rows = client.query("SELECT * FROM test_table", &[]).await?;
        let result = rows_to_tsv(&rows);
        let expected = "id\tname\tgeom\n1\tAlice\tPoint\n2\tBob\tNULL\n";
        assert_eq!(result, expected);
        Ok(())
    }
}
