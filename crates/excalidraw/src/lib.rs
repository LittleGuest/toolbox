//! Excalidraw 白板内核。
//!
//! 从 `crates/ui` 抽离出来的独立库：
//!
//! - [`ExcalidrawView`] —— 可直接挂进 GPUI 元素树的画布视图（元素增删改、选择、
//!   平移缩放、撤销重做、样式面板、SVG / PNG 导出、文档存取）。
//! - [`DocStore`] / [`DocRecord`] —— 持久化边界，由引入方实现，内核不绑定存储实现。
//!
//! 依赖方向是单向的：`excalidraw` ← `ui`，内核不认识宿主，宿主通过
//! [`ExcalidrawView::new`] 注入存储实现。

mod store;
mod view;

pub use store::{DocRecord, DocStore, NoopDocStore};
pub use view::ExcalidrawView;
