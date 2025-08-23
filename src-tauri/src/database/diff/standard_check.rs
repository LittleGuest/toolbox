use std::{collections::HashMap, sync::LazyLock};

use dashmap::{DashMap, DashSet};
use database::{
    ColumnType,
    error::{Error, Result},
};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::database::{DatasourceInfo, diff::IndexBo};

static MYSQL_RESERVED_KEY_WORDS: [&str; 235] = [
    "accessible",
    "add",
    "all",
    "alter",
    "analyze",
    "and",
    "as",
    "asc",
    "asensitive",
    "before",
    "between",
    "bigint",
    "binary",
    "blob",
    "both",
    "by",
    "call",
    "cascade",
    "case",
    "change",
    "char",
    "character",
    "check",
    "collate",
    "column",
    "condition",
    "constraint",
    "continue",
    "convert",
    "create",
    "cross",
    "current_date",
    "current_time",
    "current_timestamp",
    "current_user",
    "cursor",
    "database",
    "databases",
    "day_hour",
    "day_microsecond",
    "day_minute",
    "day_second",
    "dec",
    "decimal",
    "declare",
    "default",
    "delayed",
    "delete",
    "desc",
    "describe",
    "deterministic",
    "distinct",
    "distinctrow",
    "div",
    "double",
    "drop",
    "dual",
    "each",
    "else",
    "elseif",
    "enclosed",
    "escaped",
    "exists",
    "exit",
    "explain",
    "false",
    "fetch",
    "float",
    "float4",
    "float8",
    "for",
    "force",
    "foreign",
    "from",
    "fulltext",
    "generated",
    "get",
    "grant",
    "group",
    "having",
    "high_priority",
    "hour_microsecond",
    "hour_minute",
    "hour_second",
    "if",
    "ignore",
    "in",
    "index",
    "infile",
    "inner",
    "inout",
    "insensitive",
    "insert",
    "int",
    "int1",
    "int2",
    "int3",
    "int4",
    "int8",
    "integer",
    "interval",
    "into",
    "io_after_gtids",
    "io_before_gtids",
    "is",
    "iterate",
    "join",
    "key",
    "keys",
    "kill",
    "leading",
    "leave",
    "left",
    "like",
    "limit",
    "linear",
    "lines",
    "load",
    "localtime",
    "localtimestamp",
    "lock",
    "long",
    "longblob",
    "longtext",
    "loop",
    "low_priority",
    "master_bind",
    "master_ssl_verify_server_cert",
    "match",
    "maxvalue",
    "mediumblob",
    "mediumint",
    "mediumtext",
    "middleint",
    "minute_microsecond",
    "minute_second",
    "mod",
    "modifies",
    "natural",
    "not",
    "no_write_to_binlog",
    "null",
    "numeric",
    "on",
    "optimize",
    "optimizer_costs",
    "option",
    "optionally",
    "or",
    "order",
    "out",
    "outer",
    "outfile",
    "partition",
    "precision",
    "primary",
    "procedure",
    "purge",
    "range",
    "read",
    "reads",
    "read_write",
    "real",
    "references",
    "regexp",
    "release",
    "rename",
    "repeat",
    "replace",
    "require",
    "resignal",
    "restrict",
    "return",
    "revoke",
    "right",
    "rlike",
    "schema",
    "schemas",
    "second_microsecond",
    "select",
    "sensitive",
    "separator",
    "set",
    "show",
    "signal",
    "smallint",
    "spatial",
    "specific",
    "sql",
    "sqlexception",
    "sqlstate",
    "sqlwarning",
    "sql_big_result",
    "sql_calc_found_rows",
    "sql_small_result",
    "ssl",
    "starting",
    "stored",
    "straight_join",
    "table",
    "terminated",
    "then",
    "tinyblob",
    "tinyint",
    "tinytext",
    "to",
    "trailing",
    "trigger",
    "true",
    "undo",
    "union",
    "unique",
    "unlock",
    "unsigned",
    "update",
    "usage",
    "use",
    "using",
    "utc_date",
    "utc_time",
    "utc_timestamp",
    "values",
    "varbinary",
    "varchar",
    "varcharacter",
    "varying",
    "virtual",
    "when",
    "where",
    "while",
    "with",
    "write",
    "xor",
    "year_month",
    "zerofill",
];

/// 规范检查宏，定义 StandardCheck struct
macro_rules! standard_check {
    ($($name:ident = $code:expr, $desc:expr),*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
        pub enum StandardCheck {
            $($name = $code),*
        }

        // FIXME: 怎样动态 format desc
        //
        // macro_rules! format_vec_items {
        //     ($fmt:expr, $vec:expr) => {{
        //         match $vec.len(){
        //             0=> format!($fmt),
        //             1=> format!($fmt,$vec[0]),
        //             2=> format!($fmt,$vec[0],$vec[1]),
        //             3=> format!($fmt,$vec[0],$vec[1], $vec[2]),
        //             4=> format!($fmt,$vec[0],$vec[1], $vec[2], $vec[3]),
        //             5=> format!($fmt,$vec[0],$vec[1], $vec[2], $vec[3], $vec[4]),
        //             _=> format!("{} {}", $fmt, $vec.join(","))
        //         }
        //     }};
        // }

        // macro_rules! format_desc {
        //     ($fmt:expr, $arg:expr) => {
        //         format!($fmt, $arg);
        //     };
        // }

        impl StandardCheck {
            pub fn code(&self) -> i32 {
                *self as i32
            }

            pub fn desc(&self) -> &str {
                match self {
                    $(StandardCheck::$name => $desc),*
                }
            }

            // pub fn format(&self, args: Vec<String>) -> String {
            //     match self {
            //         $(StandardCheck::$name => format_vec_items!($desc,args)),*
            //     }
            // }

            // pub fn format(&self, args: Vec<String>) -> String {
            //     match self {
            //         $(StandardCheck::$name => format_desc!($desc,args.into_iter()
            // .map(|s| format!("\"{s}\""))
            // .collect::<Vec<_>>()
            // .join(","))),*
            //     }
            // }

            pub fn codes() -> Vec<HashMap<String, String>> {
                let checks = vec![
                    $(StandardCheck::$name),*
                ];
                let len = checks.len();
                checks
                    .into_iter()
                    .fold(Vec::with_capacity(len), |mut vec, c| {
                        let mut map = HashMap::with_capacity(2);
                        map.insert("code".into(), c.code().to_string());
                        map.insert("desc".into(), c.desc().into());
                        vec.push(map);
                        vec
                    })
            }

        }
    };
}

standard_check! {
    // 检查小写
    NameContainUpperCase = 11, "应当使用小写",
    // 检查单词拼写
    NameErrorSpell = 12, "单词拼写建议，原词：{}，建议词：{}",
    // 检查数字开头
    NameDigitStart = 13, "不能数字开头，原词：{}",
    // 检查保留字
    NameUseKeyword = 14, "禁用保留字",
    // 检查复数单词
    TableNameContainPlurality = 31, "表名不要使用复数名词，原词：{}，建议词：{}",
    // 检查必备字段
    TableMissField = 32, "表缺少必备三字段：id, created_at, updated_at。",

    // 检查索引命名。主键索引名为 pk_字段名、唯一索引名为 uk_字段名、普通索引名则为 idx_字段名
    IndexNameError = 33, "索引命名不规范，原索引名：{}",

    // 检查is开头
    FieldIsStartErrorType = 41, "字段是is开头，但类型不是unsigned tinyint",
    // 检查is开头
    FieldIsStartErrorComment = 42, "字段是is开头，但字段备注没有包含“是否”二字",
    // 检查“是否”类型的字段
    FieldIsContainComment = 43, "字段备注包含“是否”二字，但字段名称不是is开头",
    // 检查小数类型
    FieldTypeUseFloat = 44, "小数类型为 decimal，禁止使用 float 和 double"
}

impl StandardCheck {
    pub fn format_desc(&self, args: &[String]) -> String {
        match self {
            StandardCheck::NameErrorSpell => {
                format!("单词拼写建议，原词：{}，建议词：{}", args[0], args[1])
            }
            StandardCheck::NameDigitStart => {
                format!("不能数字开头，原词：{}", args[0])
            }
            StandardCheck::TableNameContainPlurality => {
                format!(
                    "表名不要使用复数名词，原词：{}，建议词：{}",
                    args[0], args[1]
                )
            }
            StandardCheck::IndexNameError => {
                format!("索引命名不规范，原索引名：{}", args[0])
            }
            _ => self.desc().into(),
        }
    }
}

/// 规范检查
pub async fn standard_check(
    source: DatasourceInfo,
    check_codes: Vec<i32>,
) -> Result<Vec<CheckReportBo>> {
    if check_codes.is_empty() {
        return Err(Error::E("check item is empty"));
    }
    let mut source_ts = super::table_struct(&source).await?;
    let mut map = DashMap::new();
    let mut words = DashMap::new();
    let spelling_check = check_codes.contains(&StandardCheck::NameErrorSpell.code());

    for (sname, st) in source_ts.iter_mut() {
        check_word(sname, &check_codes, sname, &mut map).await?;
        if spelling_check {
            collect_word(sname, sname, &mut words).await?;
        }

        // 表名不要使用复数名词
        if check_codes.contains(&StandardCheck::TableNameContainPlurality.code()) {
            if sname.contains('_') {
                for name in sname.split('_') {
                    check_plural_word(name, sname, &mut map).await?;
                }
            } else {
                check_plural_word(sname, sname, &mut map).await?;
            }
        }

        // 索引命名
        if check_codes.contains(&StandardCheck::IndexNameError.code()) {
            check_index(&st.indexs, sname, &mut map).await?;
        }

        let mut cnames = vec![];
        for (cname, sc) in st.columns.iter() {
            let key = format!("{sname}#{cname}");
            cnames.push(cname);
            check_word(cname, &check_codes, &key, &mut map).await?;
            if spelling_check {
                collect_word(&key, cname, &mut words).await?;
            }

            // 字段是is开头，但类型不是unsigned tinyint
            if check_codes.contains(&StandardCheck::FieldIsStartErrorType.code())
                && cname.starts_with("is_")
                && !ColumnType::TinyInt.eq(&sc.r#type.unwrap())
            {
                add_to_map(&mut map, &key, StandardCheck::FieldIsStartErrorType, vec![]);
            }

            // 字段是is开头，但字段备注没有包含“是否”二字
            if check_codes.contains(&StandardCheck::FieldIsStartErrorComment.code())
                && cname.starts_with("is_")
                && !sc.comment.contains("是否")
            {
                add_to_map(
                    &mut map,
                    &key,
                    StandardCheck::FieldIsStartErrorComment,
                    vec![],
                );
            }

            // 字段备注包含“是否”二字，但字段名称不是is开头
            if check_codes.contains(&StandardCheck::FieldIsContainComment.code())
                && !cname.starts_with("is_")
                && sc.comment.contains("是否")
            {
                add_to_map(&mut map, &key, StandardCheck::FieldIsContainComment, vec![]);
            }

            // 小数类型为 decimal
            if check_codes.contains(&StandardCheck::FieldTypeUseFloat.code())
                && (ColumnType::Float.eq(&sc.r#type.unwrap())
                    || ColumnType::Double.eq(&sc.r#type.unwrap()))
            {
                add_to_map(&mut map, &key, StandardCheck::FieldTypeUseFloat, vec![]);
            }
        }
        // 表缺少必备三字段
        if check_codes.contains(&StandardCheck::TableMissField.code())
            && cnames
                .iter()
                .any(|&n| !(n.eq("id") && n.eq("created_at") && n.eq("updated_at")))
        {
            add_to_map(&mut map, sname, StandardCheck::TableMissField, vec![]);
        }
    }

    if spelling_check {
        check_spelling(words, &mut map).await?;
    }

    check_report(map).await
}

async fn check_report(map: DashMap<String, Vec<Suggest>>) -> Result<Vec<CheckReportBo>> {
    let mut check_report_map: HashMap<String, CheckReportBo> = HashMap::new();
    for m in map.iter() {
        let key = m.key();
        let suggests = m.value();

        let mut is_table = true;
        let mut table_name = key.as_str();
        let mut column_name = "";

        if key.contains('#') {
            is_table = false;
            let mut sp = key.split('#');
            table_name = sp.next().unwrap();
            column_name = sp.next().unwrap();
        }

        if let Some(cr) = check_report_map.get_mut(table_name) {
            if is_table {
                cr.suggests = suggests.clone();
            } else {
                let child = CheckReportBo {
                    name: column_name.to_string(),
                    suggests: suggests.clone(),
                    ..Default::default()
                };
                cr.children.push(child);
            }
        } else {
            let mut report = CheckReportBo {
                name: table_name.to_string(),
                ..Default::default()
            };
            if is_table {
                report.suggests = suggests.clone();
            } else {
                let child = CheckReportBo {
                    name: column_name.to_string(),
                    suggests: suggests.clone(),
                    ..Default::default()
                };
                report.children.push(child);
            }
            check_report_map.insert(table_name.to_string(), report);
        }
    }
    Ok(check_report_map.into_values().collect::<Vec<_>>())
}

static REG_UPPER_CASE: LazyLock<Regex> = LazyLock::new(|| Regex::new(".*[A-Z]+.*").unwrap());
static REG_START_WITH_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[0-9].*").unwrap());
static REG_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new("[\\d]").unwrap());

/// 检查名称
async fn check_word(
    word: &str,
    check_codes: &[i32],
    key: &str,
    map: &mut DashMap<String, Vec<Suggest>>,
) -> Result<()> {
    // 检查小写
    if check_codes.contains(&StandardCheck::NameContainUpperCase.code())
        && REG_UPPER_CASE.is_match(word)
    {
        add_to_map(map, key, StandardCheck::NameContainUpperCase, vec![]);
    }
    // 不能以数字开头
    if check_codes.contains(&StandardCheck::NameDigitStart.code())
        && REG_START_WITH_NUMBER.is_match(word)
    {
        add_to_map(map, key, StandardCheck::NameDigitStart, vec![word.into()]);
    }
    // 禁用关键字
    if check_codes.contains(&StandardCheck::NameUseKeyword.code())
        && MYSQL_RESERVED_KEY_WORDS.contains(&word.to_lowercase().as_str())
    {
        add_to_map(map, key, StandardCheck::NameUseKeyword, vec![]);
    }
    Ok(())
}

async fn collect_word(
    key: &str,
    name: &str,
    map: &mut DashMap<String, DashSet<String>>,
) -> Result<()> {
    let add_to_map = |name: &str, map: &mut DashMap<String, DashSet<String>>| {
        if let Some(ws) = map.get_mut(key) {
            ws.insert(name.into());
        } else {
            let ws = DashSet::new();
            ws.insert(name.to_lowercase());
            map.insert(key.into(), ws);
        }
    };
    if name.contains('_') {
        name.to_lowercase().split('_').for_each(|w| {
            add_to_map(w, map);
        })
    } else {
        add_to_map(name, map);
    }
    Ok(())
}

/// 单词单数形式缓存
static SINGULARIZE_MAP: LazyLock<DashMap<String, String>> = LazyLock::new(DashMap::new);
/// 忽略拼写检查的单词
static IGNORE_SPELLING_WORD: LazyLock<Vec<String>> = LazyLock::new(Vec::new);
/// 拼写检查缓存
static SPELLING_MAP: LazyLock<DashMap<String, Vec<String>>> = LazyLock::new(DashMap::new);

/// 检查单词的单复数
async fn check_plural_word(
    word: &str,
    key: &str,
    map: &mut DashMap<String, Vec<Suggest>>,
) -> Result<()> {
    if let Some(pw) = SINGULARIZE_MAP.get(word) {
        if !pw.eq(word) {
            add_to_map(
                map,
                key,
                StandardCheck::TableNameContainPlurality,
                vec![word.into(), pw.clone()],
            );
        }
    } else {
        let mut result = word.to_string();
        let plural = inflector::string::pluralize::to_plural(word);
        if plural.eq(word) {
            result = inflector::string::singularize::to_singular(word);
        }
        SINGULARIZE_MAP.insert(word.into(), result.clone());
        if !word.eq(&result) {
            add_to_map(
                map,
                key,
                StandardCheck::TableNameContainPlurality,
                vec![word.into(), result],
            );
        }
    }

    Ok(())
}

/// 检查索引名称
async fn check_index(
    indexs: &HashMap<String, IndexBo>,
    key: &str,
    map: &mut DashMap<String, Vec<Suggest>>,
) -> Result<()> {
    for (iname, index) in indexs
        .iter()
        .filter(|(inm, _)| !inm.eq_ignore_ascii_case("PRIMARY"))
    {
        if (index.non_unique == 0 && !iname.contains("uk_")) || !iname.contains("idx_") {
            add_to_map(map, key, StandardCheck::IndexNameError, vec![iname.clone()]);
        }
    }
    Ok(())
}

/// TODO: 单词拼写检查
async fn check_spelling(
    word: DashMap<String, DashSet<String>>,
    map: &mut DashMap<String, Vec<Suggest>>,
) -> Result<()> {
    // let mut temp_map = HashMap::new();
    // for (key, ws) in word.into_iter().filter(|(_, ws)| !ws.is_empty()) {
    //     let mut symspell: SymSpell<AsciiStringStrategy> = SymSpell::default();
    //     symspell.load_dictionary("./database/diff/word_checker", 0, 1, ",");
    //     for w in ws.iter().filter(|w| !IGNORE_SPELLING_WORD.contains(w)) {
    //         let suggests = SPELLING_MAP.get_mut(w.key());
    //         let mut suggestions = vec![];
    //
    //         if suggests.is_none_or(|s| s.value().is_empty()) {
    //             let w2 = REG_NUMBER.replace_all(w.key(), "");
    //             suggestions = if w2.len() > 1 {
    //                 let suggestions = symspell.lookup(&w2, Verbosity::Top, 2);
    //                 suggestions
    //                     .iter()
    //                     .map(|s| s.term.clone())
    //                     .collect::<Vec<_>>()
    //             } else {
    //                 vec![w.key().into()]
    //             };
    //         }
    //         SPELLING_MAP.entry(w.key().into()).insert(suggestions);
    //         let suggests = SPELLING_MAP.get(w.key());
    //         if let Some(sgs) = suggests {
    //             if !sgs.value().is_empty() {
    //                 let args = vec![w.key().into()];
    //                 let args = args
    //                     .into_iter()
    //                     .chain(sgs.value().clone())
    //                     .collect::<Vec<_>>();
    //                 add_to_map(map, &key, StandardCheck::NameErrorSpell, args);
    //             }
    //         }
    //         // SPELLING_MAP.entry(w.key().into()).insert(sgs);
    //     }
    // }
    Ok(())
}

fn add_to_map(
    map: &mut DashMap<String, Vec<Suggest>>,
    key: &str,
    check: StandardCheck,
    args: Vec<String>,
) {
    let desc = check.format_desc(&args);
    let suggest = if StandardCheck::NameErrorSpell.eq(&check) {
        Suggest::new_with_word(
            check.code(),
            desc,
            args.first().cloned().unwrap_or_default(),
        )
    } else {
        Suggest::new(check.code(), desc)
    };
    if map.contains_key(key) {
        if let Some(mut sts) = map.get_mut(key) {
            sts.push(suggest);
        }
    } else {
        map.insert(key.into(), vec![suggest]);
    }
}

#[derive(Debug, Default, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReportBo {
    /// 表名或字段名
    name: String,
    /// 建议列表
    suggests: Vec<Suggest>,
    /// 字段列表
    children: Vec<CheckReportBo>,
}

#[derive(Debug, Clone, Default, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggest {
    /// 规范检查类型code
    code: i32,
    /// 检查建议
    desc: String,
    /// 原词
    origin_word: String,
    /// 是否展示
    show: u8,
}

impl From<StandardCheck> for Suggest {
    fn from(c: StandardCheck) -> Self {
        Self {
            code: c.code(),
            desc: c.desc().into(),
            origin_word: String::new(),
            show: 1,
        }
    }
}

impl Suggest {
    fn new(code: i32, desc: String) -> Self {
        Self {
            code,
            desc,
            origin_word: String::new(),
            show: 1,
        }
    }

    fn new_with_word(code: i32, desc: String, origin_word: String) -> Self {
        Self {
            code,
            desc,
            origin_word,
            show: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check() {
        let check = StandardCheck::NameContainUpperCase;
        assert_eq!(11, check.code());
        assert_eq!("应当使用小写", check.desc());

        let check = StandardCheck::FieldTypeUseFloat;
        assert_eq!(44, check.code());
        assert_eq!("小数类型为 decimal，禁止使用 float 和 double", check.desc());
    }

    #[test]
    fn test_check_codes() {
        assert_eq!(11, StandardCheck::codes().len())
    }
}
