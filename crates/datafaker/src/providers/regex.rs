use rand::prelude::*;

use crate::{DefaultComponent, Error, NullComponent, Result, UniqueComponent};

pub struct Regex<R: Rng> {
    rng: R,
    pub pattern: String,
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

    pub fn random(&mut self, count: usize) -> Result<Vec<String>> {
        let mut parser = regex_syntax::ParserBuilder::new().unicode(false).build();
        let hir = parser
            .parse(&self.pattern)
            .map_err(|_| Error::RegexSyntax)?;
        let reg =
            rand_regex::Regex::with_hir(hir, self.max_repeat).map_err(|_| Error::RegexGenerator)?;
        let samples = (&mut self.rng)
            .sample_iter(&reg)
            .take(count)
            .collect::<Vec<String>>();
        Ok(samples)
    }
}

#[derive(Debug, Clone)]
pub struct RegexGenerator {
    pub pattern: String,

    pub include_default: Option<DefaultComponent>,
    pub include_null: Option<NullComponent>,
    pub unique: Option<UniqueComponent>,
    pub forbidden_links: bool,
}

impl Default for RegexGenerator {
    fn default() -> Self {
        Self {
            pattern: "[A-Za-z0-9]{10}".into(),
            include_default: None,
            include_null: None,
            unique: None,
            forbidden_links: false,
        }
    }
}

impl RegexGenerator {
    pub fn new(
        pattern: String,
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
        regex_syntax::parse(&self.pattern).map_err(|_| Error::RegexSyntax)?;
        let mut percent = 0.0;
        if let Some(dc) = &self.include_default {
            dc.check(None)?;
            percent += dc.percent;
        }
        if let Some(nc) = &self.include_null {
            nc.check()?;
            percent += nc.percent;
        }
        if percent - 100.0 > 0.0 {
            return Err(Error::PercentNotGreaterThan100);
        }
        Ok(())
    }

    pub fn generate(&mut self, count: usize) -> Result<Vec<Option<String>>> {
        self.check()?;
        let mut res = Vec::with_capacity(count);
        let mut regex = Regex::new(rand::rng(), self.pattern.clone(), 1);
        let samples = regex.random(count)?;
        for item in samples.iter().take(count) {
            if let Some(dc) = &self.include_default {
                let include_default = regex.rng.random_bool(dc.percent / 100.0);
                if include_default {
                    res.push(Some(dc.default.clone()));
                    continue;
                }
            }
            if let Some(nc) = &self.include_null {
                let include_null = regex.rng.random_bool(nc.percent / 100.0);
                if include_null {
                    res.push(None);
                    continue;
                }
            }
            res.push(Some(item.clone()));
        }
        Ok(res)
    }

    pub fn preview(&self) -> Result<String> {
        let mut regex = Regex::new(rand::rng(), self.pattern.clone(), 1);
        let samples = regex.random(1)?;
        Ok(samples[0].clone())
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
            let pattern = "[A-Za-z0-9]{10}".into();
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
            let pattern = "[A-Za-z0-9]{10}".into();
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
            let pattern = "[A-Za-z0-9]{10}".into();
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
            let pattern = "[A-Za-z0-9]{x}".into();
            RegexGenerator::new(pattern, None, None, None, false).unwrap();
        }
    }
}
