use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::extensions::Extensions;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct SecurityRequirement {
    #[serde(flatten)]
    value: BTreeMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SecurityScheme {
    #[serde(rename = "oauth2")]
    OAuth2(OAuth2),
    ApiKey(ApiKey),
    Http(Http),
    OpenIdConnect(OpenIdConnect),
    #[serde(rename = "mutualTLS")]
    MutualTls {
        #[allow(missing_docs)]
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", flatten)]
        extensions: Option<Extensions>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "in", rename_all = "lowercase")]
pub enum ApiKey {
    Header(ApiKeyValue),
    Query(ApiKeyValue),
    Cookie(ApiKeyValue),
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ApiKeyValue {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Http {
    pub scheme: HttpAuthScheme,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HttpAuthScheme {
    #[default]
    Basic,
    Bearer,
    Digest,
    Hoba,
    Mutual,
    Negotiate,
    OAuth,
    #[serde(rename = "scram-sha-1")]
    ScramSha1,
    #[serde(rename = "scram-sha-256")]
    ScramSha256,
    Vapid,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenIdConnect {
    pub open_id_connect_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct OAuth2 {
    pub flows: BTreeMap<String, Flow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Flow {
    Implicit(Implicit),
    Password(Password),
    ClientCredentials(ClientCredentials),
    AuthorizationCode(AuthorizationCode),
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Implicit {
    pub authorization_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
    #[serde(flatten)]
    pub scopes: Scopes,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationCode {
    pub authorization_url: String,
    pub token_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
    #[serde(flatten)]
    pub scopes: Scopes,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Password {
    pub token_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
    #[serde(flatten)]
    pub scopes: Scopes,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClientCredentials {
    pub token_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
    #[serde(flatten)]
    pub scopes: Scopes,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Scopes {
    scopes: BTreeMap<String, String>,
}
