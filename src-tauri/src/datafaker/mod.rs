#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap, HashSet};

use indexmap::IndexMap;
use providers::{Address, Education, Emoji, File, Internet, Number, Person, Uuid};
use rand::rngs::ThreadRng;
use rust_embed::Embed;
use serde::Deserialize;
use thiserror::Error;

mod providers;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("无效的最大最小值")]
    InvalidSequenceMinMax,
    #[error("序列号数量不足")]
    SequenceCountNotEnough,
    #[error("开始值不能大于结束值")]
    StartNotGreaterThanEnd,
    #[error("小数位数不能超过10位")]
    DecimalPlacesNotGreaterThan10,
    #[error("出现百分比不能超过100")]
    PercentNotGreaterThan100,
    #[error("默认值长度不能超过字段长度")]
    LengthNotGreaterThanFieldLength,
    #[error("无效时间")]
    InvalidDateTime,
    #[error("无效参数 {0}")]
    InvalidParameter(&'static str),
    #[error("正则表达式错误")]
    RegexSyntaxError(#[from] regex_syntax::Error),
    #[error("正则表达式随机字符串生成错误")]
    RegexGeneratorError(#[from] rand_regex::Error),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Embed)]
#[folder = "fakerdata"]
pub struct FakerData;

#[derive(Default, Clone, Copy, Deserialize)]
pub enum Locale {
    /// 英文
    #[default]
    #[serde(rename = "en_us")]
    EnUs,
    /// 简体中文
    #[serde(rename = "zh_cn")]
    ZhCn,
    /// 繁体中文
    #[serde(rename = "zh_tw")]
    ZhTw,
    /// 中文拼音
    #[serde(rename = "zh_pinyin")]
    ZhPinyin,
}

/// 数据提供者
pub trait Provider {
    fn name(&self) -> String;
}

/// 假数据生成器
pub struct Faker {
    /// 随机数生成器
    rng: ThreadRng,
    pub locale: Locale,
    // providers: HashMap<String, Box<dyn Provider>>,
    providers: HashMap<String, String>,
}

impl Faker {
    pub fn new() -> Self {
        Self {
            rng: rand::rng(),
            locale: Default::default(),
            providers: {
                let mut p = HashMap::new();
                p.insert("address".into(), "地址".into());
                p.insert("education".into(), "教育".into());
                p.insert("emoji".into(), "emoji".into());
                p.insert("file".into(), "文件".into());
                p.insert("internet".into(), "互联网".into());
                p.insert("name".into(), "姓名".into());
                p.insert("number".into(), "数字".into());
                p.insert("person".into(), "个人".into());
                p.insert("regex".into(), "正则表达式".into());
                p.insert("sequence".into(), "序列".into());
                p.insert("text".into(), "文本".into());
                p.insert("uuid".into(), "UUID".into());
                p
            },
        }
    }

    // pub fn register(&mut self, key: String, value: Box<dyn Provider>) {
    //     self.providers.insert(key, value);
    // }

    // pub fn providers(&self) -> Vec<String> {
    //     self.providers.keys().cloned().collect::<Vec<_>>()
    // }

    pub fn education(&self) -> Education {
        Education::new_with_locale(self.locale)
    }

    pub fn address(&self) -> Address {
        Address::new_with_locale(self.locale)
    }

    pub fn emoji(&self) -> Emoji {
        Emoji
    }

    pub fn file(&self) -> File {
        File
    }

    pub fn internet(&self) -> Internet {
        Internet::new_with_locale(self.locale)
    }

    pub fn number(&self) -> Number<ThreadRng> {
        Number::new(self.rng.clone())
    }

    pub fn person(&self) -> Person {
        Person::new_with_locale(self.locale)
    }

    pub fn uuid(&self) -> Uuid {
        Uuid
    }
}

/// 默认值组件
#[derive(Debug, Clone)]
pub struct DefaultComponent {
    /// 默认值
    pub default: String,
    /// 默认值出现百分比
    pub percent: f64,
}

impl DefaultComponent {
    pub fn new(default: String, percent: f64) -> Self {
        Self { default, percent }
    }

    /// 检查默认值组件参数  
    pub fn check(&self, len: Option<usize>) -> Result<()> {
        if self.percent - 100.0 > 0.0 {
            return Err(Error::PercentNotGreaterThan100);
        }
        if let Some(len) = len
            && self.default.len() > len
        {
            return Err(Error::LengthNotGreaterThanFieldLength);
        }
        Ok(())
    }
}

/// NULL值组件
#[derive(Debug, Clone)]
pub struct NullComponent {
    /// NULL值出现百分比
    pub percent: f64,
}

impl NullComponent {
    pub fn new(percent: f64) -> Self {
        Self { percent }
    }

    /// 检查NULL值组件参数
    pub fn check(&self) -> Result<()> {
        if self.percent - 100.0 > 0.0 {
            return Err(Error::PercentNotGreaterThan100);
        }
        Ok(())
    }
}

/// 唯一值组件
#[derive(Debug, Clone)]
pub struct UniqueComponent {
    /// 已经生成的唯一值
    pub value: HashSet<String>,
}

impl UniqueComponent {
    pub fn new() -> Self {
        Self {
            value: HashSet::new(),
        }
    }
}

/// 获取数据提供者列表
#[tauri::command]
pub async fn datafaker_providers() -> Result<HashMap<String, String>> {
    Ok(Faker::new().providers)
}

/// 根据表字段名或字段类型匹配一个合适的生成器，优先适配字段名
#[tauri::command]
pub async fn datafaker_adapter(
    field_name: Option<String>,
    field_type: Option<String>,
) -> Result<String> {
    // 根据字段名称匹配合适的生成器
    if let Some(field_name) = field_name {
        match field_name.to_lowercase().as_str() {
            "id" => return Ok("uuid".into()),
            "name" | "nick_name" => return Ok("name".into()),
            "username" => return Ok("internet".into()),
            "age" => return Ok("number".into()),
            "sex" => return Ok("person".into()),
            "email" | "phone" | "ip" => return Ok("internet".into()),
            "address" => return Ok("address".into()),
            "date" | "time" | "datetime" | "timestamp" => return Ok("datetime".into()),
            _ => {}
        }
    }

    // 根据字段类型匹配生成器
    if let Some(field_type) = field_type {
        match field_type.to_lowercase().as_str() {
            "tinyint" | "int" | "smallint" | "integer" | "bigint" | "mediumint" => {
                return Ok("number".into());
            }
            "binary" => return Ok("binary".into()),
            "bit" => return Ok("bit".into()),
            "blob" => return Ok("blob".into()),
            "decimal" => return Ok("decimal".into()),
            "double" => return Ok("double".into()),
            "enum" => return Ok("enum".into()),
            "float" => return Ok("float".into()),
            "geometry" => return Ok("geometry".into()),
            "geometrycollection" => return Ok("geometrycollection".into()),
            "json" => return Ok("json".into()),
            "longblob" => return Ok("longblob".into()),
            "mediumblob" => return Ok("mediumblob".into()),
            "multipoint" => return Ok("multipoint".into()),
            "numeric" => return Ok("numeric".into()),
            "point" => return Ok("point".into()),
            "polygon" => return Ok("polygon".into()),
            "real" => return Ok("real".into()),
            "set" => return Ok("set".into()),
            "char" => return Ok("char".into()),
            "varchar" | "tinytext" | "text" | "longtext" | "mediumtext" | "linestring"
            | "multilinestring" => return Ok("text".into()),
            "date" => return Ok("date".into()),
            "datetime" => return Ok("datetime".into()),
            "time" => return Ok("time".into()),
            "timestamp" => return Ok("timestamp".into()),
            "tinyblob" => return Ok("tinyblob".into()),
            "varbinary" => return Ok("varbinary".into()),
            "year" => return Ok("year".into()),
            _ => {}
        }
    }
    // 默认正则表达式生成器
    Ok("regex".into())
}

/// 表字段
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    /// 字段名
    pub name: String,
    /// 字段类型
    pub column_type: String,
}

/// 根据表字段名或字段类型匹配一个合适的生成器，优先适配字段名
#[tauri::command]
pub async fn datafaker_adapter_columns(columns: Vec<Column>) -> Result<IndexMap<String, String>> {
    let mut res = IndexMap::new();
    for column in columns {
        let generator =
            datafaker_adapter(Some(column.name.clone()), Some(column.column_type.clone())).await?;
        res.insert(column.name, generator);
    }
    Ok(res)
}
