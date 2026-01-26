use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{Required, content::Content, extensions::Extensions};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub content: BTreeMap<String, Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Required>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}
