#![allow(unused)]
use serde::{Deserialize, Serialize};

mod mysql;
mod postgres;
mod sqlite;

mod template;

lazy_static::lazy_static! {
    pub static ref KEYWORDS: Vec<&'static str> = {
        // Rust1.70 关键字
        vec![
            "as", "async", "await","break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false",
            "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
            "ref", "return", "Self", "self", "static", "struct", "super", "trait", "true", "type","union",
            "unsafe", "use", "where", "while", "abstract",  "become", "box", "do",
             "final", "macro", "override", "priv", "try", "typeof", "unsized", "virtual",
            "yield",
        ]
    };
}

/// 判断字段名称是否是由多个单词组成
pub fn multi_world(name: &str) -> bool {
    name.contains(|c| ['_', '-'].contains(&c))
}

/// 列名是否为Rust关键字，若为关键字，则需要在其前加 r#
pub fn column_keywords(name: &str) -> String {
    if KEYWORDS.contains(&name) {
        format!("r#{name}")
    } else {
        name.to_string()
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Table {
    pub schema: String,
    pub name: String,
    pub comment: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Column {
    pub schema: Option<String>,
    pub table_name: Option<String>,
    pub name: Option<String>,
    pub default: Option<String>,
    pub max_length: Option<i64>,
    pub is_nullable: bool,
    pub column_type: Option<String>,
    pub comment: Option<String>,

    // 对应 Rust 类型
    pub field_type: String,
    // pub multi_world: Option<bool>,
}

/// 驱动类型
#[derive(Debug, Clone, Copy, Serialize)]
pub enum Driver {
    Mysql,
    Postgres,
    Sqlite,
}
