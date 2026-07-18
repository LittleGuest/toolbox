use std::{path::PathBuf, str::FromStr};

use database::{DatasourceInfo, Driver};
use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions};

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug)]
pub struct ClipboardHistoryItem {
    pub id: i64,
    pub content: String,
    pub created_at: i64,
}

#[derive(Clone, Debug)]
pub struct SnippetRecord {
    pub id: Option<i64>,
    pub title: String,
    pub code: String,
    pub tags: Vec<String>,
    pub language: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug)]
pub struct TodoRecord {
    pub id: Option<i64>,
    pub parent_id: Option<i64>,
    pub content: String,
    pub completed: bool,
    pub created_at: i64,
}

#[derive(Clone, Debug)]
pub struct DatafakerConfigRecord {
    pub id: Option<i64>,
    pub name: String,
    pub nodes_json: String,
    pub edges_json: String,
    pub updated_at: i64,
}

#[derive(Clone, Debug)]
pub struct ExcalidrawDocRecord {
    pub id: Option<i64>,
    pub name: String,
    pub elements_json: String,
    pub updated_at: i64,
}

pub async fn save_datasource(info: DatasourceInfo) -> Result<()> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let payload = serde_json::to_string(&info).map_err(|err| err.to_string())?;
    sqlx::query(
        "INSERT INTO data_sources (name, info, updated_at) VALUES (?1, ?2, strftime('%s','now')) \
         ON CONFLICT(name) DO UPDATE SET info = excluded.info, updated_at = excluded.updated_at",
    )
    .bind(&info.name)
    .bind(payload)
    .execute(&pool)
    .await
    .map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn load_datasources() -> Result<Vec<DatasourceInfo>> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let rows = sqlx::query("SELECT info FROM data_sources ORDER BY updated_at DESC, id DESC")
        .fetch_all(&pool)
        .await
        .map_err(|err| err.to_string())?;
    rows.into_iter()
        .map(|row| {
            let payload: String = row.try_get("info").map_err(|err| err.to_string())?;
            serde_json::from_str(&payload).map_err(|err| err.to_string())
        })
        .collect()
}

pub async fn delete_datasource(name: String) -> Result<bool> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let result = sqlx::query("DELETE FROM data_sources WHERE name = ?1")
        .bind(name)
        .execute(&pool)
        .await
        .map_err(|err| err.to_string())?;
    Ok(result.rows_affected() > 0)
}

pub async fn add_clipboard_history(content: String) -> Result<()> {
    let value = content.trim().to_string();
    if value.is_empty() {
        return Ok(());
    }
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    sqlx::query("DELETE FROM clipboard_history WHERE content = ?1")
        .bind(&value)
        .execute(&pool)
        .await
        .map_err(|err| err.to_string())?;
    sqlx::query(
        "INSERT INTO clipboard_history (content, created_at) VALUES (?1, strftime('%s','now'))",
    )
    .bind(value)
    .execute(&pool)
    .await
    .map_err(|err| err.to_string())?;
    sqlx::query(
        "DELETE FROM clipboard_history WHERE id NOT IN (\
            SELECT id FROM clipboard_history ORDER BY created_at DESC, id DESC LIMIT 100\
        )",
    )
    .execute(&pool)
    .await
    .map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn load_clipboard_history(keyword: String) -> Result<Vec<ClipboardHistoryItem>> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let keyword = keyword.trim().to_string();
    let rows = if keyword.is_empty() {
        sqlx::query(
            "SELECT id, content, created_at FROM clipboard_history ORDER BY created_at DESC, id DESC",
        )
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query(
            "SELECT id, content, created_at FROM clipboard_history \
             WHERE content LIKE ?1 ORDER BY created_at DESC, id DESC",
        )
        .bind(format!("%{keyword}%"))
        .fetch_all(&pool)
        .await
    }
    .map_err(|err| err.to_string())?;

    rows.into_iter()
        .map(|row| {
            Ok(ClipboardHistoryItem {
                id: row.try_get("id").map_err(|err| err.to_string())?,
                content: row.try_get("content").map_err(|err| err.to_string())?,
                created_at: row.try_get("created_at").map_err(|err| err.to_string())?,
            })
        })
        .collect()
}

pub async fn delete_clipboard_history(id: i64) -> Result<bool> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let result = sqlx::query("DELETE FROM clipboard_history WHERE id = ?1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|err| err.to_string())?;
    Ok(result.rows_affected() > 0)
}

pub async fn clear_clipboard_history() -> Result<()> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    sqlx::query("DELETE FROM clipboard_history")
        .execute(&pool)
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}

// ===== Snippet CRUD =====

pub async fn save_snippet(record: SnippetRecord) -> Result<i64> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let tags_json = serde_json::to_string(&record.tags).map_err(|err| err.to_string())?;
    let result = sqlx::query(
        "INSERT INTO snippets (title, code, tags, language, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(&record.title)
    .bind(&record.code)
    .bind(&tags_json)
    .bind(&record.language)
    .bind(record.created_at)
    .bind(record.updated_at)
    .execute(&pool)
    .await
    .map_err(|err| err.to_string())?;
    Ok(result.last_insert_rowid())
}

pub async fn load_snippets() -> Result<Vec<SnippetRecord>> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let rows = sqlx::query(
        "SELECT id, title, code, tags, language, created_at, updated_at FROM snippets ORDER BY updated_at DESC, id DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| err.to_string())?;
    rows.into_iter()
        .map(|row| {
            let tags_json: String = row.try_get("tags").map_err(|err| err.to_string())?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            Ok(SnippetRecord {
                id: Some(row.try_get("id").map_err(|err| err.to_string())?),
                title: row.try_get("title").map_err(|err| err.to_string())?,
                code: row.try_get("code").map_err(|err| err.to_string())?,
                tags,
                language: row.try_get("language").unwrap_or_default(),
                created_at: row.try_get("created_at").map_err(|err| err.to_string())?,
                updated_at: row.try_get("updated_at").map_err(|err| err.to_string())?,
            })
        })
        .collect()
}

pub async fn update_snippet(id: i64, record: SnippetRecord) -> Result<()> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let tags_json = serde_json::to_string(&record.tags).map_err(|err| err.to_string())?;
    sqlx::query(
        "UPDATE snippets SET title = ?1, code = ?2, tags = ?3, language = ?4, updated_at = ?5 WHERE id = ?6",
    )
    .bind(&record.title)
    .bind(&record.code)
    .bind(&tags_json)
    .bind(&record.language)
    .bind(record.updated_at)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn delete_snippet(id: i64) -> Result<bool> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let result = sqlx::query("DELETE FROM snippets WHERE id = ?1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|err| err.to_string())?;
    Ok(result.rows_affected() > 0)
}

// ===== Todo CRUD =====

pub async fn save_todo(record: TodoRecord) -> Result<i64> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let result = sqlx::query(
        "INSERT INTO todos (parent_id, content, completed, created_at) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(record.parent_id)
    .bind(&record.content)
    .bind(if record.completed { 1 } else { 0 })
    .bind(record.created_at)
    .execute(&pool)
    .await
    .map_err(|err| err.to_string())?;
    Ok(result.last_insert_rowid())
}

pub async fn load_todos() -> Result<Vec<TodoRecord>> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let rows = sqlx::query(
        "SELECT id, parent_id, content, completed, created_at FROM todos ORDER BY completed ASC, created_at DESC, id DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| err.to_string())?;
    rows.into_iter()
        .map(|row| {
            Ok(TodoRecord {
                id: Some(row.try_get("id").map_err(|err| err.to_string())?),
                parent_id: row.try_get("parent_id").map_err(|err| err.to_string())?,
                content: row.try_get("content").map_err(|err| err.to_string())?,
                completed: {
                    let v: i64 = row.try_get("completed").map_err(|err| err.to_string())?;
                    v != 0
                },
                created_at: row.try_get("created_at").map_err(|err| err.to_string())?,
            })
        })
        .collect()
}

pub async fn update_todo(id: i64, record: TodoRecord) -> Result<()> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    sqlx::query("UPDATE todos SET parent_id = ?1, content = ?2, completed = ?3 WHERE id = ?4")
        .bind(record.parent_id)
        .bind(&record.content)
        .bind(if record.completed { 1 } else { 0 })
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn delete_todo(id: i64) -> Result<bool> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let result = sqlx::query("DELETE FROM todos WHERE id = ?1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|err| err.to_string())?;
    Ok(result.rows_affected() > 0)
}

// ===== App Settings =====

pub async fn get_setting(key: &str) -> Result<Option<String>> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = ?1")
        .bind(key)
        .fetch_optional(&pool)
        .await
        .map_err(|err| err.to_string())?;
    match row {
        Some(row) => {
            let value: String = row.try_get("value").map_err(|err| err.to_string())?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

pub async fn set_setting(key: &str, value: &str) -> Result<()> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(&pool)
    .await
    .map_err(|err| err.to_string())?;
    Ok(())
}

// ===== Datafaker Config CRUD =====

pub async fn save_datafaker_config(name: &str, nodes_json: &str, edges_json: &str) -> Result<()> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    sqlx::query(
        "INSERT INTO datafaker_configs (name, nodes_json, edges_json, updated_at) \
         VALUES (?1, ?2, ?3, strftime('%s','now')) \
         ON CONFLICT(name) DO UPDATE SET nodes_json = excluded.nodes_json, edges_json = excluded.edges_json, updated_at = excluded.updated_at",
    )
    .bind(name)
    .bind(nodes_json)
    .bind(edges_json)
    .execute(&pool)
    .await
    .map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn load_datafaker_configs() -> Result<Vec<DatafakerConfigRecord>> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let rows = sqlx::query(
        "SELECT id, name, nodes_json, edges_json, updated_at FROM datafaker_configs ORDER BY updated_at DESC, id DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| err.to_string())?;
    rows.into_iter()
        .map(|row| {
            Ok(DatafakerConfigRecord {
                id: Some(row.try_get("id").map_err(|err| err.to_string())?),
                name: row.try_get("name").map_err(|err| err.to_string())?,
                nodes_json: row.try_get("nodes_json").map_err(|err| err.to_string())?,
                edges_json: row.try_get("edges_json").map_err(|err| err.to_string())?,
                updated_at: row.try_get("updated_at").map_err(|err| err.to_string())?,
            })
        })
        .collect()
}

pub async fn delete_datafaker_config(name: String) -> Result<bool> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let result = sqlx::query("DELETE FROM datafaker_configs WHERE name = ?1")
        .bind(name)
        .execute(&pool)
        .await
        .map_err(|err| err.to_string())?;
    Ok(result.rows_affected() > 0)
}

// ===== Excalidraw Doc CRUD =====

pub async fn save_excalidraw_doc(name: &str, elements_json: &str) -> Result<()> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    sqlx::query(
        "INSERT INTO excalidraw_docs (name, elements_json, updated_at) \
         VALUES (?1, ?2, strftime('%s','now')) \
         ON CONFLICT(name) DO UPDATE SET elements_json = excluded.elements_json, updated_at = excluded.updated_at",
    )
    .bind(name)
    .bind(elements_json)
    .execute(&pool)
    .await
    .map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn load_excalidraw_docs() -> Result<Vec<ExcalidrawDocRecord>> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let rows = sqlx::query(
        "SELECT id, name, elements_json, updated_at FROM excalidraw_docs ORDER BY updated_at DESC, id DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| err.to_string())?;
    rows.into_iter()
        .map(|row| {
            Ok(ExcalidrawDocRecord {
                id: Some(row.try_get("id").map_err(|err| err.to_string())?),
                name: row.try_get("name").map_err(|err| err.to_string())?,
                elements_json: row
                    .try_get("elements_json")
                    .map_err(|err| err.to_string())?,
                updated_at: row.try_get("updated_at").map_err(|err| err.to_string())?,
            })
        })
        .collect()
}

pub async fn delete_excalidraw_doc(name: String) -> Result<bool> {
    let pool = open_pool().await?;
    init_schema(&pool).await?;
    let result = sqlx::query("DELETE FROM excalidraw_docs WHERE name = ?1")
        .bind(name)
        .execute(&pool)
        .await
        .map_err(|err| err.to_string())?;
    Ok(result.rows_affected() > 0)
}

pub fn driver_label(driver: &Driver) -> String {
    match driver {
        Driver::Mysql => "MySQL",
        Driver::Sqlite => "SQLite",
        Driver::Postgres => "PostgreSQL",
    }
    .to_string()
}

async fn open_pool() -> Result<SqlitePool> {
    let path = config_path()?;
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
        .map_err(|err| err.to_string())?
        .create_if_missing(true)
        .foreign_keys(true);
    SqlitePool::connect_with(options)
        .await
        .map_err(|err| err.to_string())
}

async fn init_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS data_sources (\
            id INTEGER PRIMARY KEY AUTOINCREMENT,\
            name TEXT NOT NULL UNIQUE,\
            info TEXT NOT NULL,\
            updated_at INTEGER NOT NULL\
        )",
    )
    .execute(pool)
    .await
    .map_err(|err| err.to_string())?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS clipboard_history (\
            id INTEGER PRIMARY KEY AUTOINCREMENT,\
            content TEXT NOT NULL UNIQUE,\
            created_at INTEGER NOT NULL\
        )",
    )
    .execute(pool)
    .await
    .map_err(|err| err.to_string())?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS snippets (\
            id INTEGER PRIMARY KEY AUTOINCREMENT,\
            title TEXT NOT NULL,\
            code TEXT NOT NULL,\
            tags TEXT NOT NULL DEFAULT '[]',\
            language TEXT,\
            created_at INTEGER NOT NULL,\
            updated_at INTEGER NOT NULL\
        )",
    )
    .execute(pool)
    .await
    .map_err(|err| err.to_string())?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS todos (\
            id INTEGER PRIMARY KEY AUTOINCREMENT,\
            parent_id INTEGER,\
            content TEXT NOT NULL,\
            completed INTEGER NOT NULL DEFAULT 0,\
            created_at INTEGER NOT NULL,\
            FOREIGN KEY(parent_id) REFERENCES todos(id) ON DELETE CASCADE\
        )",
    )
    .execute(pool)
    .await
    .map_err(|err| err.to_string())?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS app_settings (\
            key TEXT PRIMARY KEY,\
            value TEXT NOT NULL\
        )",
    )
    .execute(pool)
    .await
    .map_err(|err| err.to_string())?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS datafaker_configs (\
            id INTEGER PRIMARY KEY AUTOINCREMENT,\
            name TEXT NOT NULL UNIQUE,\
            nodes_json TEXT NOT NULL,\
            edges_json TEXT NOT NULL,\
            updated_at INTEGER NOT NULL\
        )",
    )
    .execute(pool)
    .await
    .map_err(|err| err.to_string())?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS excalidraw_docs (\
            id INTEGER PRIMARY KEY AUTOINCREMENT,\
            name TEXT NOT NULL UNIQUE,\
            elements_json TEXT NOT NULL,\
            updated_at INTEGER NOT NULL\
        )",
    )
    .execute(pool)
    .await
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "无法获取系统配置目录".to_string())?;
    let app_dir = config_dir.join("toolbox");
    // 确保目录存在
    std::fs::create_dir_all(&app_dir)
        .map_err(|err| format!("创建配置目录失败：{err}"))?;
    Ok(app_dir.join("tool.db"))
}
