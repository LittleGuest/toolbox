use std::collections::HashSet;

use crate::{Error, Result};

mod number;
mod sequence;

pub use number::NumberGenerator;
pub use sequence::SequenceGenerator;

/// 默认值组件
#[derive(Debug, Clone)]
pub struct DefaultComponent {
    /// 默认值
    pub default: String,
    /// 默认值出现百分比
    pub percent: u8,
}

impl DefaultComponent {
    pub fn new(default: String, percent: u8) -> Self {
        Self { default, percent }
    }

    /// 检查默认值组件参数  
    pub fn check(&self, len: Option<usize>) -> Result<()> {
        if self.percent > 100 {
            return Err(Error::E("默认值出现百分比不能超过100".into()));
        }
        if let Some(len) = len
            && self.default.len() > len
        {
            return Err(Error::E("默认值长度不能超过字段长度".into()));
        }
        Ok(())
    }
}

/// NULL值组件
#[derive(Debug, Clone)]
pub struct NullComponent {
    /// NULL值出现百分比
    pub percent: u8,
}

impl NullComponent {
    pub fn new(percent: u8) -> Self {
        Self { percent }
    }

    /// 检查NULL值组件参数
    pub fn check(&self) -> Result<()> {
        if self.percent > 100 {
            return Err(Error::E("NULL值出现百分比不能超过100".into()));
        }
        Ok(())
    }
}

/// 唯一值组件
#[derive(Debug, Clone)]
pub struct UniqueComponent {
    /// 已经生成的唯一值
    pub value: HashSet<String>,
}

impl UniqueComponent {
    pub fn new() -> Self {
        Self {
            value: HashSet::new(),
        }
    }
}
