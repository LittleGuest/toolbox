use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{RefOr, encoding::Encoding, example::Example, extensions::Extensions, schema::Schema};

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[non_exhaustive]
pub struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<RefOr<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub examples: BTreeMap<String, RefOr<Example>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub encoding: BTreeMap<String, Encoding>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}
