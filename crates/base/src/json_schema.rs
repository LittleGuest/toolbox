use anyhow::{Result, anyhow};
use jsonschema::{error::ValidationErrorKind, paths::LocationSegment};
use serde_json::Value;

#[derive(Debug)]
pub struct SchemaIssue {
    pub path: String,
    pub message: String,
}

#[derive(Debug)]
pub struct SchemaReport {
    pub valid: bool,
    pub total_errors: usize,
    pub issues: Vec<SchemaIssue>,
}

const DRAFT_OPTIONS: [&str; 6] = ["自动", "4", "6", "7", "2019", "2020"];

pub fn draft_options() -> &'static [&'static str] {
    &DRAFT_OPTIONS
}

pub fn validate(schema_text: &str, instance_text: &str, draft: &str) -> Result<SchemaReport> {
    let schema: Value = serde_json::from_str(schema_text).map_err(|e| {
        anyhow!(
            "JSON Schema 解析失败：第 {} 行，第 {} 列 — {}",
            e.line(),
            e.column(),
            e
        )
    })?;
    let instance: Value = serde_json::from_str(instance_text).map_err(|e| {
        anyhow!(
            "待校验 JSON 解析失败：第 {} 行，第 {} 列 — {}",
            e.line(),
            e.column(),
            e
        )
    })?;

    let validator = build_validator(&schema, draft)?;
    Ok(match validator.validate(&instance) {
        Ok(()) => SchemaReport {
            valid: true,
            total_errors: 0,
            issues: Vec::new(),
        },
        Err(_) => {
            let issues = collect_all(&validator, &instance);
            let issues = if issues.is_empty() {
                vec![SchemaIssue {
                    path: "$".to_string(),
                    message: "校验失败".to_string(),
                }]
            } else {
                issues
            };
            SchemaReport {
                valid: false,
                total_errors: issues.len(),
                issues,
            }
        }
    })
}

fn build_validator(schema: &Value, draft: &str) -> Result<jsonschema::Validator> {
    use jsonschema::Draft;
    let mut options = jsonschema::options();
    options = match draft {
        "4" => options.with_draft(Draft::Draft4),
        "6" => options.with_draft(Draft::Draft6),
        "7" => options.with_draft(Draft::Draft7),
        "2019" => options.with_draft(Draft::Draft201909),
        "2020" => options.with_draft(Draft::Draft202012),
        _ => options,
    };
    options
        .build(schema)
        .map_err(|e| anyhow!("JSON Schema 编译失败：{e}"))
}

fn collect_all(validator: &jsonschema::Validator, instance: &Value) -> Vec<SchemaIssue> {
    let mut out = Vec::new();
    for err in validator.iter_errors(instance) {
        flatten(&err, &mut out);
    }
    out
}

fn flatten(err: &jsonschema::ValidationError, out: &mut Vec<SchemaIssue>) {
    let branches = branch_groups(&err.kind);
    if let Some(groups) = branches {
        if let Some(first) = groups.iter().find(|g| !g.is_empty()) {
            for child in first.iter() {
                flatten(child, out);
            }
            return;
        }
    }
    out.push(SchemaIssue {
        path: display_path(err.instance_path.as_str()),
        message: translate(&err.masked().to_string()),
    });
}

fn branch_groups<'a>(
    kind: &'a ValidationErrorKind,
) -> Option<Vec<Vec<&'a jsonschema::ValidationError<'static>>>> {
    match kind {
        ValidationErrorKind::AnyOf { context } | ValidationErrorKind::OneOfNotValid { context } => {
            Some(context.iter().map(|g| g.iter().collect()).collect())
        }
        ValidationErrorKind::OneOfMultipleValid { context } => {
            Some(context.iter().map(|g| g.iter().collect()).collect())
        }
        _ => None,
    }
}

fn display_path(pointer: &str) -> String {
    if pointer.is_empty() {
        return "$".to_string();
    }
    let mut out = String::from("$");
    for seg in pointer.split('/').skip(1) {
        let decoded = seg.replace("~1", "/").replace("~0", "~");
        if decoded.chars().all(|c| c.is_ascii_digit()) && !decoded.is_empty() {
            out.push('[');
            out.push_str(&decoded);
            out.push(']');
        } else if decoded
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            && !decoded.is_empty()
        {
            out.push('.');
            out.push_str(&decoded);
        } else {
            out.push_str(&format!("[\"{decoded}\"]"));
        }
    }
    out
}

pub fn translate(message: &str) -> String {
    let m = message.trim();
    if m.ends_with("is not of type \"string\"") {
        return "类型不匹配，期望：字符串".to_string();
    }
    if let Some(idx) = m.find(" is not of type ") {
        let tail = &m[idx + " is not of type ".len()..];
        return format!("类型不匹配，期望：{}", friendly_types(tail));
    }
    if let Some(idx) = m.find(" is not allowed for ") {
        let tail = &m[idx + " is not allowed for ".len()..];
        return format!("取值不被允许（受 {tail} 约束）");
    }
    if m.contains("is not valid under any of the schemas listed in the 'anyOf' keyword") {
        return "不满足 anyOf 中的任何一个子模式".to_string();
    }
    if m.contains("is not valid under any of the schemas listed in the 'oneOf' keyword") {
        return "不满足 oneOf 中的任何一个子模式".to_string();
    }
    if m.contains("is valid under more than one of the schemas listed in the 'oneOf' keyword") {
        return "同时满足 oneOf 中多个子模式（要求恰好一个）".to_string();
    }
    if m.starts_with("Additional properties are not allowed") {
        return format!("存在未声明的额外属性：{}", strip_tail(m));
    }
    if m.starts_with("Additional items are not allowed") {
        return format!("存在多余的数组元素：{}", strip_tail(m));
    }
    if let Some(idx) = m.find(" is a required property") {
        return format!("缺少必填属性：{}", m[..idx].trim());
    }
    if m.starts_with("Unevaluated properties are not allowed") {
        return format!("存在未求值的属性：{}", strip_tail(m));
    }
    if m.contains("is not a \"") {
        if let Some(start) = m.find("is not a \"") {
            let fmt = m[start + 10..].trim_end_matches('"').trim();
            return format!("不满足格式要求：{fmt}");
        }
    }
    if let Some(idx) = m.find(" does not match \"") {
        let tail = &m[idx + " does not match ".len()..];
        return format!("不匹配正则模式 {tail}");
    }
    if m.contains("are not unique") || m.contains("has non-unique elements") {
        return "数组元素存在重复".to_string();
    }
    if let Some(idx) = m.find(" is not a multiple of ") {
        let tail = &m[idx + " is not a multiple of ".len()..];
        return format!("必须是 {tail} 的整数倍");
    }
    if let Some(idx) = m.find(" is less than the minimum of ") {
        let tail = &m[idx + " is less than the minimum of ".len()..];
        return format!("小于最小值 {tail}");
    }
    if let Some(idx) = m.find(" is greater than the maximum of ") {
        let tail = &m[idx + " is greater than the maximum of ".len()..];
        return format!("大于最大值 {tail}");
    }
    if let Some(idx) = m.find(" is less than or equal to the exclusive minimum of ") {
        let tail = &m[idx + " is less than or equal to the exclusive minimum of ".len()..];
        return format!("必须严格大于最小值 {tail}");
    }
    if let Some(idx) = m.find(" is greater than or equal to the exclusive maximum of ") {
        let tail = &m[idx + " is greater than or equal to the exclusive maximum of ".len()..];
        return format!("必须严格小于最大值 {tail}");
    }
    if let Some(idx) = m.find(" is longer than ") {
        let tail = &m[idx + " is longer than ".len()..];
        let n = tail.split_whitespace().next().unwrap_or("");
        return format!("长度超出限制，最多 {n}");
    }
    if let Some(idx) = m.find(" is shorter than ") {
        let tail = &m[idx + " is shorter than ".len()..];
        let n = tail.split_whitespace().next().unwrap_or("");
        return format!("长度不足，最少 {n}");
    }
    if let Some(idx) = m.find(" has less than ") {
        let tail = &m[idx + " has less than ".len()..];
        let n = tail.split_whitespace().next().unwrap_or("");
        return format!("数量 / 长度不得小于 {n}");
    }
    if let Some(idx) = m.find(" has more than ") {
        let tail = &m[idx + " has more than ".len()..];
        let n = tail.split_whitespace().next().unwrap_or("");
        return format!("数量 / 长度不得大于 {n}");
    }
    if m.contains("None of") && m.contains("are valid under the given schema") {
        return "没有任何元素满足 contains 约束".to_string();
    }
    if m.is_empty() {
        return "校验失败".to_string();
    }
    m.to_string()
}

fn strip_tail(m: &str) -> String {
    match (m.find('('), m.rfind(')')) {
        (Some(a), Some(b)) if b > a => {
            let mut inner = m[a + 1..b].to_string();
            if let Some(p) = inner.rfind(" (") {
                let tail = &inner[p..];
                if tail.contains("was") || tail.contains("were") {
                    inner.truncate(p);
                }
            }
            inner
        }
        _ => String::new(),
    }
}

fn friendly_types(raw: &str) -> String {
    raw.split(',')
        .map(|part| {
            let t = part.trim().trim_matches('"').trim();
            match t {
                "string" => "字符串".to_string(),
                "integer" => "整数".to_string(),
                "number" => "数字".to_string(),
                "object" => "对象".to_string(),
                "array" => "数组".to_string(),
                "boolean" => "布尔值".to_string(),
                "null" => "空值".to_string(),
                other => other.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

pub fn format_report_text(report: &SchemaReport) -> String {
    if report.valid {
        return "校验通过：实例完全符合 JSON Schema。".to_string();
    }
    let mut out = format!("校验未通过，共 {} 处问题：\n", report.total_errors);
    for (i, issue) in report.issues.iter().enumerate() {
        out.push_str(&format!("{}. {} → {}\n", i + 1, issue.path, issue.message));
    }
    out
}

pub fn paths_of(instance_text: &str) -> Vec<String> {
    let value: Value = match serde_json::from_str(instance_text) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    walk(&value, "$", &mut out);
    out
}

fn walk(value: &Value, cur: &str, out: &mut Vec<String>) {
    out.push(cur.to_string());
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let next = format!("{cur}{}", key_segment(k));
                walk(v, &next, out);
            }
        }
        Value::Array(items) => {
            for (i, v) in items.iter().enumerate() {
                walk(v, &format!("{cur}[{i}]"), out);
            }
        }
        _ => {}
    }
}

fn key_segment(name: &str) -> String {
    if !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        format!(".{name}")
    } else {
        format!("[\"{name}\"]")
    }
}

pub fn pointer_segments(path: &str) -> Vec<LocationSegment<'static>> {
    let mut out = Vec::new();
    let mut rest = path.trim_start_matches('$');
    loop {
        if rest.is_empty() {
            break;
        }
        if let Some(stripped) = rest.strip_prefix('.') {
            let end = stripped.find(['.', '[']).unwrap_or(stripped.len());
            let key = stripped[..end].to_string();
            rest = &stripped[end..];
            out.push(LocationSegment::Property(std::borrow::Cow::Owned(key)));
        } else if let Some(stripped) = rest.strip_prefix("[\"") {
            match stripped.find("\"]") {
                Some(end) => {
                    let key = stripped[..end].to_string();
                    rest = &stripped[end + 2..];
                    out.push(LocationSegment::Property(std::borrow::Cow::Owned(key)));
                }
                None => break,
            }
        } else if let Some(stripped) = rest.strip_prefix('[') {
            match stripped.find(']') {
                Some(end) => {
                    if let Ok(idx) = stripped[..end].parse::<usize>() {
                        out.push(LocationSegment::Index(idx));
                    }
                    rest = &stripped[end + 1..];
                }
                None => break,
            }
        } else {
            break;
        }
    }
    out
}

pub fn schema_template() -> &'static str {
    r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "用户信息",
  "type": "object",
  "required": ["id", "name", "email"],
  "properties": {
    "id": { "type": "integer", "minimum": 1 },
    "name": { "type": "string", "minLength": 1, "maxLength": 32 },
    "email": { "type": "string", "format": "email" },
    "age": { "type": "integer", "minimum": 0, "maximum": 150 },
    "tags": {
      "type": "array",
      "items": { "type": "string" },
      "uniqueItems": true,
      "maxItems": 6
    }
  },
  "additionalProperties": false
}"#
}

pub fn instance_template() -> &'static str {
    r#"{
  "id": 1,
  "name": "张三",
  "email": "zhangsan@example.com",
  "age": 28,
  "tags": ["后端", "Rust"]
}"#
}

pub fn draft_of(schema_text: &str) -> String {
    let value: Value = match serde_json::from_str(schema_text) {
        Ok(v) => v,
        Err(_) => return "自动".to_string(),
    };
    let declared = value
        .get("$schema")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let draft = if declared.contains("draft-04") {
        "4"
    } else if declared.contains("draft-06") {
        "6"
    } else if declared.contains("draft-07") {
        "7"
    } else if declared.contains("2019-09") {
        "2019"
    } else if declared.contains("2020-12") {
        "2020"
    } else {
        "自动"
    };
    draft.to_string()
}

pub fn pretty_json(text: &str) -> Result<String> {
    let value: Value = serde_json::from_str(text)
        .map_err(|e| anyhow!("JSON 解析失败：第 {} 行 — {}", e.line(), e))?;
    serde_json::to_string_pretty(&value).map_err(|e| anyhow!("序列化失败：{e}"))
}
