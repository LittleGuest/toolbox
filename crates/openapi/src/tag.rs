use serde::{Deserialize, Serialize};

use super::{extensions::Extensions, external_docs::ExternalDocs};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}
