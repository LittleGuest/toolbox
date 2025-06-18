use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use super::{
    cores::{Column, ColumnType, Error, Result},
    DatasourceInfo,
};
use crate::database::cores::{DatabaseMetadata, Driver, MysqlMetadata};

mod report;

/// 所有表结构信息
pub async fn table_struct(datasource_info: &DatasourceInfo) -> Result<HashMap<String, TableBo>> {
    match datasource_info.driver {
        Driver::Mysql => {
            let Some(database) = &datasource_info.database else {
                return Err(Error::E("choose database"));
            };
            let pool = MySqlPool::connect(&datasource_info.url()).await?;

            let tables = MysqlMetadata::tables(&pool, database).await?;
            let mut data = HashMap::with_capacity(tables.len());
            for table in tables.into_iter() {
                // 表字段
                let columns = MysqlMetadata::columns(&pool, database, &table.name)
                    .await?
                    .into_iter()
                    .map(|c| (c.name.clone(), c.into()))
                    .collect::<HashMap<String, FieldBo>>();
                // 索引
                let indexs: HashMap<String, IndexBo> = HashMap::new();

                data.insert(
                    table.name.clone(),
                    TableBo {
                        name: table.name,
                        comment: table.comment,
                        columns,
                        indexs,
                        is_both_has: false,
                    },
                );
            }
            Ok(data)
        }
        Driver::Postgres => todo!(),
        Driver::Sqlite => todo!(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableBo {
    /// 表名
    name: String,
    /// 注释
    comment: String,
    /// 字段
    columns: HashMap<String, FieldBo>,
    /// 索引
    indexs: HashMap<String, IndexBo>,

    /// 是否双方都有(用于比对)
    is_both_has: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldBo {
    // 表名
    pub table_name: String,
    /// 字段名
    pub name: String,
    /// 字段类型
    pub r#type: Option<ColumnType>,
    /// 字段长度，可以为空
    pub length: Option<i32>,
    /// 字段精度
    pub scale: Option<i32>,
    /// 默认值
    pub default: Option<String>,
    /// 枚举值列表
    pub enum_values: Option<Vec<String>>,
    /// 备注
    pub comment: String,
    /// 是否为空
    pub is_null: bool,
    /// 是否自增
    pub is_auto_incr: bool,
    /// 是否唯一
    pub is_unique: bool,
    /// 是否主键
    pub is_primary_key: bool,
    /// 是否无符号
    pub is_unsigned: bool,

    /// 是否双方都有(用于比对)
    is_both_has: bool,
}

impl From<Column> for FieldBo {
    fn from(c: Column) -> Self {
        Self {
            table_name: c.table_name,
            name: c.name,
            r#type: c.r#type,
            length: c.length,
            scale: c.scale,
            default: c.default,
            enum_values: c.enum_values,
            comment: c.comment,
            is_null: c.is_null,
            is_auto_incr: c.is_auto_incr,
            is_unique: c.is_unique,
            is_primary_key: c.is_primary_key,
            is_unsigned: c.is_unsigned,
            is_both_has: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexBo {
    /// 表名
    table_name: String,
    /// 如果索引不能包括重复词，则为0。如果可以，则为1。
    non_unique: i32,
    /// 索引的名称
    key_name: String,
    /// 索引中的列的序号。对于组合索引，这表示列在索引中的位置。
    seq_index: i32,
    /// 作用于列名称
    column_name: String,
    /// 索引的前缀长度。对于部分索引，这表示索引的前缀长度。
    sub_part: i32,
    /// 用过的索引方法（BTREE, FULLTEXT, HASH, RTREE）
    index_type: String,
    /// 索引的注释
    index_comment: Option<String>,

    /// 是否双方都有(用于比对)
    is_both_has: bool,
}
