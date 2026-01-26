use rand::Rng;

use crate::{DefaultComponent, Error, NullComponent, Result, providers::Name};

/// 文本生成器
pub struct Text<R: Rng> {
    /// 随机数生成器
    rng: R,
}

impl<R: Rng> Text<R> {
    pub fn new(rng: R) -> Self {
        Self { rng }
    }

    /// 生成随机文本
    pub fn random_text(&mut self, min: Option<usize>, max: Option<usize>) -> String {
        let min = min.unwrap_or(10);
        let max = max.unwrap_or(255);
        let length = self.rng.random_range(min..=max);
        let mut text = String::with_capacity(length);
        let mut name = Name::new(rand::rng());
        for _ in 0..length {
            if text.trim().len() > length {
                text = text.trim().to_string();
                text.truncate(length);
                break;
            }
            let word = name.full_name();
            text.push_str(&word);
            text.push(' ');
        }
        text.trim().to_string()
    }
}

/// 文本生成器
#[derive(Debug, Clone)]
pub struct TextGenerator {
    /// 最小字符数
    pub min: Option<usize>,
    /// 最大字符数
    pub max: Option<usize>,

    /// 包含默认值
    pub include_default: Option<DefaultComponent>,
    /// 包含NULL值
    pub include_null: Option<NullComponent>,
}

impl Default for TextGenerator {
    fn default() -> Self {
        Self {
            min: Some(10),
            max: Some(255),
            include_default: None,
            include_null: None,
        }
    }
}

impl TextGenerator {
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        Self {
            min,
            max,
            include_default: None,
            include_null: None,
        }
    }

    pub fn check(&self) -> Result<()> {
        if let Some(min) = self.min
            && let Some(max) = self.max
            && min > max
        {
            return Err(Error::InvalidParameter("最小字符数不能大于最大字符数"));
        }
        let mut percent = 0.0;
        if let Some(dc) = &self.include_default {
            dc.check(None)?;
            percent += dc.percent;
        }
        if let Some(nc) = &self.include_null {
            nc.check()?;
            percent += nc.percent;
        }
        // 检查出现百分比是否大于100
        if percent - 100.0 > 0.0 {
            return Err(Error::PercentNotGreaterThan100);
        }
        Ok(())
    }

    pub fn generate(&mut self, count: usize) -> Result<Vec<Option<String>>> {
        self.check()?;
        let mut res = Vec::with_capacity(count);
        let mut text = Text::new(rand::rng());
        for _ in 0..count {
            // 包含默认值
            if let Some(dc) = &self.include_default {
                // 根据默认值出现百分比判断是否应用默认值
                let include_default = text.rng.random_bool(dc.percent / 100.0);
                if include_default {
                    res.push(Some(dc.default.clone()));
                    continue;
                }
            }
            // 包含NULL值
            if let Some(nc) = &self.include_null {
                // 根据NULL值出现百分比判断是否应用NULL值
                let include_null = text.rng.random_bool(nc.percent / 100.0);
                if include_null {
                    res.push(None);
                    continue;
                }
            }
            // 生成随机文本
            let text = text.random_text(self.min, self.max);
            res.push(Some(text));
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text() {
        let mut text = Text::new(rand::rng());
        let s = text.random_text(None, None);
        assert!(s.len() >= 10 && s.len() <= 255);
        let s = text.random_text(Some(100), Some(100));
        assert!(s.len() <= 100);
    }
}
