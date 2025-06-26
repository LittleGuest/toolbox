mod generator;
// mod mysql;
// mod postgres;
// mod sqlite;

lazy_static::lazy_static! {
    pub static ref KEYWORDS: Vec<&'static str> = {
        // Rust1.85 关键字
        vec![
            "as", "async", "await","break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false",
            "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
            "ref", "return", "Self", "self", "static", "struct", "super", "trait", "true", "type","union",
            "unsafe", "use", "where", "while", "abstract",  "become", "box", "do",
             "final","gen", "macro", "override", "priv", "try", "typeof", "unsized", "virtual",
            "yield",
        ]
    };
}

/// 判断字段名称是否是由多个单词组成
fn multi_world(name: &str) -> bool {
    name.contains(|c| ['_', '-'].contains(&c))
}

/// 列名是否为Rust关键字，若为关键字，则需要在其前加 r#
fn column_keywords(name: &str) -> String {
    if KEYWORDS.contains(&name) {
        format!("r#{name}")
    } else {
        name.into()
    }
}
