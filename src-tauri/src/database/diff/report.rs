use serde::{Deserialize, Serialize};

use super::TableBo;
use crate::database::{
    cores::{ColumnType, Result},
    diff, DatasourceInfo,
};

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
            || !table_info.incre_column.is_empty()
            || !table_info.miss_column.is_empty()
            || !table_info.indexs.is_empty()
            || !table_info.incre_index.is_empty()
            || !table_info.miss_index.is_empty()
            || table_info.comment_change
        {
            changes.push(table_info);
        }
    }

    diff_report.change = changes;
    for (tname, tt) in target_ts.into_iter() {
        if !tt.is_both_has {
            diff_report.incre.push(tname);
        }
    }
    for (sname, st) in source_ts.into_iter() {
        if !st.is_both_has {
            diff_report.miss.push(sname);
        }
    }
    diff_report.change.sort();
    diff_report.incre.sort();
    diff_report.miss.sort();

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

        if tf.r#type != sf.r#type {
            change = true;
            field_info.field_type_change = true;
            field_info.source_field_type = sf.r#type;
            field_info.target_field_type = tf.r#type;
        }

        if tf.length != sf.length {
            change = true;
            field_info.length_change = true;
            field_info.source_length = sf.length;
            field_info.target_length = tf.length;
        }

        if tf.scale != sf.scale {
            change = true;
            field_info.scale_change = true;
            field_info.source_scale = sf.scale;
            field_info.target_scale = tf.scale;
        }

        if (tf.default.is_some() || sf.default.is_some()) && tf.default != sf.default {
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

        if tf.is_null.eq(&sf.is_null) {
            change = true;
            field_info.null_change = true;
            field_info.source_null = sf.is_null;
            field_info.target_null = tf.is_null;
        }

        if tf.is_unsigned.eq(&sf.is_unsigned) {
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
            table_info.incre_column.push(tname.clone());
        }
    }

    for (sname, sf) in source.columns.iter() {
        if !sf.is_both_has {
            table_info.miss_column.push(sname.clone());
        }
    }

    table_info.columns = fields;

    table_info.columns.sort();
    table_info.incre_column.sort();
    table_info.miss_column.sort();
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
            table_info.incre_index.push(iname.clone());
        }
    }

    for (iname, si) in source.indexs.iter() {
        if !si.is_both_has {
            table_info.miss_index.push(iname.clone());
        }
    }

    table_info.indexs = indexs;

    table_info.indexs.sort();
    table_info.incre_index.sort();
    table_info.miss_index.sort();
}

/// 差异报告
#[derive(Debug, Default, Hash, Serialize, Deserialize)]
pub struct DiffReport {
    /// 增加的表
    incre: Vec<String>,
    /// 缺少的表
    miss: Vec<String>,
    /// 变化的表
    change: Vec<TableInfo>,
}

/// 表信息变化
#[derive(Debug, Default, Hash, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableInfo {
    /// 表名
    table_name: String,
    /// 增加的字段
    incre_column: Vec<String>,
    /// 缺少的字段
    miss_column: Vec<String>,
    /// 增加的索引
    incre_index: Vec<String>,
    /// 缺少的索引
    miss_index: Vec<String>,
    /// 是否改过表的描述
    comment_change: bool,
    /// 原表表描述
    source_comment: String,
    /// 目标表表描述
    target_comment: String,
    /// 有改动的列
    columns: Vec<FieldInfo>,
    /// 有改动的索引
    indexs: Vec<IndexInfo>,

    /// 前端是否展开
    close: bool,
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
#[derive(Debug, Default, Hash, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldInfo {
    /// 列名称
    name: String,
    /// 类型是否改变
    field_type_change: bool,
    source_field_type: Option<ColumnType>,
    target_field_type: Option<ColumnType>,
    /// 数据长度是否改变
    length_change: bool,
    source_length: Option<i32>,
    target_length: Option<i32>,
    /// 小数位数是否改变
    scale_change: bool,
    source_scale: Option<i32>,
    target_scale: Option<i32>,
    /// 默认值是否改变
    default_change: bool,
    source_default: Option<String>,
    target_default: Option<String>,
    /// 注释是否改变
    comment_change: bool,
    source_comment: String,
    target_comment: String,
    /// 非空是否改变
    null_change: bool,
    source_null: bool,
    target_null: bool,
    /// 无符号是否改变
    unsigned_change: bool,
    source_unsigned: bool,
    target_unsigned: bool,
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
#[derive(Debug, Default, Hash, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexInfo {
    /// 索引名称
    name: String,
    /// 索引唯一性 是否改变
    non_unique_change: bool,
    source_non_unique: i32,
    target_non_unique: i32,
    /// 作用于列名称 是否改变
    column_name_change: bool,
    target_column_name: String,
    source_column_name: String,
    /// 索引类型 是否改变
    index_type_change: bool,
    source_index_type: String,
    target_index_type: String,
    /// 索引注释 是否改变
    index_comment_change: bool,
    source_index_comment: Option<String>,
    target_index_comment: Option<String>,
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
        let source = DatasourceInfo {
            driver: crate::database::cores::Driver::Mysql,
            name: "127.0.0.1".into(),
            host: "127.0.0.1".into(),
            port: Some(3306),
            username: Some("root".into()),
            password: Some("123456".into()),
            database: Some("test".into()),
        };
        let target = DatasourceInfo {
            driver: crate::database::cores::Driver::Mysql,
            name: "127.0.0.1".into(),
            host: "127.0.0.1".into(),
            port: Some(3306),
            username: Some("root".into()),
            password: Some("123456".into()),
            database: Some("differ".into()),
        };
        let report = diff_report(source, target).await?;
        dbg!(&report);
        println!("{}", report.change.len());
        println!("{}", report.incre.len());
        println!("{}", report.miss.len());

        Ok(())
    }
}
