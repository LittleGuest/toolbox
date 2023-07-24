//! 配置文件格式转换
//!
//! JOSN <> YAML <> TOML <> XML

use serde::{Deserialize, Serialize};

use crate::{ToolError, ToolResult};

/// 支持的文件格式
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Ft {
    #[default]
    Json,
    Yaml,
    Toml,
    Xml,
}

impl std::fmt::Display for Ft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ft::Json => write!(f, "Json"),
            Ft::Yaml => write!(f, "Yaml"),
            Ft::Toml => write!(f, "Toml"),
            Ft::Xml => write!(f, "Xml"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    /// 原始格式
    pub from: Ft,
    /// 目标格式
    pub to: Ft,
    pub indentation: u8,
    /// 文本数据
    pub input: String,
}

impl Data {
    pub fn new(from: Ft, to: Ft, input: &str) -> Self {
        Self {
            from,
            to,
            indentation: 2,
            input: input.into(),
        }
    }

    /// FIXME 优化，判断文件格式
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

    /// 格式转换
    pub fn convert(&self) -> ToolResult<String> {
        if self.input.is_empty() {
            return Ok("".to_string());
        }

        let _ = Self::auto(&self.input);
        match self.from {
            Ft::Json => match self.to {
                Ft::Json => Ok(self.input.clone()),
                Ft::Yaml => Self::cto_yaml(Self::from_json::<serde_yaml::Value>(&self.input)?),
                Ft::Toml => Self::cto_toml(Self::from_json::<toml::Value>(&self.input)?),
                Ft::Xml => Self::cto_xml(Self::from_json(&self.input)?),
            },
            Ft::Yaml => match self.to {
                Ft::Json => Self::cto_json(Self::from_yaml::<serde_json::Value>(&self.input)?),
                Ft::Yaml => Ok(self.input.clone()),
                Ft::Toml => Self::cto_toml(Self::from_yaml::<toml::Value>(&self.input)?),
                Ft::Xml => Self::cto_xml(Self::from_json(&self.input)?),
            },
            Ft::Toml => match self.to {
                Ft::Json => Self::cto_json(Self::from_toml::<serde_json::Value>(&self.input)?),
                Ft::Yaml => Self::cto_yaml(Self::from_toml::<serde_yaml::Value>(&self.input)?),
                Ft::Toml => Ok(self.input.clone()),
                Ft::Xml => Self::cto_xml(Self::from_json(&self.input)?),
            },
            Ft::Xml => match self.to {
                Ft::Json => Self::cto_json(Self::from_toml::<serde_json::Value>(&self.input)?),
                Ft::Yaml => Self::cto_yaml(Self::from_toml::<serde_yaml::Value>(&self.input)?),
                Ft::Toml => Self::cto_toml(Self::from_yaml::<toml::Value>(&self.input)?),
                Ft::Xml => Ok(self.input.clone()),
            },
        }
    }
}

impl Data {
    /// 校验数据格式是否正确
    // pub fn check(&self) -> bool {
    //     if let Some(text) = self.text.as_ref() {
    //         if let Some(f) = self.from.as_ref() {
    //             // FIXME: 寻找更好的办法
    //             return match f.to_lowercase().as_ref() {
    //                 "json" => serde_json::from_str::<serde_json::Value>(text).is_ok(),
    //                 // FIXME: json也被认为是yaml格式的
    //                 "yaml" => serde_yaml::from_str::<serde_yaml::Value>(text).is_ok(),
    //                 "toml" => toml::from_str::<toml::Value>(text).is_ok(),
    //                 _ => false,
    //             };
    //         }
    //     }
    //     false
    // }

    fn from_json<T>(text: &str) -> ToolResult<T>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        serde_json::from_str(text).map_err(|e| ToolError::SerializeErr(e.to_string()))
    }

    fn from_yaml<T>(text: &str) -> ToolResult<T>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        serde_yaml::from_str::<T>(text).map_err(|e| ToolError::SerializeErr(e.to_string()))
    }

    fn from_toml<T>(text: &str) -> ToolResult<T>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        toml::from_str(text).map_err(|e| ToolError::SerializeErr(e.to_string()))
    }

    fn from_xml<T>(text: &str) -> ToolResult<T>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        quick_xml::de::from_str(text).map_err(|e| ToolError::SerializeErr(e.to_string()))
    }

    fn cto_json<T>(v: T) -> ToolResult<String>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        serde_json::to_string(&v).map_err(|e| ToolError::SerializeErr(e.to_string()))
    }

    fn cto_yaml<T>(v: T) -> ToolResult<String>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        serde_yaml::to_string(&v).map_err(|e| ToolError::SerializeErr(e.to_string()))
    }

    fn cto_toml<T>(v: T) -> ToolResult<String>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        toml::to_string(&v).map_err(|e| ToolError::SerializeErr(e.to_string()))
    }

    fn cto_xml<T>(v: T) -> ToolResult<String>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        quick_xml::se::to_string(&v).map_err(|e| ToolError::SerializeErr(e.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::{Data, Ft};

    const TOML: &str = r#"
    [package]
    name = "cffc"
    version = "0.1.0"
    authors = ["gopher9527 <gopher9527@gmail.com>"]
    edition = "2018"
    [dependencies]
    actix-web = "3.3.2"
    serde = "1.0.125"
    serde_json = "1.0.64"
    tera = "1.8.0"
    tokio = "1.5.0"
    toml = "0.5.8"
        "#;

    const JSON: &str = r#"
    {
        "package":{
            "name":"cffc",
            "version":"0.1.0",
            "authors":[
                "gopher9527 <gopher9527@gmail.com>"
            ],
            "edition":"2018"
        },
        "dependencies":{
            "actix-web":"3.3.2",
            "serde":"1.0.125",
            "serde_json":"1.0.64",
            "tera":"1.8.0",
            "tokio":"1.5.0",
            "toml":"0.5.8"
        }
    }
        "#;

    const YAML: &str = r#"
    package:
        name: cffc
        version: 0.1.0
        authors:
        - gopher9527 <gopher9527@gmail.com>
        edition: "2018"
    dependencies:
        actix-web: 3.3.2
        serde: 1.0.125
        serde_json: 1.0.64
        tera: 1.8.0
        tokio: 1.5.0
        toml: 0.5.8
        "#;

    const XML: &str = r#"
    <?xml version="1.0" encoding="UTF-8" ?>
    <package>
        <name>cffc</name>
        <version>0.1.0</version>
        <authors>gopher9527 &lt;gopher9527@gmail.com&gt;</authors>
        <edition>2018</edition>
    </package>
    <dependencies>
        <actix-web>3.3.2</actix-web>
        <serde>1.0.125</serde>
        <serde_json>1.0.64</serde_json>
        <tera>1.8.0</tera>
        <tokio>1.5.0</tokio>
        <toml>0.5.8</toml>
    </dependencies>
        "#;

    #[test]
    fn test_convert() {
        let c = Data::new(Ft::Yaml, Ft::Toml, YAML);

        let _ = c.convert().unwrap();
    }

    // #[test]
    fn test() {
        // toml --> json
        let mut deserializer = toml::Deserializer::new(TOML);
        let mut serializer = serde_json::Serializer::new(std::io::stdout());
        serde_transcode::transcode(&mut deserializer, &mut serializer).unwrap();

        let t = toml::from_str::<serde_json::Value>(TOML).unwrap();
        let ts = serde_json::to_string(&t).unwrap();
        println!("\n{}", ts);

        println!("=======================================");

        // toml --> yaml
        let mut deserializer = toml::Deserializer::new(TOML);
        let mut serializer = serde_yaml::Serializer::new(std::io::stdout());
        serde_transcode::transcode(&mut deserializer, &mut serializer).unwrap();

        println!("=======================================");

        // json --> yaml
        let mut jde = serde_json::Deserializer::from_str(JSON);
        let mut jse = serde_yaml::Serializer::new(std::io::stdout());
        serde_transcode::transcode(&mut jde, &mut jse).unwrap();

        println!("=======================================");

        // json --> toml
        let mut jde = serde_json::Deserializer::from_str(JSON);
        let mut tser = "".to_string();
        let mut tse = toml::Serializer::new(&mut tser);
        serde_transcode::transcode(&mut jde, &mut tse).unwrap();
        println!("{}", tser);

        println!("=======================================");

        // yaml --> json
        let yjs = serde_yaml::from_str::<serde_json::Value>(YAML).unwrap();
        let yjs = serde_json::to_string(&yjs).unwrap();
        println!("{}", yjs);

        println!("=======================================");

        // yaml --> toml
        let yt = serde_yaml::from_str::<toml::Value>(YAML).unwrap();
        let ys = toml::to_string(&yt).unwrap();
        println!("{}", ys);
    }
}
