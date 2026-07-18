//! 轻量级语法高亮工具，基于字符串词法分析为 JSON / SQL / XML 文本生成 HighlightStyle 区间。
//!
//! 不依赖 tree-sitter，适合在只读展示区域着色；可编辑输入区仍使用普通 Input。

use std::ops::Range;

use gpui::{HighlightStyle, Hsla, StyledText};

/// 语法高亮配色（与主题解耦，使用 HSLA 自定义色板，兼容深/浅色背景）
pub struct HighlightPalette {
    pub key: Hsla,
    pub string: Hsla,
    pub number: Hsla,
    pub boolean: Hsla,
    pub null: Hsla,
    pub punctuation: Hsla,
    pub keyword: Hsla,
    pub comment: Hsla,
    pub tag: Hsla,
    pub attr: Hsla,
}

impl HighlightPalette {
    /// 默认配色（紫色 key、绿色 string、橙色 number、红色 boolean）
    pub fn default_light() -> Self {
        Self {
            key: gpui::hsla(0.75, 0.55, 0.45, 1.0),
            string: gpui::hsla(0.33, 0.6, 0.4, 1.0),
            number: gpui::hsla(0.08, 0.7, 0.5, 1.0),
            boolean: gpui::hsla(0.0, 0.65, 0.5, 1.0),
            null: gpui::hsla(0.0, 0.0, 0.45, 1.0),
            punctuation: gpui::hsla(0.58, 0.0, 0.45, 1.0),
            keyword: gpui::hsla(0.58, 0.75, 0.45, 1.0),
            comment: gpui::hsla(0.33, 0.3, 0.45, 1.0),
            tag: gpui::hsla(0.0, 0.6, 0.5, 1.0),
            attr: gpui::hsla(0.75, 0.55, 0.45, 1.0),
        }
    }
}

/// 高亮区间条目，记录字节范围与对应配色
pub struct HighlightRange {
    pub range: Range<usize>,
    pub color: Hsla,
}

/// 构建 JSON 高亮区间列表
///
/// 支持：字符串值/键、数字、true/false/null、标点 { } [ ] : ,
pub fn json_highlights(text: &str, palette: &HighlightPalette) -> Vec<HighlightRange> {
    let mut ranges = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // 跳过空白
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }

        let start = i;

        match bytes[i] {
            b'"' => {
                // 字符串字面量
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if bytes[i] == b'"' {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                // 判断是 key 还是 string value：看后面紧跟的非空字符是否为 ':'
                let mut j = i;
                while j < len && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                let is_key = j < len && bytes[j] == b':';
                let color = if is_key { palette.key } else { palette.string };
                ranges.push(HighlightRange {
                    range: start..i,
                    color,
                });
            }
            b'{' | b'}' | b'[' | b']' | b':' | b',' => {
                i += 1;
                ranges.push(HighlightRange {
                    range: start..i,
                    color: palette.punctuation,
                });
            }
            b'-' | b'0'..=b'9' => {
                // 数字
                i += 1;
                while i < len
                    && (bytes[i].is_ascii_digit()
                        || matches!(bytes[i], b'.' | b'e' | b'E' | b'+' | b'-'))
                {
                    i += 1;
                }
                ranges.push(HighlightRange {
                    range: start..i,
                    color: palette.number,
                });
            }
            b't' | b'f' => {
                // true / false
                if text[i..].starts_with("true") {
                    i += 4;
                    ranges.push(HighlightRange {
                        range: start..i,
                        color: palette.boolean,
                    });
                } else if text[i..].starts_with("false") {
                    i += 5;
                    ranges.push(HighlightRange {
                        range: start..i,
                        color: palette.boolean,
                    });
                } else {
                    // 未识别标识符，按普通文本处理
                    i += 1;
                }
            }
            b'n' => {
                if text[i..].starts_with("null") {
                    i += 4;
                    ranges.push(HighlightRange {
                        range: start..i,
                        color: palette.null,
                    });
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    ranges
}

/// 构建 SQL 高亮区间列表
///
/// 关键字（大小写不敏感）着色，字符串字面量、数字、注释、标识符分别着色。
pub fn sql_highlights(text: &str, palette: &HighlightPalette) -> Vec<HighlightRange> {
    const KEYWORDS: &[&str] = &[
        "SELECT",
        "FROM",
        "WHERE",
        "AND",
        "OR",
        "JOIN",
        "LEFT",
        "RIGHT",
        "INNER",
        "OUTER",
        "ON",
        "GROUP",
        "BY",
        "ORDER",
        "HAVING",
        "LIMIT",
        "OFFSET",
        "INSERT",
        "INTO",
        "VALUES",
        "UPDATE",
        "SET",
        "DELETE",
        "CREATE",
        "TABLE",
        "DROP",
        "ALTER",
        "ADD",
        "COLUMN",
        "PRIMARY",
        "KEY",
        "FOREIGN",
        "REFERENCES",
        "UNIQUE",
        "INDEX",
        "AS",
        "DISTINCT",
        "UNION",
        "ALL",
        "EXISTS",
        "IN",
        "BETWEEN",
        "LIKE",
        "IS",
        "NULL",
        "NOT",
        "ASC",
        "DESC",
        "CASE",
        "WHEN",
        "THEN",
        "ELSE",
        "END",
        "WITH",
        "RECURSIVE",
        "DEFAULT",
        "CONSTRAINT",
        "CHECK",
        "CASCADE",
        "GRANT",
        "REVOKE",
        "BEGIN",
        "COMMIT",
        "ROLLBACK",
        "TRANSACTION",
    ];

    let mut ranges = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }

        let start = i;

        match bytes[i] {
            b'-' if i + 1 < len && bytes[i + 1] == b'-' => {
                // 行注释 --
                while i < len && bytes[i] != b'\n' {
                    i += 1;
                }
                ranges.push(HighlightRange {
                    range: start..i,
                    color: palette.comment,
                });
            }
            b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
                // 块注释 /* */
                i += 2;
                while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < len {
                    i += 2;
                }
                ranges.push(HighlightRange {
                    range: start..i,
                    color: palette.comment,
                });
            }
            b'\'' | b'"' | b'`' => {
                let quote = bytes[i];
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if bytes[i] == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                ranges.push(HighlightRange {
                    range: start..i,
                    color: palette.string,
                });
            }
            b'0'..=b'9' => {
                i += 1;
                while i < len
                    && (bytes[i].is_ascii_digit() || matches!(bytes[i], b'.' | b'e' | b'E'))
                {
                    i += 1;
                }
                ranges.push(HighlightRange {
                    range: start..i,
                    color: palette.number,
                });
            }
            b'(' | b')' | b',' | b';' | b'.' | b'*' | b'=' | b'<' | b'>' | b'!' | b'+' | b'-' => {
                i += 1;
                ranges.push(HighlightRange {
                    range: start..i,
                    color: palette.punctuation,
                });
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                // 标识符/关键字
                while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let upper = text[start..i].to_uppercase();
                if KEYWORDS.contains(&upper.as_str()) {
                    ranges.push(HighlightRange {
                        range: start..i,
                        color: palette.keyword,
                    });
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    ranges
}

/// 构建 XML 高亮区间列表
///
/// 标签名、属性名、字符串值、注释分别着色；标点 < > / = 着灰色。
pub fn xml_highlights(text: &str, palette: &HighlightPalette) -> Vec<HighlightRange> {
    let mut ranges = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }

        let start = i;

        if bytes[i] == b'<'
            && i + 3 < len
            && bytes[i + 1] == b'!'
            && bytes[i + 2] == b'-'
            && bytes[i + 3] == b'-'
        {
            // 注释 <!-- -->
            while i + 2 < len && !(bytes[i] == b'-' && bytes[i + 1] == b'-' && bytes[i + 2] == b'>')
            {
                i += 1;
            }
            if i + 2 < len {
                i += 3;
            }
            ranges.push(HighlightRange {
                range: start..i,
                color: palette.comment,
            });
            continue;
        }

        if bytes[i] == b'<' {
            // 标签开始
            // '<' 着标点色
            ranges.push(HighlightRange {
                range: i..i + 1,
                color: palette.punctuation,
            });
            i += 1;

            // 可选的 '/' 或 '?' 或 '!'
            if i < len && (bytes[i] == b'/' || bytes[i] == b'?' || bytes[i] == b'!') {
                ranges.push(HighlightRange {
                    range: i..i + 1,
                    color: palette.punctuation,
                });
                i += 1;
            }

            // 标签名
            let tag_start = i;
            while i < len
                && !bytes[i].is_ascii_whitespace()
                && !matches!(bytes[i], b'>' | b'/' | b'?' | b'=')
            {
                i += 1;
            }
            if i > tag_start {
                ranges.push(HighlightRange {
                    range: tag_start..i,
                    color: palette.tag,
                });
            }

            // 属性与值
            while i < len && bytes[i] != b'>' {
                if bytes[i].is_ascii_whitespace() {
                    i += 1;
                    continue;
                }

                if bytes[i] == b'/' || bytes[i] == b'?' {
                    ranges.push(HighlightRange {
                        range: i..i + 1,
                        color: palette.punctuation,
                    });
                    i += 1;
                    continue;
                }

                if bytes[i] == b'=' {
                    ranges.push(HighlightRange {
                        range: i..i + 1,
                        color: palette.punctuation,
                    });
                    i += 1;
                    continue;
                }

                if bytes[i] == b'"' || bytes[i] == b'\'' {
                    let quote = bytes[i];
                    let s = i;
                    i += 1;
                    while i < len && bytes[i] != quote {
                        i += 1;
                    }
                    if i < len {
                        i += 1;
                    }
                    ranges.push(HighlightRange {
                        range: s..i,
                        color: palette.string,
                    });
                    continue;
                }

                // 属性名
                let attr_start = i;
                while i < len
                    && !bytes[i].is_ascii_whitespace()
                    && !matches!(bytes[i], b'=' | b'>' | b'/' | b'"' | b'\'')
                {
                    i += 1;
                }
                if i > attr_start {
                    ranges.push(HighlightRange {
                        range: attr_start..i,
                        color: palette.attr,
                    });
                }
            }

            // '>' 着标点色
            if i < len && bytes[i] == b'>' {
                ranges.push(HighlightRange {
                    range: i..i + 1,
                    color: palette.punctuation,
                });
                i += 1;
            }
            continue;
        }

        // 文本节点（非标签内容）
        let text_start = i;
        while i < len && bytes[i] != b'<' {
            i += 1;
        }
        if i > text_start {
            // 普通文本不着色（默认前景色）
        }
    }

    ranges
}

/// 将 HighlightRange 列表转换为 GPUI 的 (Range<usize>, HighlightStyle) 元组列表
pub fn to_highlight_styles(
    _text: &str,
    ranges: &[HighlightRange],
) -> Vec<(Range<usize>, HighlightStyle)> {
    ranges
        .iter()
        .map(|r| {
            (
                r.range.clone(),
                HighlightStyle {
                    color: Some(r.color),
                    ..Default::default()
                },
            )
        })
        .collect()
}

/// 构建一个带高亮的 StyledText 元素
pub fn styled_text(text: &str, ranges: Vec<HighlightRange>) -> StyledText {
    let highlights = to_highlight_styles(text, &ranges);
    StyledText::new(text.to_string()).with_highlights(highlights)
}
