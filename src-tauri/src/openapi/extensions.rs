use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Extensions {
    #[serde(flatten)]
    extensions: HashMap<String, serde_json::Value>,
}
