use database_core::{ColumnType, Driver, error::Result};
use serde::{Deserialize, Serialize};

use super::{FieldBo, IndexBo, TableBo};
use crate::DatasourceInfo;

/// 获取差异报告信息，以source为基准，target变动
pub async fn diff_report(source: DatasourceInfo, target: DatasourceInfo) -> Result<DiffReport> {
    let mut source_ts = super::table_struct(&source).await?;
    let mut target_ts = super::table_struct(&target).await?;

    let mut diff_report = DiffReport::default();
    let mut changes = Vec::<TableInfo>::new();

    for (name, target_table) in target_ts.iter_mut() {
        let Some(source_table) = source_ts.get_mut(name) else {
            continue;
        };
        target_table.is_both_has = true;
        source_table.is_both_has = true;

        let mut table_info = TableInfo::new(name.clone());
        handle_column_change(source_table, target_table, &mut table_info).await;
        handle_index_change(source_table, target_table, &mut table_info).await;

        if !target_table.comment.eq(&source_table.comment) {
            table_info.comment_change = true;
            table_info.source_comment = source_table.comment.clone();
            table_info.target_comment = target_table.comment.clone();
        }
        if !table_info.columns.is_empty()
            || !table_info.incre_columns.is_empty()
            || !table_info.miss_columns.is_empty()
            || !table_info.indexs.is_empty()
            || !table_info.incre_indexs.is_empty()
            || !table_info.miss_indexs.is_empty()
            || table_info.comment_change
        {
            changes.push(table_info);
        }
    }

    diff_report.changes = changes;
    for (tname, tt) in target_ts.into_iter() {
        if !tt.is_both_has {
            diff_report.incres.push(tname);
        }
    }
    for (sname, st) in source_ts.into_iter() {
        if !st.is_both_has {
            diff_report.misses.push(sname);
        }
    }
    diff_report.changes.sort();
    diff_report.incres.sort();
    diff_report.misses.sort();

    Ok(diff_report)
}

async fn handle_column_change(
    source: &mut TableBo,
    target: &mut TableBo,
    table_info: &mut TableInfo,
) {
    if source.columns.is_empty() || target.columns.is_empty() {
        return;
    }

    let mut fields = vec![];
    for (tname, tf) in target.columns.iter_mut() {
        let Some(sf) = source.columns.get_mut(tname) else {
            continue;
        };
        sf.is_both_has = true;
        tf.is_both_has = true;

        let mut field_info = FieldInfo::new(tname.clone());
        let mut change = false;

        if !tf.r#type.eq(&sf.r#type)
            || ((!tf.db_type.is_empty() || !sf.db_type.is_empty()) && !tf.db_type.eq(&sf.db_type))
        {
            change = true;
            field_info.field_type_change = true;
            field_info.source_field_type = sf.r#type;
            field_info.target_field_type = tf.r#type;
        }

        if !tf.length.eq(&sf.length) {
            change = true;
            field_info.length_change = true;
            field_info.source_length = sf.length;
            field_info.target_length = tf.length;
        }

        if !tf.scale.eq(&sf.scale) {
            change = true;
            field_info.scale_change = true;
            field_info.source_scale = sf.scale;
            field_info.target_scale = tf.scale;
        }

        if (tf.default.is_some() || sf.default.is_some()) && !tf.default.eq(&sf.default) {
            change = true;
            field_info.default_change = true;
            field_info.source_default = sf.default.clone();
            field_info.target_default = tf.default.clone();
        }

        if !tf.comment.eq(&sf.comment) {
            change = true;
            field_info.comment_change = true;
            field_info.source_comment = sf.comment.clone();
            field_info.target_comment = tf.comment.clone();
        }

        if !tf.is_null.eq(&sf.is_null) {
            change = true;
            field_info.null_change = true;
            field_info.source_null = sf.is_null;
            field_info.target_null = tf.is_null;
        }

        if !tf.is_unsigned.eq(&sf.is_unsigned) {
            change = true;
            field_info.unsigned_change = true;
            field_info.source_unsigned = sf.is_unsigned;
            field_info.target_unsigned = tf.is_unsigned;
        }

        if change {
            fields.push(field_info);
        }
    }

    for (tname, tf) in target.columns.iter() {
        if !tf.is_both_has {
            table_info.incre_columns.push(tname.clone());
        }
    }

    for (sname, sf) in source.columns.iter() {
        if !sf.is_both_has {
            table_info.miss_columns.push(sname.clone());
        }
    }

    table_info.columns = fields;

    table_info.columns.sort();
    table_info.incre_columns.sort();
    table_info.miss_columns.sort();
}

async fn handle_index_change(
    source: &mut TableBo,
    target: &mut TableBo,
    table_info: &mut TableInfo,
) {
    if source.indexs.is_empty() || target.indexs.is_empty() {
        return;
    }
    let mut indexs = vec![];
    for (iname, ti) in target.indexs.iter_mut() {
        let Some(si) = source.indexs.get_mut(iname) else {
            continue;
        };
        si.is_both_has = true;
        ti.is_both_has = true;

        let mut index_info = IndexInfo::new(iname.clone());
        let mut change = false;

        if !ti.non_unique.eq(&si.non_unique) {
            change = true;
            index_info.non_unique_change = true;
            index_info.source_non_unique = si.non_unique;
            index_info.target_non_unique = ti.non_unique;
        }

        if !ti.column_name.eq(&si.column_name) {
            change = true;
            index_info.column_name_change = true;
            index_info.source_column_name = si.column_name.clone();
            index_info.target_column_name = ti.column_name.clone();
        }

        if !ti.index_type.eq(&si.index_type) {
            change = true;
            index_info.index_type_change = true;
            index_info.source_index_type = si.index_type.clone();
            index_info.target_index_type = ti.index_type.clone();
        }

        if !ti.index_comment.eq(&si.index_comment) {
            change = true;
            index_info.index_comment_change = true;
            index_info.source_index_comment = si.index_comment.clone();
            index_info.target_index_comment = ti.index_comment.clone();
        }

        if change {
            indexs.push(index_info);
        }
    }

    for (iname, ti) in target.indexs.iter() {
        if !ti.is_both_has {
            table_info.incre_indexs.push(iname.clone());
        }
    }

    for (iname, si) in source.indexs.iter() {
        if !si.is_both_has {
            table_info.miss_indexs.push(iname.clone());
        }
    }

    table_info.indexs = indexs;

    table_info.indexs.sort();
    table_info.incre_indexs.sort();
    table_info.miss_indexs.sort();
}

/// 获取结构差异SQL，以source为基准，target变动
pub async fn diff_sql(source: DatasourceInfo, target: DatasourceInfo) -> Result<Vec<String>> {
    let source_ts = super::table_struct(&source).await?;
    let target_ts = super::table_struct(&target).await?;

    let mut res = vec![];
    for (sname, source_table) in source_ts.iter() {
        let target_table = target_ts.get(sname);
        let sqls = diff_table(source_table, target_table, &source).await?;
        res.push(sqls);
    }

    Ok(res.into_iter().flatten().collect::<Vec<_>>())
}

async fn diff_table(
    source: &TableBo,
    target: Option<&TableBo>,
    source_di: &DatasourceInfo,
) -> Result<Vec<String>> {
    let Some(target) = target else {
        let mut sql = super::create_table_sql(source_di, &source.name).await?;
        sql.push(';');
        return Ok(vec![sql]);
    };

    let mut fix_sql = Vec::new();
    let mut last_column = "";
    for (sname, sf) in source.columns.iter() {
        let tf = target.columns.get(sname);
        if let Some(tf) = tf
            && tf.eq(sf)
        {
            continue;
        }

        fix_sql.push(diff_column(sf, tf, last_column, source_di.driver).await);
        last_column = sname;
    }

    for (iname, si) in source.indexs.iter() {
        let ti = target.indexs.get(iname);
        let fix_index_sql = if source_di.driver == Driver::Postgres {
            diff_index_postgres(si, ti).await
        } else {
            diff_index(si, ti).await
        };
        if !fix_index_sql.is_empty() {
            fix_sql.push(fix_index_sql);
        }
    }

    Ok(fix_sql)
}

async fn diff_column(
    sf: &FieldBo,
    tf: Option<&FieldBo>,
    last_column: &str,
    driver: Driver,
) -> String {
    if driver == Driver::Postgres {
        return diff_column_postgres(sf, tf).await;
    }

    let mut fix_sql = String::new();
    if tf.is_none() {
        fix_sql.push_str(&format!(
            "alter table `{}` add `{}` {}",
            sf.table_name,
            sf.name,
            sf.r#type.unwrap()
        ));
    } else {
        fix_sql.push_str(&format!(
            "alter table `{}` modify `{}` {}",
            sf.table_name,
            sf.name,
            sf.r#type.unwrap()
        ));
    }

    if let Some(len) = sf.length
        && len > 0
    {
        fix_sql.push('(');
        fix_sql.push_str(len.to_string().as_str());
        if let Some(scale) = sf.scale
            && scale > 0
        {
            fix_sql.push(',');
            fix_sql.push_str(scale.to_string().as_str());
        }

        fix_sql.push(')');
    }

    if sf.is_unsigned {
        fix_sql.push_str(" unsigned ");
    }

    if sf.is_auto_incr {
        fix_sql.push_str(" not null auto_increment ");
    } else if !sf.is_null {
        fix_sql.push_str(" not null ");
    }

    if let Some(default) = &sf.default {
        if let Some(t) = sf.r#type {
            match t {
                ColumnType::Char | ColumnType::VarChar => {
                    fix_sql.push_str(" default ");
                    fix_sql.push('\'');
                    fix_sql.push_str(default);
                    fix_sql.push('\'');
                }
                _ => {
                    fix_sql.push_str(" default ");
                    fix_sql.push_str(default);
                }
            }
        }
    } else if !sf.is_auto_incr || !sf.is_null {
        fix_sql.push_str(" default null ");
    }

    if !sf.comment.is_empty() {
        fix_sql.push_str(" comment ");
        fix_sql.push('\'');
        fix_sql.push_str(&sf.comment);
        fix_sql.push('\'');
    }

    if last_column.is_empty() {
        fix_sql.push_str(" first ");
    } else {
        fix_sql.push_str(&format!(" after `{last_column}` "));
    }

    fix_sql.push(';');

    fix_sql
}

async fn diff_column_postgres(sf: &FieldBo, tf: Option<&FieldBo>) -> String {
    let (schema, table) = super::split_pg_table_name(&sf.table_name);
    let table_name = super::quote_pg_table(schema, table);
    let column_name = super::quote_pg_ident(&sf.name);

    if tf.is_none() {
        let mut sql = format!(
            "ALTER TABLE {} ADD COLUMN {}",
            table_name,
            super::pg_column_definition(sf)
        );
        if !sf.comment.is_empty() {
            sql.push_str(&format!(
                ";\nCOMMENT ON COLUMN {}.{} IS '{}'",
                table_name,
                column_name,
                super::escape_sql_string(&sf.comment)
            ));
        }
        sql.push(';');
        return sql;
    }

    let mut statements = vec![format!(
        "ALTER TABLE {} ALTER COLUMN {} TYPE {}",
        table_name,
        column_name,
        super::pg_column_db_type(sf)
    )];

    if sf.is_null {
        statements.push(format!(
            "ALTER TABLE {} ALTER COLUMN {} DROP NOT NULL",
            table_name, column_name
        ));
    } else {
        statements.push(format!(
            "ALTER TABLE {} ALTER COLUMN {} SET NOT NULL",
            table_name, column_name
        ));
    }

    if let Some(default) = &sf.default {
        statements.push(format!(
            "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT {}",
            table_name, column_name, default
        ));
    } else {
        statements.push(format!(
            "ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT",
            table_name, column_name
        ));
    }

    statements.push(format!(
        "COMMENT ON COLUMN {}.{} IS '{}'",
        table_name,
        column_name,
        super::escape_sql_string(&sf.comment)
    ));

    statements
        .into_iter()
        .map(|sql| format!("{sql};"))
        .collect::<Vec<_>>()
        .join("\n")
}

async fn diff_index(si: &IndexBo, ti: Option<&IndexBo>) -> String {
    let mut fix_index_sql = String::from(" add ");
    if "FULLTEXT".eq(&si.index_type) {
        fix_index_sql.push_str(" fulltext ");
    } else if "PRIMARY".eq(&si.key_name) {
        fix_index_sql.push_str(" primary key ");
    } else if si.non_unique == 0 {
        fix_index_sql.push_str(" unique index ");
    } else {
        fix_index_sql.push_str(" index ");
    }

    fix_index_sql.push_str(if "PRIMARY".eq(&si.key_name) {
        ""
    } else {
        &si.key_name
    });

    fix_index_sql.push('(');
    fix_index_sql.push_str(&si.column_name);
    fix_index_sql.push(')');

    if "BTREE".eq(&si.index_type) && !"PRIMARY".eq(&si.key_name) && si.non_unique == 1 {
        fix_index_sql.push_str(" using btree ");
    }

    fix_index_sql.push_str(&format!(" comment `{}`;", si.index_comment));

    if let Some(ti) = ti {
        if !ti.eq(si) {
            format!(
                "alter table `{}` drop index {} , {fix_index_sql}",
                si.table_name, si.key_name
            )
        } else {
            String::new()
        }
    } else {
        format!("alter table `{}` {fix_index_sql}", si.table_name)
    }
}

async fn diff_index_postgres(si: &IndexBo, ti: Option<&IndexBo>) -> String {
    let (schema, _) = super::split_pg_table_name(&si.table_name);

    if si.key_name == "PRIMARY" {
        return String::new();
    }

    let create_index_sql = super::pg_create_index_sql(si);
    if let Some(ti) = ti {
        if ti.eq(si) {
            String::new()
        } else {
            format!(
                "DROP INDEX IF EXISTS {}.{};\n{};",
                super::quote_pg_ident(schema),
                super::quote_pg_ident(&si.key_name),
                create_index_sql
            )
        }
    } else {
        format!("{create_index_sql};")
    }
}

/// 差异报告
#[derive(Debug, Clone, Default, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffReport {
    /// 增加的表
    pub incres: Vec<String>,
    /// 缺少的表
    pub misses: Vec<String>,
    /// 变化的表
    pub changes: Vec<TableInfo>,
}

/// 表信息变化
#[derive(Debug, Clone, Default, Hash, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct TableInfo {
    /// 表名
    pub table_name: String,
    /// 增加的字段
    pub incre_columns: Vec<String>,
    /// 缺少的字段
    pub miss_columns: Vec<String>,
    /// 增加的索引
    pub incre_indexs: Vec<String>,
    /// 缺少的索引
    pub miss_indexs: Vec<String>,
    /// 是否改过表的描述
    pub comment_change: bool,
    /// 原表表描述
    pub source_comment: String,
    /// 目标表表描述
    pub target_comment: String,
    /// 有改动的列
    pub columns: Vec<FieldInfo>,
    /// 有改动的索引
    pub indexs: Vec<IndexInfo>,

    /// 前端是否展开
    pub close: bool,
}

impl TableInfo {
    fn new(name: String) -> Self {
        Self {
            table_name: name,
            ..Default::default()
        }
    }
}

/// 列信息变化
#[derive(Debug, Clone, Default, Hash, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct FieldInfo {
    /// 列名称
    pub name: String,
    /// 类型是否改变
    pub field_type_change: bool,
    pub source_field_type: Option<ColumnType>,
    pub target_field_type: Option<ColumnType>,
    /// 数据长度是否改变
    pub length_change: bool,
    pub source_length: Option<i32>,
    pub target_length: Option<i32>,
    /// 小数位数是否改变
    pub scale_change: bool,
    pub source_scale: Option<i32>,
    pub target_scale: Option<i32>,
    /// 默认值是否改变
    pub default_change: bool,
    pub source_default: Option<String>,
    pub target_default: Option<String>,
    /// 注释是否改变
    pub comment_change: bool,
    pub source_comment: String,
    pub target_comment: String,
    /// 非空是否改变
    pub null_change: bool,
    pub source_null: bool,
    pub target_null: bool,
    /// 无符号是否改变
    pub unsigned_change: bool,
    pub source_unsigned: bool,
    pub target_unsigned: bool,
}

impl FieldInfo {
    fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

/// 索引信息变化
#[derive(Debug, Clone, Default, Hash, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct IndexInfo {
    /// 索引名称
    pub name: String,
    /// 索引唯一性 是否改变
    pub non_unique_change: bool,
    pub source_non_unique: i32,
    pub target_non_unique: i32,
    /// 作用于列名称 是否改变
    pub column_name_change: bool,
    pub target_column_name: String,
    pub source_column_name: String,
    /// 索引类型 是否改变
    pub index_type_change: bool,
    pub source_index_type: String,
    pub target_index_type: String,
    /// 索引注释 是否改变
    pub index_comment_change: bool,
    pub source_index_comment: String,
    pub target_index_comment: String,
}

impl IndexInfo {
    fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_diff_report() -> Result<()> {
        sqlx::any::install_default_drivers();
        let source = DatasourceInfo {
            driver: Driver::Mysql,
            name: "127.0.0.1".into(),
            host: "127.0.0.1".into(),
            port: Some(3306),
            username: Some("root".into()),
            password: Some("123456".into()),
            database: Some("test".into()),
        };
        let target = DatasourceInfo {
            driver: Driver::Mysql,
            name: "127.0.0.1".into(),
            host: "127.0.0.1".into(),
            port: Some(3306),
            username: Some("root".into()),
            password: Some("123456".into()),
            database: Some("differ".into()),
        };

        let report = diff_report(source, target).await?;
        dbg!(&report);
        println!("{}", report.changes.len());
        println!("{}", report.incres.len());
        println!("{}", report.misses.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_diff_sql() -> Result<()> {
        sqlx::any::install_default_drivers();
        let source = DatasourceInfo {
            driver: Driver::Mysql,
            name: "127.0.0.1".into(),
            host: "127.0.0.1".into(),
            port: Some(3306),
            username: Some("root".into()),
            password: Some("123456".into()),
            database: Some("test".into()),
        };
        let target = DatasourceInfo {
            driver: Driver::Mysql,
            name: "127.0.0.1".into(),
            host: "127.0.0.1".into(),
            port: Some(3306),
            username: Some("root".into()),
            password: Some("123456".into()),
            database: Some("differ".into()),
        };

        let diff_sql = diff_sql(source, target).await?;
        dbg!(&diff_sql);

        Ok(())
    }
}
