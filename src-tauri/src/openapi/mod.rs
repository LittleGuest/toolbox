//! <https://spec.openapis.org/oas/latest.html>

use std::{fmt::Formatter, fs::File, io::Write};

use extensions::Extensions;
use external_docs::ExternalDocs;
use info::Info;
use path::Paths;
use schema::{Components, Ref};
use security::SecurityRequirement;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Expected, Visitor},
};
use server::Server;
use tag::Tag;
use tauri::{AppHandle, Manager};
use tera::Tera;

use crate::{Result, Templates};

mod content;
mod encoding;
mod example;
mod extensions;
mod external_docs;
mod header;
mod info;
mod link;
mod path;
mod request_body;
mod response;
mod schema;
mod security;
mod server;
mod tag;
mod xml;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputType {
    Docx,
    Pdf,
    Markdown,
    Html,
}

// lazy_static::lazy_static! {
//     pub static ref TEMPLATES: Tera = {
//         let mut tera = match Tera::new("templates/**/*") {
//             Ok(t) => t,
//             Err(e) => {
//                 println!("Parsing error(s): {}", e);
//                 ::std::process::exit(1);
//             }
//         };
//         tera.autoescape_on(vec![".html", ".md","pdf", "docx"]);
//         // tera.register_filter("do_nothing", do_nothing_filter);
//         tera
//     };
// }

#[tauri::command]
pub async fn fetch_api_data(url: &str) -> Result<String> {
    Ok(reqwest::get(url).await?.text().await?)
}

#[tauri::command]
pub async fn download(url: &str, _output_type: OutputType, app: AppHandle) -> Result<String> {
    let data = fetch_api_data(url).await?;
    let openapi = serde_json::from_str::<OpenApi>(&data).unwrap();
    let mut context = tera::Context::new();
    context.insert("apiData", &openapi);
    let md = Templates::get("openapi.md").unwrap();
    let mut tera = Tera::default();
    let rendered = tera
        .render_str(str::from_utf8(md.data.as_ref()).unwrap(), &context)
        .unwrap();

    let download_dir = app.path().download_dir()?;
    let path = download_dir.join("xxxxxxxxxxxxxx.md");
    let mut file = File::create_new(&path).unwrap();
    file.write_all(rendered.as_bytes())?;

    Ok(rendered)
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OpenApi {
    pub openapi: String,
    pub info: Info,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,
    pub paths: Paths,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<SecurityRequirement>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
    #[serde(rename = "$schema", default, skip_serializing_if = "String::is_empty")]
    pub schema: String,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

// impl OpenApi {
//     /// Construct a new [`OpenApi`] object.
//     ///
//     /// Function accepts two arguments one which is [`Info`] metadata of the API; two which is [`Paths`]
//     /// containing operations for the API.
//     ///
//     /// # Examples
//     ///
//     /// ```rust
//     /// # use utoipa::openapi::{Info, Paths, OpenApi};
//     /// #
//     /// let openapi = OpenApi::new(Info::new("pet api", "0.1.0"), Paths::new());
//     /// ```
//     pub fn new<P: Into<Paths>>(info: Info, paths: P) -> Self {
//         Self {
//             info,
//             paths: paths.into(),
//             ..Default::default()
//         }
//     }
//
//     /// Converts this [`OpenApi`] to JSON String. This method essentially calls [`serde_json::to_string`] method.
//     pub fn to_json(&self) -> Result<String, serde_json::Error> {
//         serde_json::to_string(self)
//     }
//
//     /// Converts this [`OpenApi`] to pretty JSON String. This method essentially calls [`serde_json::to_string_pretty`] method.
//     pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
//         serde_json::to_string_pretty(self)
//     }
//
//     /// Converts this [`OpenApi`] to YAML String. This method essentially calls [`serde_norway::to_string`] method.
//     #[cfg(feature = "yaml")]
//     #[cfg_attr(doc_cfg, doc(cfg(feature = "yaml")))]
//     pub fn to_yaml(&self) -> Result<String, serde_norway::Error> {
//         serde_norway::to_string(self)
//     }
//
//     /// Merge `other` [`OpenApi`] moving `self` and returning combined [`OpenApi`].
//     ///
//     /// In functionality wise this is exactly same as calling [`OpenApi::merge`] but but provides
//     /// leaner API for chaining method calls.
//     pub fn merge_from(mut self, other: OpenApi) -> OpenApi {
//         self.merge(other);
//         self
//     }
//
//     /// Merge `other` [`OpenApi`] consuming it and resuming it's content.
//     ///
//     /// Merge function will take all `self` nonexistent _`servers`, `paths`, `schemas`, `responses`,
//     /// `security_schemes`, `security_requirements` and `tags`_ from _`other`_ [`OpenApi`].
//     ///
//     /// This function performs a shallow comparison for `paths`, `schemas`, `responses` and
//     /// `security schemes` which means that only _`name`_ and _`path`_ is used for comparison. When
//     /// match occurs the whole item will be ignored from merged results. Only items not
//     /// found will be appended to `self`.
//     ///
//     /// For _`servers`_, _`tags`_ and _`security_requirements`_ the whole item will be used for
//     /// comparison. Items not found from `self` will be appended to `self`.
//     ///
//     /// **Note!** `info`, `openapi`, `external_docs` and `schema` will not be merged.
//     pub fn merge(&mut self, mut other: OpenApi) {
//         if let Some(other_servers) = &mut other.servers {
//             let servers = self.servers.get_or_insert(Vec::new());
//             other_servers.retain(|server| !servers.contains(server));
//             servers.append(other_servers);
//         }
//
//         if !other.paths.paths.is_empty() {
//             self.paths.merge(other.paths);
//         };
//
//         if let Some(other_components) = &mut other.components {
//             let components = self.components.get_or_insert(Components::default());
//
//             other_components
//                 .schemas
//                 .retain(|name, _| !components.schemas.contains_key(name));
//             components.schemas.append(&mut other_components.schemas);
//
//             other_components
//                 .responses
//                 .retain(|name, _| !components.responses.contains_key(name));
//             components.responses.append(&mut other_components.responses);
//
//             other_components
//                 .security_schemes
//                 .retain(|name, _| !components.security_schemes.contains_key(name));
//             components
//                 .security_schemes
//                 .append(&mut other_components.security_schemes);
//         }
//
//         if let Some(other_security) = &mut other.security {
//             let security = self.security.get_or_insert(Vec::new());
//             other_security.retain(|requirement| !security.contains(requirement));
//             security.append(other_security);
//         }
//
//         if let Some(other_tags) = &mut other.tags {
//             let tags = self.tags.get_or_insert(Vec::new());
//             other_tags.retain(|tag| !tags.contains(tag));
//             tags.append(other_tags);
//         }
//     }
//
//     /// Nest `other` [`OpenApi`] to this [`OpenApi`].
//     ///
//     /// Nesting performs custom [`OpenApi::merge`] where `other` [`OpenApi`] paths are prepended with given
//     /// `path` and then appended to _`paths`_ of this [`OpenApi`] instance. Rest of the  `other`
//     /// [`OpenApi`] instance is merged to this [`OpenApi`] with [`OpenApi::merge_from`] method.
//     ///
//     /// **If multiple** APIs are being nested with same `path` only the **last** one will be retained.
//     ///
//     /// Method accepts two arguments, first is the path to prepend .e.g. _`/user`_. Second argument
//     /// is the [`OpenApi`] to prepend paths for.
//     ///
//     /// # Examples
//     ///
//     /// _**Merge `user_api` to `api` nesting `user_api` paths under `/api/v1/user`**_
//     /// ```rust
//     ///  # use utoipa::openapi::{OpenApi, OpenApiBuilder};
//     ///  # use utoipa::openapi::path::{PathsBuilder, PathItemBuilder, PathItem,
//     ///  # HttpMethod, OperationBuilder};
//     ///  let api = OpenApiBuilder::new()
//     ///      .paths(
//     ///          PathsBuilder::new().path(
//     ///              "/api/v1/status",
//     ///              PathItem::new(
//     ///                  HttpMethod::Get,
//     ///                  OperationBuilder::new()
//     ///                      .description(Some("Get status"))
//     ///                      .build(),
//     ///              ),
//     ///          ),
//     ///      )
//     ///      .build();
//     ///  let user_api = OpenApiBuilder::new()
//     ///     .paths(
//     ///         PathsBuilder::new().path(
//     ///             "/",
//     ///             PathItem::new(HttpMethod::Post, OperationBuilder::new().build()),
//     ///         )
//     ///     )
//     ///     .build();
//     ///  let nested = api.nest("/api/v1/user", user_api);
//     /// ```
//     pub fn nest<P: Into<String>, O: Into<OpenApi>>(self, path: P, other: O) -> Self {
//         self.nest_with_path_composer(path, other, |base, path| format!("{base}{path}"))
//     }
//
//     /// Nest `other` [`OpenApi`] with custom path composer.
//     ///
//     /// In most cases you should use [`OpenApi::nest`] instead.
//     /// Only use this method if you need custom path composition for a specific use case.
//     ///
//     /// `composer` is a function that takes two strings, the base path and the path to nest, and returns the composed path for the API Specification.
//     pub fn nest_with_path_composer<
//         P: Into<String>,
//         O: Into<OpenApi>,
//         F: Fn(&str, &str) -> String,
//     >(
//         mut self,
//         path: P,
//         other: O,
//         composer: F,
//     ) -> Self {
//         let path: String = path.into();
//         let mut other_api: OpenApi = other.into();
//
//         let nested_paths = other_api
//             .paths
//             .paths
//             .into_iter()
//             .map(|(item_path, item)| {
//                 let path = composer(&path, &item_path);
//                 (path, item)
//             })
//             .collect::<PathsMap<_, _>>();
//
//         self.paths.paths.extend(nested_paths);
//
//         // paths are already merged, thus we can ignore them
//         other_api.paths.paths = PathsMap::new();
//         self.merge_from(other_api)
//     }
// }

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum Number {
    Int(isize),
    UInt(usize),
    Float(f64),
}

impl Eq for Number {}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(left), Self::Int(right)) => left == right,

            (Self::UInt(left), Self::UInt(right)) => left == right,

            (Self::Float(left), Self::Float(right)) => left == right,

            _ => false,
        }
    }
}

#[derive(Serialize, Clone, PartialEq, Eq, Default)]
pub enum OpenApiVersion {
    #[serde(rename = "3.1.0")]
    #[default]
    Version31,
}

impl<'de> Deserialize<'de> for OpenApiVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionVisitor;

        impl<'v> Visitor<'v> for VersionVisitor {
            type Value = OpenApiVersion;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a version string in 3.1.x format")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_string(v.to_string())
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let version = v
                    .split('.')
                    .flat_map(|digit| digit.parse::<i8>())
                    .collect::<Vec<_>>();

                if version.len() == 3 && version.first() == Some(&3) && version.get(1) == Some(&1) {
                    Ok(OpenApiVersion::Version31)
                } else {
                    let expected: &dyn Expected = &"3.1.0";
                    Err(Error::invalid_value(
                        serde::de::Unexpected::Str(&v),
                        expected,
                    ))
                }
            }
        }

        deserializer.deserialize_string(VersionVisitor)
    }
}

#[derive(PartialEq, Eq, Clone, Default)]
pub enum Deprecated {
    True,
    #[default]
    False,
}

impl Serialize for Deprecated {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(matches!(self, Self::True))
    }
}

impl<'de> Deserialize<'de> for Deprecated {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BoolVisitor;
        impl<'de> Visitor<'de> for BoolVisitor {
            type Value = Deprecated;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a bool true or false")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    true => Ok(Deprecated::True),
                    false => Ok(Deprecated::False),
                }
            }
        }
        deserializer.deserialize_bool(BoolVisitor)
    }
}

#[derive(PartialEq, Eq, Clone, Default)]
pub enum Required {
    True,
    #[default]
    False,
}

impl Serialize for Required {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(matches!(self, Self::True))
    }
}

impl<'de> Deserialize<'de> for Required {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BoolVisitor;
        impl<'de> Visitor<'de> for BoolVisitor {
            type Value = Required;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a bool true or false")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    true => Ok(Required::True),
                    false => Ok(Required::False),
                }
            }
        }
        deserializer.deserialize_bool(BoolVisitor)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum RefOr<T> {
    Ref(Ref),
    T(T),
}
