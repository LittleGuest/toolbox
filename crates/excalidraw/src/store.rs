//! 文档持久化边界。
//!
//! 白板内核不认识 SQLite、也不认识宿主的配置库：它只依赖 [`DocStore`] 这个抽象，
//! 由引入方（`crates/ui`）实现并注入。这样内核可以独立编译、独立测试，
//! 换存储（文件 / 内存 / 远端）也不需要动内核一行代码。

use async_trait::async_trait;

/// 一份已保存的文档。
///
/// 与存储层解耦的中立表示 —— 字段对齐 `ui` 侧 SQLite 表的列，
/// 但这里不依赖任何具体驱动类型。
#[derive(Clone, Debug)]
pub struct DocRecord {
    /// 存储层主键；尚未落库时为 `None`。
    pub id: Option<i64>,
    /// 文档名（唯一键）。
    pub name: String,
    /// 元素数组的 JSON，由内核自己序列化 / 反序列化。
    pub elements_json: String,
    /// 最后更新时间（秒级 Unix 时间戳）。
    pub updated_at: i64,
}

/// 文档存储。实现方需保证三个方法可并发调用（内核在 GPUI 的异步任务里 await 它们）。
#[async_trait]
pub trait DocStore: Send + Sync + 'static {
    /// 按名字 upsert 一份文档。
    async fn save(&self, name: &str, elements_json: &str) -> Result<(), String>;

    /// 拉取全部文档，按更新时间倒序。
    async fn load(&self) -> Result<Vec<DocRecord>, String>;

    /// 按名字删除，返回是否真的删掉了。
    async fn delete(&self, name: &str) -> Result<bool, String>;
}

/// 什么都不做的存储：用于测试与「不持久化」场景。
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
