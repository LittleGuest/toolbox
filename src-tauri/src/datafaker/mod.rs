#![allow(dead_code)]

use providers::{Address, Education, Emoji, File, Internet, Number, Person, Uuid};
use rust_embed::Embed;
use thiserror::Error;

mod generators;
mod providers;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid sequence min max")]
    InvalidSequenceMinMax,
    #[error("sequence count not enough")]
    SequenceCountNotEnough,
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

#[derive(Default, Clone, Copy)]
pub enum Locale {
    CN,
    #[default]
    En,
}

/// 数据提供者
pub trait Provider {
    fn name(&self) -> String;
}

/// 假数据生成器
pub struct Faker {
    pub locale: Locale,
    // providers: HashMap<String, Box<dyn Provider>>,
    providers: Vec<String>,
}

impl Faker {
    pub fn new() -> Self {
        Self {
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

    pub fn number(&self) -> Number {
        Number
    }

    pub fn person(&self) -> Person {
        Person::new_with_locale(self.locale)
    }

    pub fn uuid(&self) -> Uuid {
        Uuid
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
