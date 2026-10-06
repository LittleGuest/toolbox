use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Ft {
    #[default]
    Json,
    Yaml,
    Toml,
}

impl From<&str> for Ft {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_ref() {
            "json" => Self::Json,
            "yaml" => Self::Yaml,
            "toml" => Self::Toml,
            _ => Self::default(),
        }
    }
}

impl std::fmt::Display for Ft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ft::Json => write!(f, "Json"),
            Ft::Yaml => write!(f, "Yaml"),
            Ft::Toml => write!(f, "Toml"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub from: Ft,
    pub to: Ft,
    pub indent: u8,
    pub input: String,
}

impl Data {
    pub fn new(from: Ft, to: Ft, input: &str, indent: u8) -> Self {
        Self {
            from,
            to,
            indent,
            input: input.into(),
        }
    }

    fn auto(text: &str) -> Option<Ft> {
        if serde_json::from_str::<serde_json::Value>(text).is_ok() {
            return Some(Ft::Json);
        }
        if serde_yaml::from_str::<serde_yaml::Value>(text).is_ok() {
            return Some(Ft::Yaml);
        }
        if toml::from_str::<toml::Value>(text).is_ok() {
            return Some(Ft::Toml);
        }
        None
    }

    pub fn transform(&self) -> Result<String> {
        if self.input.is_empty() {
            return Ok(String::new());
        }

        let _ = Self::auto(&self.input);
        match self.from {
            Ft::Json => match self.to {
                Ft::Json => Ok(self.input.clone()),
                Ft::Yaml => Self::cto_yaml(Self::from_json::<serde_yaml::Value>(&self.input)?),
                Ft::Toml => Self::cto_toml(Self::from_json::<toml::Value>(&self.input)?),
            },
            Ft::Yaml => match self.to {
                Ft::Json => Self::cto_json(Self::from_yaml::<serde_json::Value>(&self.input)?),
                Ft::Yaml => Ok(self.input.clone()),
                Ft::Toml => Self::cto_toml(Self::from_yaml::<toml::Value>(&self.input)?),
            },
            Ft::Toml => match self.to {
                Ft::Json => Self::cto_json(Self::from_toml::<serde_json::Value>(&self.input)?),
                Ft::Yaml => Self::cto_yaml(Self::from_toml::<serde_yaml::Value>(&self.input)?),
                Ft::Toml => Ok(self.input.clone()),
            },
        }
    }
}

impl Data {
    fn from_json<T>(text: &str) -> Result<T>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        serde_json::from_str(text).map_err(|e| Error::msg(e.to_string()))
    }

    fn from_yaml<T>(text: &str) -> Result<T>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        serde_yaml::from_str::<T>(text).map_err(|e| Error::msg(e.to_string()))
    }

    fn from_toml<T>(text: &str) -> Result<T>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        toml::from_str(text).map_err(|e| Error::msg(e.to_string()))
    }

    fn from_xml<T>(text: &str) -> Result<T>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        quick_xml::de::from_str(text).map_err(|e| Error::msg(e.to_string()))
    }

    fn cto_json<T>(v: T) -> Result<String>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        serde_json::to_string(&v).map_err(|e| Error::msg(e.to_string()))
    }

    fn cto_yaml<T>(v: T) -> Result<String>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        serde_yaml::to_string(&v).map_err(|e| Error::msg(e.to_string()))
    }

    fn cto_toml<T>(v: T) -> Result<String>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        toml::to_string(&v).map_err(|e| Error::msg(e.to_string()))
    }

    fn cto_xml<T>(v: T) -> Result<String>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        quick_xml::se::to_string(&v).map_err(|e| Error::msg(e.to_string()))
    }
}
