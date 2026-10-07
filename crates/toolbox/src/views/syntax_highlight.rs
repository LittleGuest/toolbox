use std::ops::Range;

use gpui_kit::{HighlightStyle, Hsla, StyledText};

#[allow(dead_code)]
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
    pub fn default_light() -> Self {
        Self {
            key: gpui_kit::hsla(0.75, 0.55, 0.45, 1.0),
            string: gpui_kit::hsla(0.33, 0.6, 0.4, 1.0),
            number: gpui_kit::hsla(0.08, 0.7, 0.5, 1.0),
            boolean: gpui_kit::hsla(0.0, 0.65, 0.5, 1.0),
            null: gpui_kit::hsla(0.0, 0.0, 0.45, 1.0),
            punctuation: gpui_kit::hsla(0.58, 0.0, 0.45, 1.0),
            keyword: gpui_kit::hsla(0.58, 0.75, 0.45, 1.0),
            comment: gpui_kit::hsla(0.33, 0.3, 0.45, 1.0),
            tag: gpui_kit::hsla(0.0, 0.6, 0.5, 1.0),
            attr: gpui_kit::hsla(0.75, 0.55, 0.45, 1.0),
        }
    }
}

pub struct HighlightRange {
    pub range: Range<usize>,
    pub color: Hsla,
}

pub fn json_highlights(text: &str, palette: &HighlightPalette) -> Vec<HighlightRange> {
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
            b'"' => {
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
                    i += 1;
                }
            }
            b'n' if text[i..].starts_with("null") => {
                i += 4;
                ranges.push(HighlightRange {
                    range: start..i,
                    color: palette.null,
                });
            }
            _ => {
                i += 1;
            }
        }
    }

    ranges
}

#[allow(dead_code)]
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
                while i < len && bytes[i] != b'\n' {
                    i += 1;
                }
                ranges.push(HighlightRange {
                    range: start..i,
                    color: palette.comment,
                });
            }
            b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
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

#[allow(dead_code)]
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
            ranges.push(HighlightRange {
                range: i..i + 1,
                color: palette.punctuation,
            });
            i += 1;

            if i < len && (bytes[i] == b'/' || bytes[i] == b'?' || bytes[i] == b'!') {
                ranges.push(HighlightRange {
                    range: i..i + 1,
                    color: palette.punctuation,
                });
                i += 1;
            }

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

            if i < len && bytes[i] == b'>' {
                ranges.push(HighlightRange {
                    range: i..i + 1,
                    color: palette.punctuation,
                });
                i += 1;
            }
            continue;
        }

        let text_start = i;
        while i < len && bytes[i] != b'<' {
            i += 1;
        }
        if i > text_start {}
    }

    ranges
}

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

pub fn styled_text(text: &str, ranges: Vec<HighlightRange>) -> StyledText {
    let highlights = to_highlight_styles(text, &ranges);
    StyledText::new(text.to_string()).with_highlights(highlights)
}
