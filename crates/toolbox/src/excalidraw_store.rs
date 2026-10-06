use std::sync::Arc;

use async_trait::async_trait;
use excalidraw::{DocRecord, DocStore};

use crate::config_store;

pub struct SqliteDocStore;

#[async_trait]
impl DocStore for SqliteDocStore {
    async fn save(&self, name: &str, elements_json: &str) -> Result<(), String> {
        config_store::save_excalidraw_doc(name, elements_json).await
    }

    async fn load(&self) -> Result<Vec<DocRecord>, String> {
        let docs = config_store::load_excalidraw_docs().await?;
        Ok(docs
            .into_iter()
            .map(|doc| DocRecord {
                id: doc.id,
                name: doc.name,
                elements_json: doc.elements_json,
                updated_at: doc.updated_at,
            })
            .collect())
    }

    async fn delete(&self, name: &str) -> Result<bool, String> {
        config_store::delete_excalidraw_doc(name.to_string()).await
    }
}

pub fn store() -> Arc<SqliteDocStore> {
    Arc::new(SqliteDocStore)
}
