use std::{
    fs::{self},
    io::Write,
};

use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::database::{
    cores::{Column, DatabaseMetadata, Driver, MysqlMetadata, Table},
    DatasourceInfo, Result,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Generator {
    /// 数据源
    pub datasource_info: DatasourceInfo,
    /// 代码生成的路径
    pub path: String,
    /// 指定要生成代码的表名，为空表示全部
    pub table_names: Vec<String>,
    /// 忽略表名前缀
    pub ignore_table_prefix: Option<String>,

    /// 是否生成 Entity 文件
    pub gen_entity: bool,
    /// 是否生成 Mapper 文件
    pub gen_mapper: bool,
    /// 是否生成 MapperXml 文件
    pub gen_mapper_xml: bool,
    /// 是否生成 Service 文件
    pub gen_service: bool,
    /// 是否生成 ServiceImpl 文件
    pub gen_service_impl: bool,
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
        if !self.path.is_empty() && !self.path.ends_with('/') {
            self.path.push('/')
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.deal_path();
        let (tables, tables_columns) = self.prepare().await?;
        if tables.is_empty() {
            return Ok(());
        }

        if tables_columns.is_empty() {
            return Ok(());
        }
        // self.write(tables, tables_columns).await?;

        Ok(())
    }

    pub async fn prepare(&self) -> Result<(Vec<Table>, Vec<Column>)> {
        // let table_names = self
        //     .table_names
        //     .iter()
        //     .filter(|t| !t.is_empty())
        //     .collect::<Vec<_>>();

        match self.datasource_info.driver {
            Driver::Mysql => {
                let pool = MySqlPool::connect(&self.datasource_info.url()).await?;
                let tables = MysqlMetadata::tables(
                    &pool,
                    &self.datasource_info.database.clone().unwrap_or_default(),
                )
                .await?;

                let mut table_names = vec![];
                if self.table_names.is_empty() {
                    table_names = tables.iter().map(|t| t.name.clone()).collect::<Vec<_>>();
                }

                let mut columns = vec![];
                for t in table_names {
                    columns.extend(
                        MysqlMetadata::columns(
                            &pool,
                            &self.datasource_info.database.clone().unwrap_or_default(),
                            &t,
                        )
                        .await?,
                    );
                }
                todo!()
            }
            Driver::Postgres => todo!(),
            Driver::Sqlite => todo!(),
            // Driver::Sqlite => {
            //     // let pool = sqlx::SqlitePool::connect(&self.driver_url()).await?;
            //     // let tables = sqlite::tables(&pool, &table_names).await?;
            //     // let tables_columns = sqlite::columns(&pool, &table_names).await?;
            //     // Ok((tables, tables_columns))
            //     todo!()
            // }
            // Driver::Mysql => {
            //     // let pool = sqlx::MySqlPool::connect(&self.driver_url()).await?;
            //     // let tables = mysql::tables(&pool, &table_names).await?;
            //     // let tables_columns = mysql::columns(&pool, &table_names).await?;
            //     // Ok((tables, tables_columns))
            //     todo!()
            // }
            // Driver::Postgres => {
            //     // let pool = sqlx::PgPool::connect(&self.driver_url()).await?;
            //     // let tables =
            //     //     postgres::tables(&self.datasource_info.database, &pool, &table_names).await?;
            //     // let tables_columns =
            //     //     postgres::columns(&self.datasource_info.database, &pool, &table_names).await?;
            //     // Ok((tables, tables_columns))
            //     todo!()
            // }
        }
    }

    // pub async fn write(&self, tables: Vec<Table>, tables_columns: Vec<Column>) -> Result<()> {
    //     // 将tables转换为map，K：表名，V：表信息
    //     let table_map: HashMap<String, Table> =
    //         tables.into_iter().map(|t| (t.name.to_owned(), t)).collect();
    //
    //     // 组装表信息和表列信息，K：表名，V：表列信息
    //     // FIXME：有没有办法直接将Vec分组，类似Java的Collectors.groupby
    //     let table_column_map =
    //         table_map
    //             .keys()
    //             .fold(HashMap::new(), |mut table_column_map, table_name| {
    //                 table_column_map.insert(
    //                     table_name,
    //                     tables_columns
    //                         .iter()
    //                         .filter(|table_column| {
    //                             Some(table_name.clone()) == table_column.table_name
    //                         })
    //                         .collect::<Vec<_>>(),
    //                 );
    //                 table_column_map
    //             });
    //
    //     // 创建生成目录
    //     fs::create_dir_all(&self.path)?;
    //
    //     // 创建模板引擎
    //     let mut ctx = tera::Context::new();
    //     ctx.insert("driver", &self.driver);
    //     ctx.insert("driver_url", &self.driver_url());
    //     ctx.insert("table_names", &table_map);
    //     let mut tera = tera::Tera::default();
    //
    //     for (table_name, table) in table_map.iter() {
    //         let column = table_column_map.get(&table_name);
    //         // 创建上下文
    //         ctx.insert("struct_name", &table_name.to_upper_camel_case());
    //         ctx.insert("table", &table);
    //         let mut has_columns = false;
    //         if let Some(columns) = column {
    //             has_columns = !columns.is_empty();
    //             ctx.insert("column_num", &columns.len());
    //             ctx.insert("columns", &columns);
    //             ctx.insert(
    //                 "column_names",
    //                 &columns
    //                     .iter()
    //                     .map(|c| c.name.clone().unwrap())
    //                     .collect::<Vec<String>>()
    //                     .join(","),
    //             );
    //         }
    //         ctx.insert("has_columns", &has_columns);
    //
    //         let contents = tera.render_str(MODEL_TEMPLATE, &ctx).expect("渲染模板错误");
    //         Self::write_file(&format!("{}{}.rs", self.path, &table_name), &contents).await?;
    //     }
    //
    //     // 创建 mod.rs 文件
    //     let contents = tera.render_str(MOD_TEMPLATE, &ctx)?;
    //     Self::write_file(&format!("{}mod.rs", self.path), &contents).await?;
    //
    //     // 创建 error.rs 文件
    //     let contents = tera.render_str(ERROR_TEMPLATE, &ctx)?;
    //     Self::write_file(&format!("{}error.rs", self.path), &contents).await?;
    //
    //     // 创建 result.rs 文件
    //     let contents = tera.render_str(RESULT_TEMPLATE, &ctx)?;
    //     Self::write_file(&format!("{}result.rs", self.path), &contents).await?;
    //
    //     Ok(())
    // }

    async fn write_file(path: &str, contents: &str) -> Result<()> {
        // let mut tf = fs::File::create(path)?;
        // tf.write_all(contents.as_bytes())?;
        // println!("the {path} has been generated");
        Ok(())
    }
}
