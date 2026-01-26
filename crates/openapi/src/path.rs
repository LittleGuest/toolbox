use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    Deprecated, RefOr, Required, extensions::Extensions, external_docs::ExternalDocs,
    request_body::RequestBody, response::Responses, schema::Schema, security::SecurityRequirement,
    server::Server,
};

pub type PathsMap<K, V> = std::collections::BTreeMap<K, V>;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct Paths {
    #[serde(flatten)]
    pub paths: PathsMap<String, PathItem>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    pub responses: Responses,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callbacks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<Deprecated>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<SecurityRequirement>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: Option<String>,
    #[serde(rename = "in")]
    pub parameter_in: Option<ParameterIn>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required: Option<Required>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<Deprecated>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<RefOr<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ParameterStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_reserved: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    example: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[derive(Serialize, Default, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ParameterIn {
    Query,
    #[default]
    Path,
    Header,
    Cookie,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ParameterStyle {
    Matrix,
    Label,
    Form,
    Simple,
    SpaceDelimited,
    PipeDelimited,
    DeepObject,
}
