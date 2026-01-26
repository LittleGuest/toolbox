use serde::{Deserialize, Serialize};

use super::extensions::Extensions;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_of_service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct License {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}
