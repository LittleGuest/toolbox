//! 数字生成器

use super::{DefaultComponent, NullComponent, UniqueComponent};
use crate::{Error, Result};

/// 数字生成器
#[derive(Debug, Clone)]
pub struct NumberGenerator {
    /// 开始值
    pub start: i64,
    /// 结束值
    pub end: i64,
    /// 小数位数
    pub decimal_places: u8,
    /// 包含默认值
    pub include_default: Option<DefaultComponent>,
    /// 包含NULL值
    pub include_null: Option<NullComponent>,
    /// 唯一值
    pub unique: Option<UniqueComponent>,
    /// 禁用链接
    pub forbidden_links: bool,
}

impl Default for NumberGenerator {
    fn default() -> Self {
        Self {
            start: 0,
            end: 1000,
            decimal_places: 0,
            include_default: None,
            include_null: None,
            unique: None,
            forbidden_links: false,
        }
    }
}

impl NumberGenerator {
    /// 数字生成器名称
    pub fn name() -> &'static str {
        "数字生成器"
    }

    /// 创建数字生成器
    pub fn new(
        start: i64,
        end: i64,
        decimal_places: u8,
        include_default: Option<DefaultComponent>,
        include_null: Option<NullComponent>,
        unique: Option<UniqueComponent>,
        forbidden_links: bool,
    ) -> Result<Self> {
        let ng = Self {
            start,
            end,
            decimal_places,
            include_default,
            include_null,
            unique,
            forbidden_links,
        };
        ng.check()?;
        Ok(ng)
    }

    /// 检查数字生成器参数
    pub fn check(&self) -> Result<()> {
        if self.start > self.end {
            return Err(Error::E("开始值不能大于结束值".into()));
        }
        if self.decimal_places > 10 {
            return Err(Error::E("小数位数不能超过10位".into()));
        }
        if let Some(dc) = &self.include_default {
            dc.check(None)?;
        }
        if let Some(nc) = &self.include_null {
            nc.check()?;
        }
        Ok(())
    }

    /// 生成数字，num为生成数量
    pub fn generate(&mut self, count: usize) -> Result<Vec<Option<String>>> {
        self.check()?;
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            // 包含默认值
            if let Some(dc) = &self.include_default {
                dc.check(None)?;
                // 根据默认值出现百分比判断是否应用默认值
                if self.should_apply_percent(dc.percent) {
                    res.push(Some(dc.default.clone()));
                    continue;
                }
            }
            // 包含NULL值
            if let Some(nc) = &self.include_null {
                nc.check()?;
                // 根据NULL值出现百分比判断是否应用NULL值
                if self.should_apply_percent(nc.percent) {
                    res.push(None);
                    continue;
                }
            }
            let mut num = fastrand::i64(self.start..=self.end);
            // 校验唯一值
            if let Some(unique) = &mut self.unique {
                if unique.value.contains(num.to_string().as_str()) {
                    loop {
                        // 重新生成
                        num = fastrand::i64(self.start..=self.end);
                        if !unique.value.iter().any(|v| v.eq(&num.to_string())) {
                            unique.value.insert(num.to_string());
                            break;
                        }
                    }
                } else {
                    unique.value.insert(num.to_string());
                }
            }

            if self.decimal_places > 0 {
                let num = num as f64;
                res.push(Some(num.round().to_string()));
            } else {
                res.push(Some(num.to_string()));
            }
        }
        Ok(res)
    }

    /// 检查是否应用百分比
    fn should_apply_percent(&self, percent: u8) -> bool {
        fastrand::u8(0..100) < percent
    }

    // 重置生成器状态（用于批量生成时重置唯一值集合）
    pub fn reset(&mut self) {
        if let Some(unique) = &mut self.unique {
            unique.value.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_generation() {
        let mut generator = NumberGenerator::new(0, 100, 0, None, None, None, false).unwrap();
        let res = generator.generate(5).unwrap();
        assert_eq!(res.len(), 5);
    }

    #[test]
    fn test_default_values() {
        let mut generator = NumberGenerator::new(
            0,
            100,
            0,
            Some(DefaultComponent::new("100".into(), 5)),
            None,
            None,
            false,
        )
        .unwrap();
        let res = generator.generate(5).unwrap();
        assert_eq!(res.len(), 5);
    }

    #[test]
    fn test_null_values() {
        let mut generator =
            NumberGenerator::new(0, 100, 0, None, Some(NullComponent::new(50)), None, false)
                .unwrap();
        let res = generator.generate(1000).unwrap();
        assert_eq!(res.len(), 1000);
        let null_count = res.iter().filter(|v| v.is_none()).count();
        // 大约50%的概率生成NULL
        assert!(null_count > 400 && null_count < 600);
    }

    #[test]
    fn test_unique_values() {
        let mut generator =
            NumberGenerator::new(1, 5, 0, None, None, Some(UniqueComponent::new()), false).unwrap();

        let mut res = generator.generate(5).unwrap();
        assert_eq!(res.len(), 5);
        res.sort();
        assert!(res.eq(&vec![
            Some("1".to_string()),
            Some("2".to_string()),
            Some("3".to_string()),
            Some("4".to_string()),
            Some("5".to_string()),
        ]));
    }
}
