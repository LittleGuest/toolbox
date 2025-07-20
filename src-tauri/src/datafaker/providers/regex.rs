//! 正则表达式随机字符串生成器
//! 根据指定的正则表达式模式生成符合规则的随机字符串

use rand::prelude::*;

use crate::datafaker::{DefaultComponent, Error, NullComponent, Result, UniqueComponent};

/// 正则表达式随机字符串生成器
pub struct Regex<R: Rng> {
    /// 随机数生成器
    rng: R,
    /// 正则表达式模式
    pub pattern: String,
    /// 最大重复次数
    pub max_repeat: u32,
}

impl Default for Regex<ThreadRng> {
    fn default() -> Self {
        Self::new(rand::rng(), "".to_string(), 1)
    }
}

impl<R: Rng> Regex<R> {
    pub fn new(rng: R, pattern: String, max_repeat: u32) -> Self {
        Self {
            rng,
            pattern,
            max_repeat,
        }
    }

    /// 生成随机字符串，count为字符串数量
    pub fn random(&mut self, count: usize) -> Result<Vec<String>> {
        let mut parser = regex_syntax::ParserBuilder::new().unicode(false).build();
        let hir = parser.parse(&self.pattern)?;
        let reg = rand_regex::Regex::with_hir(hir, self.max_repeat)?;
        let samples = (&mut self.rng)
            .sample_iter(&reg)
            .take(count)
            .collect::<Vec<String>>();
        Ok(samples)
    }
}

/// 正则表达式随机字符串生成器
#[derive(Debug, Clone)]
pub struct RegexGenerator {
    /// 正则表达式模式
    pub pattern: &'static str,

    /// 包含默认值
    pub include_default: Option<DefaultComponent>,
    /// 包含NULL值
    pub include_null: Option<NullComponent>,
    /// 唯一值
    pub unique: Option<UniqueComponent>,
    /// 禁用链接
    pub forbidden_links: bool,
}

impl RegexGenerator {
    pub fn new(
        pattern: &'static str,
        include_default: Option<DefaultComponent>,
        include_null: Option<NullComponent>,
        unique: Option<UniqueComponent>,
        forbidden_links: bool,
    ) -> Result<Self> {
        let regex = Self {
            pattern,
            include_default,
            include_null,
            unique,
            forbidden_links,
        };
        regex.check()?;
        Ok(regex)
    }

    pub fn check(&self) -> Result<()> {
        regex_syntax::parse(&self.pattern)?;
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

    /// 生成随机字符串，count为字符串数量
    pub fn generate(&mut self, count: usize) -> Result<Vec<Option<String>>> {
        self.check()?;
        let mut res = Vec::with_capacity(count);
        let mut regex = Regex::new(rand::rng(), self.pattern.into(), 1);
        let samples = regex.random(count)?;
        for i in 0..count {
            // 包含默认值
            if let Some(dc) = &self.include_default {
                // 根据默认值出现百分比判断是否应用默认值
                let include_default = regex.rng.random_bool(dc.percent / 100.0);
                if include_default {
                    res.push(Some(dc.default.clone()));
                    continue;
                }
            }
            // 包含NULL值
            if let Some(nc) = &self.include_null {
                // 根据NULL值出现百分比判断是否应用NULL值
                let include_null = regex.rng.random_bool(nc.percent / 100.0);
                if include_null {
                    res.push(None);
                    continue;
                }
            }
            // 不包含默认值和NULL值
            // // 如果是原始数据模式，直接返回生成的字符串
            // if self.config.raw_data_mode {
            //     return Ok(Some(result));
            // }
            res.push(Some(samples[i].clone()));
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        let pattern = r"[A-Za-z0-9]{10}";
        let mut regex = Regex::new(rand::rng(), pattern.into(), 1);
        let samples = regex.random(10);
        assert!(samples.is_ok());
        let s = samples.unwrap();
        assert_eq!(s.len(), 10);
        assert!(s.iter().all(|x| x.len() == 10));
        let rgx = regex::Regex::new(pattern).unwrap();
        assert!(s.iter().all(|x| rgx.is_match(x)));
    }

    mod generator {
        use super::*;

        #[test]
        fn test_regex_generator_basic() {
            let pattern = r"[A-Za-z0-9]{10}";
            let generator = RegexGenerator::new(pattern, None, None, None, false);
            assert!(generator.is_ok());
            let mut generator = generator.unwrap();
            let result = generator.generate(10);
            assert!(result.is_ok());
            let s = result.unwrap();
            assert_eq!(s.len(), 10);
        }

        #[test]
        fn test_regex_generator_with_null() {
            // 配置100%生成NULL值
            let pattern = r"[A-Za-z0-9]{10}";
            let generator =
                RegexGenerator::new(pattern, None, Some(NullComponent::new(100.0)), None, false);
            assert!(generator.is_ok());
            let mut generator = generator.unwrap();
            let result = generator.generate(10);
            assert!(result.is_ok());
            let s = result.unwrap();
            assert_eq!(s.len(), 10);
            assert!(s.iter().all(|x| x.is_none()));
        }

        #[test]
        fn test_regex_generator_with_default() {
            // 配置100%生成默认值
            let pattern = r"[A-Za-z0-9]{10}";
            let generator = RegexGenerator::new(
                pattern,
                Some(DefaultComponent::new("DEFAULT_VALUE".to_string(), 100.0)),
                None,
                None,
                false,
            );
            assert!(generator.is_ok());
            let mut generator = generator.unwrap();
            let result = generator.generate(10);
            assert!(result.is_ok());
            let s = result.unwrap();
            assert_eq!(s.len(), 10);
            assert!(s.iter().all(|x| x.eq(&Some("DEFAULT_VALUE".to_string()))));
        }

        #[test]
        #[should_panic]
        fn test_invalid_patterns() {
            let pattern = r"[A-Za-z0-9]{x}";
            RegexGenerator::new(pattern, None, None, None, false).unwrap();
        }
    }
}
