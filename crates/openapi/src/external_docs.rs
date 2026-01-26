use serde::{Deserialize, Serialize};

use super::extensions::Extensions;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDocs {
    pub url: String,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}
