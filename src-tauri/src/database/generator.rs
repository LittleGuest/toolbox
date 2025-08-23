use std::{
    collections::HashMap,
    fs::{self},
    io::Write,
    path::Path,
};

use database::{
    Column, Driver, Table, database_metadata,
    error::{Error, Result},
};
use heck::ToUpperCamelCase;
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::{Templates, database::DatasourceInfo};

/// Rust 1.85关键字
const KEYWORDS: [&str; 53] = [
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "Self", "self", "static", "struct", "super", "trait", "true", "type", "union",
    "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "gen", "macro",
    "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
];

/// 判断字段名称是否是由多个单词组成
fn multi_world(name: &str) -> bool {
    name.contains(|c| ['_', '-'].contains(&c))
}

/// 列名是否为Rust关键字，若为关键字，则需要在其前加 r#
fn is_keywords(name: &str) -> String {
    if KEYWORDS.contains(&name) {
        format!("r#{name}")
    } else {
        name.into()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    Rust,
    Java,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Generator {
    /// 数据源
    pub datasource_info: DatasourceInfo,
    /// 编程语言
    pub language: Language,
    /// 指定要生成代码的表名，为空表示全部
    pub table_names: Vec<String>,
    /// 忽略的表名
    pub ignore_tables: Vec<String>,
    /// 忽略表名前缀
    pub ignore_table_prefix: Option<String>,

    /// 代码生成的路径
    pub path: Option<String>,
    /// 是否覆盖
    pub r#override: bool,

    /// 是否生成 mod.rs 文件
    pub gen_mod: bool,
    /// 是否生成 error.rs 文件
    pub gen_error: bool,
    /// 是否生成 Entity 文件
    pub gen_entity: bool,
    /// 是否生成 Mapper 文件
    pub gen_mapper: bool,
    /// 是否生成 MapperXml 文件
    pub gen_mapper_xml: bool,
    /// 是否生成 Service 文件
    pub gen_service: bool,
    /// 是否生成 Controller 文件
    pub gen_controller: bool,

    /// entity的包名
    pub entity_package_name: Option<String>,
    /// mapper的包名
    pub mapper_package_name: Option<String>,
    /// mapperXml的包名
    pub mapper_xml_package_name: Option<String>,
    /// service的包名
    pub service_package_name: Option<String>,
    /// serviceImpl的包名
    pub service_impl_package_name: Option<String>,
    /// controller的包名
    pub controller_package_name: Option<String>,
}

impl Generator {
    ///  处理路径，当路径不以 / 结尾时，自动添加 /
    fn deal_path(&mut self) {
        if let Some(path) = &mut self.path
            && !path.is_empty()
            && !path.ends_with('/')
        {
            path.push('/')
        }
    }

    pub async fn run(&mut self, preview: bool) -> Result<HashMap<String, HashMap<String, String>>> {
        self.deal_path();
        let (tables, tables_columns) = self.prepare().await?;
        if tables.is_empty() {
            return Ok(HashMap::new());
        }

        if tables_columns.is_empty() {
            return Ok(HashMap::new());
        }

        if preview {
            self.preview(tables, tables_columns).await
        } else {
            self.write(tables, tables_columns).await?;
            Ok(HashMap::new())
        }
    }

    async fn prepare(&self) -> Result<(Vec<Table>, Vec<Column>)> {
        let meta = database_metadata(&self.datasource_info.url()).await;
        let tables = meta
            .tables(&self.datasource_info.database.clone().unwrap_or_default())
            .await?;
        let mut table_names = vec![];
        if self.table_names.is_empty() {
            table_names = tables.iter().map(|t| t.name.clone()).collect::<Vec<_>>();
        }
        let mut columns = vec![];
        for t in table_names {
            columns.extend(
                meta.columns(
                    &self.datasource_info.database.clone().unwrap_or_default(),
                    &t,
                )
                .await?,
            );
        }
        Ok((tables, columns))
    }

    /// 渲染模板
    async fn render(&self, path: &str, tera: &mut Tera, ctx: &tera::Context) -> Result<String> {
        let template = Templates::get(path).ok_or(Error::E("模板文件不存在"))?;
        Ok(tera
            .render_str(str::from_utf8(template.data.as_ref()).unwrap(), ctx)
            .map_err(|_| Error::E("模板渲染失败"))?)
    }

    /// 预览代码
    /// return
    ///     K：表名
    ///     V：HashMap
    ///         K：文件名
    ///         V：对应的code
    async fn preview(
        &self,
        tables: Vec<Table>,
        tables_columns: Vec<Column>,
    ) -> Result<HashMap<String, HashMap<String, String>>> {
        let mut res_map = HashMap::with_capacity(self.table_names.len());

        // 将tables转换为map，K：表名，V：表信息
        let table_map: HashMap<String, Table> =
            tables.into_iter().map(|t| (t.name.to_owned(), t)).collect();

        // 组装表信息和表列信息，K：表名，V：表列信息
        // FIXME：有没有办法直接将Vec分组，类似Java的Collectors.groupby
        let table_column_map =
            table_map
                .keys()
                .fold(HashMap::new(), |mut table_column_map, table_name| {
                    table_column_map.insert(
                        table_name,
                        tables_columns
                            .iter()
                            .filter(|table_column| {
                                Some(table_name.clone()) == Some(table_column.table_name.clone())
                            })
                            .collect::<Vec<_>>(),
                    );
                    table_column_map
                });

        // 创建模板引擎
        let mut ctx = tera::Context::new();
        ctx.insert("driver", &self.datasource_info.driver);
        ctx.insert("driver_url", &self.datasource_info.url());
        ctx.insert("table_names", &table_map);
        let mut tera = tera::Tera::default();
        match self.language {
            Language::Rust => {
                if self.gen_error {
                    let mut map = HashMap::with_capacity(1);
                    map.insert(
                        "error.rs".into(),
                        self.render("code/rust/error.html", &mut tera, &ctx).await?,
                    );
                    res_map.insert("error.rs".into(), map);
                }
                if self.gen_mod {
                    let mut map = HashMap::with_capacity(1);
                    map.insert(
                        "mod.rs".into(),
                        self.render("code/rust/mod.html", &mut tera, &ctx).await?,
                    );
                    res_map.insert("mod.rs".into(), map);
                }

                for (table_name, table) in table_map.iter() {
                    let column = table_column_map.get(&table_name);
                    // 创建上下文
                    ctx.insert("struct_name", &table_name.to_upper_camel_case());
                    ctx.insert("table", &table);
                    let mut has_columns = false;
                    if let Some(columns) = column {
                        has_columns = !columns.is_empty();
                        ctx.insert("column_num", &columns.len());
                        ctx.insert("columns", &columns);
                        ctx.insert(
                            "column_names",
                            &columns
                                .iter()
                                .map(|c| c.name.clone())
                                .collect::<Vec<String>>()
                                .join(","),
                        );
                    }
                    ctx.insert("has_columns", &has_columns);

                    let mut map = HashMap::with_capacity(3);
                    if self.gen_entity {
                        map.insert(
                            format!("{table_name}.rs"),
                            self.render("code/rust/model.html", &mut tera, &ctx).await?,
                        );
                    }
                    // if self.gen_service {
                    //     map.insert(
                    //         "service.rs".into(),
                    //         self.render("code/rust/service.html", &mut tera, &ctx)
                    //             .await?,
                    //     );
                    // }
                    // if self.gen_controller {
                    //     map.insert(
                    //         "api.rs".into(),
                    //         self.render("code/rust/api.html", &mut tera, &ctx).await?,
                    //     );
                    // }
                    res_map.insert(table_name.into(), map);
                }
            }
            Language::Java => {
                for table in &self.table_names {
                    let mut map = HashMap::with_capacity(6);
                    if self.gen_entity {
                        map.insert(
                            "entity.java".into(),
                            self.render("code/java/entity.html", &mut tera, &ctx)
                                .await?,
                        );
                    }
                    if self.gen_mapper {
                        map.insert(
                            "mapper.java".into(),
                            self.render("code/java/mapper.html", &mut tera, &ctx)
                                .await?,
                        );
                    }
                    if self.gen_mapper_xml {
                        map.insert(
                            "mapper.xml".into(),
                            self.render("code/java/mapperXml.html", &mut tera, &ctx)
                                .await?,
                        );
                    }
                    if self.gen_service {
                        map.insert(
                            "service.java".into(),
                            self.render("code/java/service.html", &mut tera, &ctx)
                                .await?,
                        );
                    }
                    if self.gen_controller {
                        map.insert(
                            "controller.java".into(),
                            self.render("code/java/controller.html", &mut tera, &ctx)
                                .await?,
                        );
                    }
                    res_map.insert(table.into(), map);
                }
            }
        }
        Ok(res_map)
    }

    /// 写入文件
    async fn write(&self, tables: Vec<Table>, tables_columns: Vec<Column>) -> Result<()> {
        let Some(ref path) = self.path else {
            return Err(Error::E("代码生成的路径为空"));
        };
        if path.is_empty() {
            return Err(Error::E("代码生成的路径为空"));
        }
        if tables.is_empty() {
            return Err(Error::E("表信息为空"));
        }

        let data = self.preview(tables, tables_columns).await?;
        match self.language {
            Language::Rust => {
                // 创建 error.rs 文件
                if self.gen_error
                    && let Some(code) = data.get("error.rs")
                    && let Some(code) = code.get("error.rs")
                {
                    Self::write_file(&format!("{path}/error.rs"), code, self.r#override).await?;
                }
                // 创建 mod.rs 文件
                if self.gen_mod
                    && let Some(code) = data.get("mod.rs")
                    && let Some(code) = code.get("mod.rs")
                {
                    Self::write_file(&format!("{path}/mod.rs"), code, self.r#override).await?;
                }
                // 创建 model 文件
                for (key, value) in data
                    .into_iter()
                    .filter(|(k, _)| !["error.rs", "mod.rs"].contains(&k.as_str()))
                {
                    for (file_name, code) in value {
                        dbg!(format!("{path}{key}/{file_name}"));
                        Self::write_file(
                            &format!("{path}{key}/{file_name}"),
                            &code,
                            self.r#override,
                        )
                        .await?;
                    }
                }
            }
            Language::Java => {
                todo!()
            }
        }
        Ok(())
    }

    /// 写入文件
    async fn write_file<P>(path: P, contents: &str, r#override: bool) -> Result<()>
    where
        P: AsRef<Path>,
    {
        if let Some(path) = path.as_ref().parent() {
            fs::create_dir_all(path)?;
        }
        if !path.as_ref().exists() || (path.as_ref().exists() && r#override) {
            let mut tf = fs::File::create(path)?;
            tf.write_all(contents.as_bytes())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn datasource_info() -> DatasourceInfo {
        DatasourceInfo {
            driver: Driver::Mysql,
            name: "127.0.0.1".into(),
            host: "127.0.0.1".into(),
            port: Some(3306),
            username: Some("root".into()),
            password: Some("123456".into()),
            database: Some("differ".into()),
        }
    }

    #[tokio::test]
    async fn test_preview_rust() -> Result<()> {
        sqlx::any::install_default_drivers();
        let mut gt = Generator {
            datasource_info: datasource_info(),
            language: Language::Rust,
            table_names: vec![],
            ignore_tables: vec![],
            ignore_table_prefix: None,
            path: Some(".".to_string()),
            r#override: false,
            gen_mod: true,
            gen_error: true,
            gen_entity: true,
            gen_mapper: false,
            gen_mapper_xml: false,
            gen_service: true,
            gen_controller: true,
            entity_package_name: None,
            mapper_package_name: None,
            mapper_xml_package_name: None,
            service_package_name: None,
            service_impl_package_name: None,
            controller_package_name: None,
        };
        let codes = gt.run(true).await?;
        assert!(!codes.is_empty());
        dbg!(&serde_json::to_string(&codes));
        Ok(())
    }

    #[tokio::test]
    async fn test_gen_rust() -> Result<()> {
        sqlx::any::install_default_drivers();
        let mut gt = Generator {
            datasource_info: datasource_info(),
            language: Language::Rust,
            table_names: vec![],
            ignore_tables: vec![],
            ignore_table_prefix: None,
            path: Some("./target".to_string()),
            r#override: false,
            gen_mod: true,
            gen_error: true,
            gen_entity: true,
            gen_mapper: false,
            gen_mapper_xml: false,
            gen_service: true,
            gen_controller: true,
            entity_package_name: None,
            mapper_package_name: None,
            mapper_xml_package_name: None,
            service_package_name: None,
            service_impl_package_name: None,
            controller_package_name: None,
        };
        let _ = gt.run(false).await;
        Ok(())
    }
}
