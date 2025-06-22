use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    extensions::Extensions, response::Response, security::SecurityScheme, xml::Xml, Deprecated,
    Number, RefOr,
};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Components {
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub schemas: BTreeMap<String, RefOr<Schema>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub responses: BTreeMap<String, RefOr<Response>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub security_schemes: BTreeMap<String, SecurityScheme>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged, rename_all = "camelCase")]
pub enum Schema {
    Array(Array),
    Object(Box<Object>),
    OneOf(OneOf),
    AllOf(AllOf),
    AnyOf(AnyOf),
}

impl Default for Schema {
    fn default() -> Self {
        Schema::Object(Box::default())
    }
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Discriminator {
    pub property_name: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub mapping: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct OneOf {
    #[serde(rename = "oneOf")]
    pub items: Vec<RefOr<Schema>>,
    #[serde(
        rename = "type",
        default = "SchemaType::any",
        skip_serializing_if = "SchemaType::is_any_value"
    )]
    pub schema_type: SchemaType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub examples: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<Discriminator>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct AllOf {
    #[serde(rename = "allOf")]
    pub items: Vec<RefOr<Schema>>,
    #[serde(
        rename = "type",
        default = "SchemaType::any",
        skip_serializing_if = "SchemaType::is_any_value"
    )]
    pub schema_type: SchemaType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub examples: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<Discriminator>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct AnyOf {
    #[serde(rename = "anyOf")]
    pub items: Vec<RefOr<Schema>>,
    #[serde(
        rename = "type",
        default = "SchemaType::any",
        skip_serializing_if = "SchemaType::is_any_value"
    )]
    pub schema_type: SchemaType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub examples: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<Discriminator>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

type ObjectPropertiesMap<K, V> = BTreeMap<K, V>;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    #[serde(rename = "type", skip_serializing_if = "SchemaType::is_any_value")]
    pub schema_type: SchemaType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<SchemaFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub required: Vec<String>,
    #[serde(
        skip_serializing_if = "ObjectPropertiesMap::is_empty",
        default = "ObjectPropertiesMap::new"
    )]
    pub properties: ObjectPropertiesMap<String, RefOr<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<Box<AdditionalProperties<Schema>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_names: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<Deprecated>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub examples: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xml: Option<Xml>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "omit_decimal_zero"
    )]
    pub multiple_of: Option<Number>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "omit_decimal_zero"
    )]
    pub maximum: Option<Number>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "omit_decimal_zero"
    )]
    pub minimum: Option<Number>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "omit_decimal_zero"
    )]
    pub exclusive_maximum: Option<Number>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "omit_decimal_zero"
    )]
    pub exclusive_minimum: Option<Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub content_encoding: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub content_media_type: String,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum AdditionalProperties<T> {
    RefOr(RefOr<T>),
    FreeForm(bool),
}

impl<T> From<RefOr<T>> for AdditionalProperties<T> {
    fn from(value: RefOr<T>) -> Self {
        Self::RefOr(value)
    }
}

impl From<Ref> for AdditionalProperties<Schema> {
    fn from(value: Ref) -> Self {
        Self::RefOr(RefOr::Ref(value))
    }
}

impl From<Schema> for AdditionalProperties<Schema> {
    fn from(value: Schema) -> Self {
        Self::RefOr(RefOr::T(value))
    }
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct Ref {
    #[serde(rename = "$ref")]
    pub ref_location: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub description: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub summary: String,
}

fn omit_decimal_zero<S>(maybe_value: &Option<Number>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match maybe_value {
        Some(Number::Float(float)) => {
            if float.fract() == 0.0 && *float >= i64::MIN as f64 && *float <= i64::MAX as f64 {
                serializer.serialize_i64(float.trunc() as i64)
            } else {
                serializer.serialize_f64(*float)
            }
        }
        Some(Number::Int(int)) => serializer.serialize_i64(*int as i64),
        Some(Number::UInt(uint)) => serializer.serialize_u64(*uint as u64),
        None => serializer.serialize_none(),
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ArrayItems {
    RefOrSchema(Box<RefOr<Schema>>),
    #[serde(with = "array_items_false")]
    False,
}

mod array_items_false {
    use serde::de::Visitor;

    pub fn serialize<S: serde::Serializer>(serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(false)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<(), D::Error> {
        struct ItemsFalseVisitor;

        impl<'de> Visitor<'de> for ItemsFalseVisitor {
            type Value = ();
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if !v {
                    Ok(())
                } else {
                    Err(serde::de::Error::custom(format!(
                        "invalid boolean value: {v}, expected false"
                    )))
                }
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("expected boolean false")
            }
        }

        deserializer.deserialize_bool(ItemsFalseVisitor)
    }
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Array {
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub items: ArrayItems,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub prefix_items: Vec<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<Deprecated>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub examples: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub unique_items: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xml: Option<Xml>,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub content_encoding: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub content_media_type: String,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum SchemaType {
    Type(Type),
    Array(Vec<Type>),
    AnyValue,
}

impl Default for SchemaType {
    fn default() -> Self {
        Self::Type(Type::default())
    }
}

impl From<Type> for SchemaType {
    fn from(value: Type) -> Self {
        SchemaType::new(value)
    }
}

impl FromIterator<Type> for SchemaType {
    fn from_iter<T: IntoIterator<Item = Type>>(iter: T) -> Self {
        Self::Array(iter.into_iter().collect())
    }
}

impl SchemaType {
    pub fn new(r#type: Type) -> Self {
        Self::Type(r#type)
    }

    pub fn any() -> Self {
        SchemaType::AnyValue
    }

    pub fn is_any_value(&self) -> bool {
        matches!(self, Self::AnyValue)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    #[default]
    Object,
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Null,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase", untagged)]
pub enum SchemaFormat {
    KnownFormat(KnownFormat),
    Custom(String),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum KnownFormat {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Double,
    Byte,
    Binary,
    Time,
    Date,
    DateTime,
    Duration,
    Password,
    Uuid,
    Ulid,
    Uri,
    UriReference,
    Iri,
    IriReference,
    Email,
    IdnEmail,
    Hostname,
    IdnHostname,
    Ipv4,
    Ipv6,
    UriTemplate,
    JsonPointer,
    RelativeJsonPointer,
    Regex,
}
