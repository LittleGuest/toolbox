use std::fmt::format;

use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, Database, FromRow, MySql, MySqlPool, Pool};

use super::{Column, DatabaseMetadata, Result};

pub struct MysqlDriver;

/// +-----------------+--------------------------------------------------------------------+------+-----+---------+-------+
/// | Field           | Type                                                               | Null | Key | Default | Extra |
/// +-----------------+--------------------------------------------------------------------+------+-----+---------+-------+
/// | TABLE_CATALOG   | varchar(64)                                                        | NO   |     | NULL    |       |
/// | TABLE_SCHEMA    | varchar(64)                                                        | NO   |     | NULL    |       |
/// | TABLE_NAME      | varchar(64)                                                        | NO   |     | NULL    |       |
/// | TABLE_TYPE      | enum('BASE TABLE','VIEW','SYSTEM VIEW')                            | NO   |     | NULL    |       |
/// | ENGINE          | varchar(64)                                                        | YES  |     | NULL    |       |
/// | VERSION         | int                                                                | YES  |     | NULL    |       |
/// | ROW_FORMAT      | enum('Fixed','Dynamic','Compressed','Redundant','Compact','Paged') | YES  |     | NULL    |       |
/// | TABLE_ROWS      | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | AVG_ROW_LENGTH  | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | DATA_LENGTH     | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | MAX_DATA_LENGTH | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | INDEX_LENGTH    | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | DATA_FREE       | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | AUTO_INCREMENT  | bigint unsigned                                                    | YES  |     | NULL    |       |
/// | CREATE_TIME     | timestamp                                                          | NO   |     | NULL    |       |
/// | UPDATE_TIME     | datetime                                                           | YES  |     | NULL    |       |
/// | CHECK_TIME      | datetime                                                           | YES  |     | NULL    |       |
/// | TABLE_COLLATION | varchar(64)                                                        | YES  |     | NULL    |       |
/// | CHECKSUM        | bigint                                                             | YES  |     | NULL    |       |
/// | CREATE_OPTIONS  | varchar(256)                                                       | YES  |     | NULL    |       |
/// | TABLE_COMMENT   | text                                                               | YES  |     | NULL    |       |
/// +-----------------+--------------------------------------------------------------------+------+-----+---------+-------+
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct Table {
    table_schema: String,
    table_name: String,
    table_comment: String,
}

/// +--------------------------+----------------------------+------+-----+---------+-------+
/// | Field                    | Type                       | Null | Key | Default | Extra |
/// +--------------------------+----------------------------+------+-----+---------+-------+
/// | TABLE_CATALOG            | varchar(64)                | NO   |     | NULL    |       |
/// | TABLE_SCHEMA             | varchar(64)                | NO   |     | NULL    |       |
/// | TABLE_NAME               | varchar(64)                | NO   |     | NULL    |       |
/// | COLUMN_NAME              | varchar(64)                | YES  |     | NULL    |       |
/// | ORDINAL_POSITION         | int unsigned               | NO   |     | NULL    |       |
/// | COLUMN_DEFAULT           | text                       | YES  |     | NULL    |       |
/// | IS_NULLABLE              | varchar(3)                 | NO   |     |         |       |
/// | DATA_TYPE                | longtext                   | YES  |     | NULL    |       |
/// | CHARACTER_MAXIMUM_LENGTH | bigint                     | YES  |     | NULL    |       |
/// | CHARACTER_OCTET_LENGTH   | bigint                     | YES  |     | NULL    |       |
/// | NUMERIC_PRECISION        | bigint unsigned            | YES  |     | NULL    |       |
/// | NUMERIC_SCALE            | bigint unsigned            | YES  |     | NULL    |       |
/// | DATETIME_PRECISION       | int unsigned               | YES  |     | NULL    |       |
/// | CHARACTER_SET_NAME       | varchar(64)                | YES  |     | NULL    |       |
/// | COLLATION_NAME           | varchar(64)                | YES  |     | NULL    |       |
/// | COLUMN_TYPE              | mediumtext                 | NO   |     | NULL    |       |
/// | COLUMN_KEY               | enum('','PRI','UNI','MUL') | NO   |     | NULL    |       |
/// | EXTRA                    | varchar(256)               | YES  |     | NULL    |       |
/// | PRIVILEGES               | varchar(154)               | YES  |     | NULL    |       |
/// | COLUMN_COMMENT           | text                       | NO   |     | NULL    |       |
/// | GENERATION_EXPRESSION    | longtext                   | NO   |     | NULL    |       |
/// | SRS_ID                   | int unsigned               | YES  |     | NULL    |       |
/// +--------------------------+----------------------------+------+-----+---------+-------+
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct TableColumn {
    table_schema: String,
    table_name: String,
    column_name: String,
    /// 字段顺序
    ordinal_position: Option<u32>,
    /// 默认值
    column_default: Option<String>,
    /// 是否允许为null
    is_nullable: String,
    data_type: Option<String>,
    character_maximum_length: Option<i64>,
    column_type: String,
    column_comment: Option<String>,
}

impl From<Table> for super::Table {
    fn from(value: Table) -> Self {
        Self {
            schema: value.table_schema,
            name: value.table_name,
            comment: value.table_comment,
        }
    }
}

impl From<TableColumn> for super::Column {
    fn from(value: TableColumn) -> Self {
        Self {
            schema: value.table_schema,
            table_name: value.table_name,
            name: value.column_name,
            data_type: value.data_type.unwrap(),
            max_length: value.character_maximum_length,
            is_nullable: match value.is_nullable.to_ascii_lowercase().as_str() {
                "yes" => true,
                _ => false,
            },
            comment: value.column_comment,
            default: value.column_default,
        }
    }
}

impl DatabaseMetadata for MysqlDriver {
    type Pool = MySqlPool;
    async fn tables(pool: Self::Pool, schema: &str) -> Result<Vec<super::Table>> {
        let sql = format!("SELECT TABLE_SCHEMA table_schema, TABLE_NAME table_name, TABLE_COMMENT table_comment FROM information_schema.`TABLES` WHERE TABLE_SCHEMA = '{schema}'");
        Ok(sqlx::query_as::<_, Table>(&sql)
            .fetch_all(&pool)
            .await?
            .into_iter()
            .map(|t| t.into())
            .collect::<Vec<_>>())
    }

    async fn columns(
        pool: Self::Pool,
        schema: &str,
        table_name: &str,
    ) -> Result<Vec<super::Column>> {
        let sql = format!("SELECT TABLE_SCHEMA table_schema, TABLE_NAME table_name, COLUMN_NAME column_name, ORDINAL_POSITION ordinal_position, COLUMN_DEFAULT column_default, IS_NULLABLE is_nullable, DATA_TYPE data_type, CHARACTER_MAXIMUM_LENGTH character_maximum_length, COLUMN_TYPE column_type, COLUMN_COMMENT column_comment FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = '{schema}' AND TABLE_NAME = '{table_name}'");
        Ok(sqlx::query_as::<_, TableColumn>(&sql)
            .fetch_all(&pool)
            .await?
            .into_iter()
            .map(|col| col.into())
            .collect::<Vec<super::Column>>())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::MySqlPool;

    use super::MysqlDriver;

    #[test]
    fn test_mysql() {
        let sql = "SELECT TABLE_SCHEMA table_schema, TABLE_NAME table_name, TABLE_COMMENT table_comment FROM information_schema.`TABLES` WHERE TABLE_SCHEMA = 'test'";
        let tables = sqlx::test_block_on(async {
            let pool = MySqlPool::connect("mysql://root:123456@127.0.0.1:3306/test")
                .await
                .unwrap();
            let tables = sqlx::query_as::<_, super::Table>(&sql)
                .fetch_all(&pool)
                .await
                .unwrap()
                .into_iter()
                .map(|t| t.into())
                .collect::<Vec<super::Table>>();
            tables
        });
        assert!(!tables.is_empty());
    }
}

// pub async fn tables(pool: &Pool<sqlx::MySql>, table_names: &[&str]) -> Result<Vec<super::Table>> {
//     let mut sql = "SELECT TABLE_SCHEMA table_schema, TABLE_NAME table_name, TABLE_COMMENT table_comment FROM information_schema.`TABLES` WHERE TABLE_SCHEMA = ( SELECT DATABASE ())"
//         .to_string();
//
//     if !table_names.is_empty() {
//         sql.push_str(&format!(
//             "AND FIND_IN_SET(TABLE_NAME, '{}')",
//             table_names.join(",")
//         ));
//     }
//
//     Ok(sqlx::query_as::<_, Table>(&sql)
//         .fetch_all(pool)
//         .await?
//         .into_iter()
//         .map(|t| t.into())
//         .collect::<Vec<_>>())
// }
//
// pub async fn columns(pool: &Pool<sqlx::MySql>, table_names: &[&str]) -> Result<Vec<super::Column>> {
//     let mut sql = r#"SELECT TABLE_SCHEMA table_schema, TABLE_NAME table_name, COLUMN_NAME column_name, ORDINAL_POSITION ordinal_position, COLUMN_DEFAULT column_default, IS_NULLABLE is_nullable, DATA_TYPE data_type, CHARACTER_MAXIMUM_LENGTH character_maximum_length, COLUMN_TYPE column_type, COLUMN_COMMENT column_comment FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = ( SELECT DATABASE ())"#
//         .to_string();
//
//     if !table_names.is_empty() {
//         sql.push_str(&format!(
//             "AND FIND_IN_SET(TABLE_NAME, '{}')",
//             table_names.join(",")
//         ));
//     }
//
//     Ok(sqlx::query_as::<_, TableColumn>(&sql)
//         .fetch_all(pool)
//         .await?
//         .into_iter()
//         .map(|col| col.into())
//         .collect::<Vec<super::Column>>())
// }
//

// enum MySqlType{
//
//     DECIMAL("DECIMAL"),
//     DECIMAL_UNSIGNED("DECIMAL ),
//
//
//     TINYINT("TINYINT"),
//
//     BOOLEAN("BOOLEAN"),
//
//     SMALLINT("SMALLINT"),
//
//     SMALLINT_UNSIGNED("SMALLINT ),
//
//     INT("INT"),
//
//     INT_UNSIGNED("INT ),
//
//
//     FLOAT("FLOAT"),
//
//     FLOAT_UNSIGNED("FLOAT ),
//                    "[(M,D)] [UNSIGNED] [ZEROFILL]"),
//
//     DOUBLE("DOUBLE"),
//
//     DOUBLE_UNSIGNED("DOUBLE ),
//                     "[(M,D)] [UNSIGNED] [ZEROFILL]"),
//
//     NULL("NULL"),
//
//     TIMESTAMP("TIMESTAMP"),
//
//     BIGINT("BIGINT"),
//
//     BIGINT_UNSIGNED("BIGINT ),
//                     "[(M)] [UNSIGNED] [ZEROFILL]"),
//
//     MEDIUMINT("MEDIUMINT"),
//
//     MEDIUMINT_UNSIGNED("MEDIUMINT ),
//                        8L, "[(M)] [UNSIGNED] [ZEROFILL]"),
//
//     DATE("DATE"),
//
//     TIME("TIME"),
//
//     DATETIME("DATETIME"),
//
//     YEAR("YEAR"),
//
//     VARCHAR("VARCHAR"),
//
//     VARBINARY("VARBINARY"),
//
//     BIT("BIT"),
//
//     JSON("JSON"),
//
//     ENUM("ENUM"),
//
//     SET("SET"),
//
//     TINYBLOB("TINYBLOB"),
//
//     TINYTEXT("TINYTEXT"),
//
//     MEDIUMBLOB("MEDIUMBLOB"),
//
//     MEDIUMTEXT("MEDIUMTEXT"),
//
//     LONGBLOB("LONGBLOB"),
//
//     LONGTEXT("LONGTEXT"),
//
//     BLOB("BLOB"),
//
//     TEXT("TEXT"),
//
//     CHAR("CHAR"),
//
//     BINARY("BINARY"),
//
//     GEOMETRY("GEOMETRY"),
//
//     UNKNOWN("UNKNOWN"),
//
//
// }
