use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct Xml {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrapped: Option<bool>,
}
