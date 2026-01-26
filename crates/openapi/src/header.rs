use serde::{Deserialize, Serialize};

use super::{RefOr, schema::Schema};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Header {
    pub schema: RefOr<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
