use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{extensions::Extensions, server::Server};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Link {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub operation_ref: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub operation_id: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<Server>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}
