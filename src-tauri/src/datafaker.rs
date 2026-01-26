use std::collections::HashMap;

use datafaker::{Column, Faker};
use indexmap::IndexMap;

type Result<T> = std::result::Result<T, String>;

#[tauri::command]
pub async fn datafaker_providers() -> Result<HashMap<String, String>> {
    Ok(Faker::new().get_providers().clone())
}

#[tauri::command]
pub async fn datafaker_adapter(
    field_name: Option<String>,
    field_type: Option<String>,
) -> Result<String> {
    datafaker::datafaker_adapter(field_name, field_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn datafaker_adapter_columns(columns: Vec<Column>) -> Result<IndexMap<String, String>> {
    let mut res = IndexMap::new();
    for column in columns {
        let generator =
            datafaker_adapter(Some(column.name.clone()), Some(column.column_type.clone())).await?;
        res.insert(column.name, generator);
    }
    Ok(res)
}

#[tauri::command]
pub async fn preview_regex(pattern: String) -> Result<String> {
    datafaker::preview_regex(pattern)
        .await
        .map_err(|e| e.to_string())
}
