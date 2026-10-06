use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct DocRecord {
    pub id: Option<i64>,

    pub name: String,

    pub elements_json: String,

    pub updated_at: i64,
}

#[async_trait]
pub trait DocStore: Send + Sync + 'static {
    async fn save(&self, name: &str, elements_json: &str) -> Result<(), String>;

    async fn load(&self) -> Result<Vec<DocRecord>, String>;

    async fn delete(&self, name: &str) -> Result<bool, String>;
}

#[derive(Default)]
pub struct NoopDocStore;

#[async_trait]
impl DocStore for NoopDocStore {
    async fn save(&self, _name: &str, _elements_json: &str) -> Result<(), String> {
        Ok(())
    }

    async fn load(&self) -> Result<Vec<DocRecord>, String> {
        Ok(Vec::new())
    }

    async fn delete(&self, _name: &str) -> Result<bool, String> {
        Ok(false)
    }
}
