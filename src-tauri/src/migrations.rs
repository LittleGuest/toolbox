use tauri_plugin_sql::{Migration, MigrationKind};

pub(crate) fn migrations() -> Vec<Migration> {
    vec![Migration {
        version: 20250527101033,
        description: "create_datasource_info_table",
        sql: include_str!("../migrations/20250527101033_datasource_info.up.sql"),
        kind: MigrationKind::Up,
    }]
}
