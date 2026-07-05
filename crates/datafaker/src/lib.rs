#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use providers::{Address, Education, Emoji, File, Internet, Number, Person, Uuid};
use rand::rngs::ThreadRng;
use rust_embed::Embed;
use serde::Deserialize;
use thiserror::Error;

pub use crate::providers::{Name, RegexGenerator};

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
    RegexSyntax,
    #[error("正则表达式随机字符串生成错误")]
    RegexGenerator,
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
    #[serde(rename = "zh_traditional", alias = "zh_tw")]
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
                p.insert("date".into(), "日期".into());
                p.insert("datetime".into(), "日期时间".into());
                p.insert("timestamp".into(), "时间戳".into());
                p.insert("timezone".into(), "时区名称".into());
                p.insert("email".into(), "邮箱".into());
                p.insert("enum".into(), "枚举".into());
                p.insert("file_extension".into(), "文件扩展名".into());
                p.insert("file_name".into(), "文件名".into());
                p.insert("file_path".into(), "文件路径".into());
                p.insert("foreign_key".into(), "外键".into());
                p.insert("province_city".into(), "省份和城市".into());
                p.insert("country_region".into(), "国家或地区".into());
                p.insert("latitude_longitude".into(), "经纬度".into());
                p.insert("phone_area_code".into(), "固话区号".into());
                p.insert("mobile".into(), "手机号".into());
                p.insert("phone".into(), "固定电话".into());
                p.insert("id_card".into(), "身份证号".into());
                p.insert("gender".into(), "性别".into());
                p.insert("username".into(), "用户名".into());
                p.insert("password".into(), "密码".into());
                p.insert("qq".into(), "QQ号".into());
                p.insert("nickname".into(), "昵称".into());
                p.insert("ethnicity".into(), "民族".into());
                p.insert("hostname".into(), "主机名".into());
                p.insert("ip".into(), "IP地址".into());
                p.insert("mac".into(), "MAC地址".into());
                p.insert("name".into(), "姓名".into());
                p.insert("number".into(), "数字".into());
                p.insert("money".into(), "金额".into());
                p.insert("stock".into(), "股票名称和代码".into());
                p.insert("stock_kline".into(), "日K线数据".into());
                p.insert("fund".into(), "基金名称和代码".into());
                p.insert("currency".into(), "货币信息".into());
                p.insert("bank_card".into(), "银行卡号".into());
                p.insert("payment_method".into(), "付款方式".into());
                p.insert("credit_card_type".into(), "信用卡类型".into());
                p.insert("credit_card_number".into(), "信用卡卡号".into());
                p.insert("credit_card_date".into(), "信用卡日期".into());
                p.insert("boolean".into(), "布尔值".into());
                p.insert("json".into(), "JSON".into());
                p.insert("address".into(), "地址".into());
                p.insert("city".into(), "城市".into());
                p.insert("state".into(), "省/州".into());
                p.insert("street_address".into(), "街道地址".into());
                p.insert("zip_code".into(), "邮编".into());
                p.insert("company".into(), "公司".into());
                p.insert("job_title".into(), "职位".into());
                p.insert("department".into(), "部门".into());
                p.insert("industry".into(), "行业".into());
                p.insert("degree".into(), "学历".into());
                p.insert("primary_school".into(), "小学名称".into());
                p.insert("primary_school_grade".into(), "小学年级".into());
                p.insert("high_school".into(), "中学名称".into());
                p.insert("high_school_grade".into(), "中学年级".into());
                p.insert("school_class".into(), "班级".into());
                p.insert("college".into(), "大学".into());
                p.insert("major".into(), "专业".into());
                p.insert("product_name".into(), "产品名称".into());
                p.insert("product_category".into(), "产品类别".into());
                p.insert("size".into(), "尺寸".into());
                p.insert("weight_unit".into(), "重量单位".into());
                p.insert("barcode".into(), "条码".into());
                p.insert("sku".into(), "SKU".into());
                p.insert("app_name".into(), "应用名".into());
                p.insert("app_bundle_id".into(), "应用Bundle ID".into());
                p.insert("app_version".into(), "应用版本".into());
                p.insert("user_agent".into(), "User-Agent".into());
                p.insert("port".into(), "端口".into());
                p.insert("color".into(), "颜色".into());
                p.insert("chinese_char".into(), "汉字".into());
                p.insert("idiom".into(), "成语".into());
                p.insert("license_plate".into(), "车牌号".into());
                p.insert("mobile_model".into(), "热门手机型号".into());
                p.insert(
                    "unified_social_credit_code".into(),
                    "统一社会信用代码".into(),
                );
                p.insert("data_tool".into(), "数据生成工具".into());
                p.insert("binary".into(), "图像或二进制".into());
                p.insert("regex".into(), "正则表达式".into());
                p.insert("sequence".into(), "序列".into());
                p.insert("text".into(), "文本".into());
                p.insert("time".into(), "时间".into());
                p.insert("uuid".into(), "UUID".into());
                p.insert("website".into(), "网址".into());
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

    pub fn name(&self) -> Name<ThreadRng> {
        Name::new_with_locale(rand::rng(), self.locale)
    }

    pub fn name_with_locale(&self, locale: Locale) -> Name<ThreadRng> {
        Name::new_with_locale(rand::rng(), locale)
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

    pub fn get_providers(&self) -> &HashMap<String, String> {
        &self.providers
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
// #[tauri::command]
pub async fn datafaker_providers() -> Result<HashMap<String, String>> {
    Ok(Faker::new().providers)
}

/// 根据表字段名或字段类型匹配一个合适的生成器，优先适配字段名
// #[tauri::command]
pub async fn datafaker_adapter(
    field_name: Option<String>,
    field_type: Option<String>,
) -> Result<String> {
    // 根据字段名称匹配合适的生成器
    if let Some(field_name) = field_name {
        let name = field_name.to_lowercase();
        let normalized = name.replace(['-', ' '], "_");
        let tokens = normalized
            .split('_')
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>();

        if matches!(normalized.as_str(), "id" | "uuid")
            || normalized.ends_with("_id")
            || normalized.ends_with("_uuid")
        {
            return Ok("uuid".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "mobile" | "phone" | "telephone"))
        {
            return Ok(if tokens.iter().any(|token| *token == "mobile") {
                "mobile".into()
            } else {
                "phone".into()
            });
        }
        if normalized.contains("id_card")
            || normalized.contains("identity_card")
            || normalized.contains("idcard")
        {
            return Ok("id_card".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "gender" | "sex"))
        {
            return Ok("gender".into());
        }
        if matches!(
            normalized.as_str(),
            "username" | "user_name" | "account" | "login"
        ) || tokens.iter().any(|token| *token == "username")
        {
            return Ok("username".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "password" | "passwd" | "pwd"))
        {
            return Ok("password".into());
        }
        if tokens.iter().any(|token| *token == "qq") {
            return Ok("qq".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "ethnicity" | "nation" | "nationality"))
        {
            return Ok("ethnicity".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "host" | "hostname" | "domain"))
        {
            return Ok("hostname".into());
        }
        if tokens.iter().any(|token| *token == "ip")
            || normalized.ends_with("_ip")
            || normalized.contains("ip_address")
        {
            return Ok("ip".into());
        }
        if tokens.iter().any(|token| *token == "mac") {
            return Ok("mac".into());
        }
        if tokens
            .iter()
            .any(|token| *token == "email" || *token == "mail")
        {
            return Ok("email".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "url" | "uri" | "website"))
        {
            return Ok("website".into());
        }
        if normalized.contains("user_agent") {
            return Ok("user_agent".into());
        }
        if normalized.contains("app_bundle") || normalized.contains("bundle_id") {
            return Ok("app_bundle_id".into());
        }
        if normalized.contains("app_version") || normalized.contains("version") {
            return Ok("app_version".into());
        }
        if normalized.contains("app_name") {
            return Ok("app_name".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "name" | "nick" | "nickname"))
        {
            return Ok(
                if tokens
                    .iter()
                    .any(|token| matches!(*token, "nick" | "nickname"))
                {
                    "nickname".into()
                } else {
                    "name".into()
                },
            );
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "amount" | "money" | "price" | "cost" | "fee"))
        {
            return Ok("money".into());
        }
        if normalized.contains("stock_kline") || normalized.contains("kline") {
            return Ok("stock_kline".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "stock" | "share"))
        {
            return Ok("stock".into());
        }
        if tokens.iter().any(|token| *token == "fund") {
            return Ok("fund".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "currency" | "currencycode"))
        {
            return Ok("currency".into());
        }
        if normalized.contains("bank_card") || normalized.contains("bankcard") {
            return Ok("bank_card".into());
        }
        if normalized.contains("credit_card_number") {
            return Ok("credit_card_number".into());
        }
        if normalized.contains("credit_card_date") {
            return Ok("credit_card_date".into());
        }
        if normalized.contains("credit_card_type") {
            return Ok("credit_card_type".into());
        }
        if normalized.contains("payment_method") || normalized.contains("pay_method") {
            return Ok("payment_method".into());
        }
        if tokens.iter().any(|token| {
            matches!(
                *token,
                "age" | "count" | "num" | "number" | "total" | "quantity"
            )
        }) {
            return Ok("number".into());
        }
        if normalized.starts_with("is_")
            || normalized.starts_with("has_")
            || tokens
                .iter()
                .any(|token| matches!(*token, "active" | "enabled" | "disabled" | "flag"))
        {
            return Ok("boolean".into());
        }
        if normalized.contains("street_address") || tokens.iter().any(|token| *token == "street") {
            return Ok("street_address".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "address" | "addr"))
        {
            return Ok("address".into());
        }
        if normalized.contains("province_city") {
            return Ok("province_city".into());
        }
        if normalized.contains("country") || normalized.contains("region") {
            return Ok("country_region".into());
        }
        if normalized.contains("latitude")
            || normalized.contains("longitude")
            || normalized.contains("lat_lon")
        {
            return Ok("latitude_longitude".into());
        }
        if normalized.contains("area_code") {
            return Ok("phone_area_code".into());
        }
        if tokens.iter().any(|token| *token == "city") {
            return Ok("city".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "province" | "state"))
        {
            return Ok("state".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "zip" | "zipcode" | "postcode" | "postal"))
        {
            return Ok("zip_code".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "company" | "corp" | "enterprise"))
        {
            return Ok("company".into());
        }
        if normalized.contains("job_title")
            || tokens
                .iter()
                .any(|token| matches!(*token, "job" | "position" | "title"))
        {
            return Ok("job_title".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "department" | "dept"))
        {
            return Ok("department".into());
        }
        if tokens.iter().any(|token| *token == "industry") {
            return Ok("industry".into());
        }
        if tokens.iter().any(|token| *token == "degree") {
            return Ok("degree".into());
        }
        if normalized.contains("primary_school") {
            return Ok("primary_school".into());
        }
        if normalized.contains("high_school") || normalized.contains("middle_school") {
            return Ok("high_school".into());
        }
        if tokens.iter().any(|token| *token == "class") {
            return Ok("school_class".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "college" | "university"))
        {
            return Ok("college".into());
        }
        if tokens.iter().any(|token| *token == "major") {
            return Ok("major".into());
        }
        if normalized.contains("product_category") {
            return Ok("product_category".into());
        }
        if normalized.contains("product_name") || tokens.iter().any(|token| *token == "product") {
            return Ok("product_name".into());
        }
        if tokens.iter().any(|token| *token == "size") {
            return Ok("size".into());
        }
        if normalized.contains("weight_unit") {
            return Ok("weight_unit".into());
        }
        if tokens.iter().any(|token| *token == "barcode") {
            return Ok("barcode".into());
        }
        if tokens.iter().any(|token| *token == "sku") {
            return Ok("sku".into());
        }
        if tokens.iter().any(|token| *token == "port") {
            return Ok("port".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "color" | "colour"))
        {
            return Ok("color".into());
        }
        if normalized.contains("license_plate") || normalized.contains("plate_no") {
            return Ok("license_plate".into());
        }
        if normalized.contains("mobile_model") || normalized.contains("phone_model") {
            return Ok("mobile_model".into());
        }
        if normalized.contains("social_credit") || normalized.contains("credit_code") {
            return Ok("unified_social_credit_code".into());
        }
        if normalized.contains("data_tool") || normalized.contains("faker_tool") {
            return Ok("data_tool".into());
        }
        if tokens.iter().any(|token| *token == "idiom") {
            return Ok("idiom".into());
        }
        if tokens.iter().any(|token| matches!(*token, "date" | "day")) {
            return Ok("date".into());
        }
        if tokens.iter().any(|token| *token == "timestamp") {
            return Ok("timestamp".into());
        }
        if normalized.contains("timezone") || normalized.contains("time_zone") {
            return Ok("timezone".into());
        }
        if tokens.iter().any(|token| matches!(*token, "time")) {
            return Ok("datetime".into());
        }
        if tokens
            .iter()
            .any(|token| matches!(*token, "status" | "type" | "mode" | "state"))
        {
            return Ok("enum".into());
        }
    }

    // 根据字段类型匹配生成器
    if let Some(field_type) = field_type {
        match field_type.to_lowercase().as_str() {
            "tinyint" | "int" | "smallint" | "integer" | "bigint" | "mediumint" | "numeric"
            | "decimal" | "double" | "float" | "real" | "double precision" => {
                return Ok("number".into());
            }
            "uuid" => return Ok("uuid".into()),
            "boolean" | "bool" | "bit" => return Ok("boolean".into()),
            "enum" => return Ok("enum".into()),
            "set" => return Ok("enum".into()),
            "json" | "jsonb" => return Ok("json".into()),
            "char" | "character" | "character varying" | "varchar" | "tinytext" | "text"
            | "longtext" | "mediumtext" => return Ok("text".into()),
            "date" => return Ok("date".into()),
            "datetime" => return Ok("datetime".into()),
            "time" | "time without time zone" | "time with time zone" => return Ok("time".into()),
            "timestamp" | "timestamp without time zone" | "timestamp with time zone" => {
                return Ok("datetime".into());
            }
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
// #[tauri::command]
pub async fn datafaker_adapter_columns(columns: Vec<Column>) -> Result<IndexMap<String, String>> {
    let mut res = IndexMap::new();
    for column in columns {
        let generator =
            datafaker_adapter(Some(column.name.clone()), Some(column.column_type.clone())).await?;
        res.insert(column.name, generator);
    }
    Ok(res)
}

/// 预览正则表达式数据
// #[tauri::command]
pub async fn preview_regex(pattern: String) -> Result<String> {
    let generator = RegexGenerator::new(pattern, None, None, None, false)?;
    generator.preview()
}
