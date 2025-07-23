#![allow(dead_code)]

use std::collections::HashSet;

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
    providers: Vec<String>,
}

impl Faker {
    pub fn new() -> Self {
        Self {
            rng: rand::rng(),
            locale: Default::default(),
            providers: {
                let mut p = Vec::new();
                p.push("education".into());
                p.push("address".into());
                p.push("emoji".into());
                p.push("file".into());
                p.push("internet".into());
                p.push("number".into());
                p.push("person".into());
                p.push("uuid".into());
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
pub async fn datafaker_providers() -> Result<Vec<String>> {
    Ok(Faker::new().providers)
}

/// 根据表字段名和类型适配一个合适的生成器，优先适配字段名
#[tauri::command]
pub async fn datafaker_adapter(field_name: String, field_type: String) -> Result<String> {
    if field_name.is_empty() {
        return Ok("name".into());
    }

    if field_type.is_empty() {
        return Ok("name".into());
    }
    let field_type = field_type.to_lowercase();
    if field_type.contains("int") || field_type.contains("number") {
        return Ok("number".into());
    }
    if field_type.contains("text") || field_type.contains("char") {
        return Ok("person".into());
    }
    if field_type.contains("date") || field_type.contains("time") {
        return Ok("datetime".into());
    }
    if field_type.contains("json") || field_type.contains("object") {
        return Ok("json".into());
    }
    Ok("name".into())
}
