use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum StandardCheck {
    /// 检查小写
    NameContainUpperCase = 11,
    /// 检查单词拼写
    NameErrorSpell = 12,
    /// 检查数字开头
    NameDigitStart = 13,
    /// 检查保留字
    NameUseKeyword = 14,

    /// 检查复数单词
    TableNameContainPlurality = 31,
    /// 检查必备字段
    TableMissField = 32,

    /// 检查索引命名。主键索引名为 pk_字段名、唯一索引名为 uk_字段名、普通索引名则为 idx_字段名
    IndexNameError = 33,

    /// 检查is开头
    FieldIsStartErrorType = 41,
    /// 检查is开头
    FieldIsStartErrorComment = 42,
    /// 检查“是否”类型的字段
    FieldIsContainComment = 43,
    /// 检查小数类型
    FieldTypeUseFloat = 44,
}

impl StandardCheck {
    pub fn desc(&self) -> &str {
        match self {
            StandardCheck::NameContainUpperCase => "应当使用小写",
            StandardCheck::NameErrorSpell => "单词拼写建议，原词：{}，建议词：{}",
            StandardCheck::NameDigitStart => "不能数字开头，原词：{}",
            StandardCheck::NameUseKeyword => "禁用保留字",
            StandardCheck::TableNameContainPlurality => {
                "表名不要使用复数名词，原词：{}，建议词：{}"
            }
            StandardCheck::TableMissField => "表缺少必备三字段：id, create_time, update_time。",
            StandardCheck::IndexNameError => "索引命名不规范，原索引名：{}",
            StandardCheck::FieldIsStartErrorType => "字段是is开头，但类型不是unsigned tinyint",
            StandardCheck::FieldIsStartErrorComment => "字段是is开头，但字段备注没有包含“是否”二字",
            StandardCheck::FieldIsContainComment => "字段备注包含“是否”二字，但字段名称不是is开头",
            StandardCheck::FieldTypeUseFloat => "小数类型为 decimal，禁止使用 float 和 double",
        }
    }
}
