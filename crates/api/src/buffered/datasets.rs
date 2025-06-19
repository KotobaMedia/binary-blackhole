use crate::error::Result as AppResult;
use crate::state::AppState;
use axum::{Json, Router, extract::State, routing::get};
use km_to_sql::metadata::TableMetadata;
use serde::Serialize;

#[derive(Serialize)]
pub struct Table {
    pub table_name: String,
    #[serde(flatten)]
    pub metadata: TableMetadata,
}

#[derive(Serialize)]
pub struct TableList {
    pub tables: Vec<Table>,
}

pub async fn index(State(state): State<AppState>) -> AppResult<Json<TableList>> {
    let pg = state.postgres_pool.get().await?;
    // Fetch available table names from the datasets table
    let rows = pg
        .query(
            r#"
                SELECT "table_name" FROM "datasets"
            "#,
            &[],
        )
        .await?;
    let table_names: Vec<String> = rows.iter().map(|row| row.get(0)).collect();
    let table_name_refs: Vec<&str> = table_names.iter().map(|s| s.as_str()).collect();
    // Fetch metadata for each table
    let metadata = km_to_sql::postgres::get(&pg, &table_name_refs).await?;
    let tables = metadata
        .into_iter()
        .map(|(table_name, metadata)| Table {
            table_name,
            metadata,
        })
        .collect();
    Ok(Json(TableList { tables }))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/datasets", get(index))
}
