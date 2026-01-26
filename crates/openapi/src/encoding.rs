use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{extensions::Extensions, header::Header, path::ParameterStyle};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Encoding {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, Header>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ParameterStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_reserved: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}
