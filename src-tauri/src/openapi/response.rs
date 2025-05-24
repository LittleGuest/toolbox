use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{content::Content, extensions::Extensions, header::Header, link::Link};
use crate::openapi::RefOr;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Responses {
    #[serde(flatten)]
    pub responses: BTreeMap<String, RefOr<Response>>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub description: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub headers: BTreeMap<String, Header>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub content: BTreeMap<String, Content>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub links: BTreeMap<String, RefOr<Link>>,
}
