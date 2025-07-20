//! 数字生成器
//! 提供各种随机数字生成功能，包括整数、小数和数字字符串

use rand::prelude::*;

use crate::datafaker::{DefaultComponent, Error, NullComponent, Result, UniqueComponent};

/// 数字Provider
pub struct Number<R: Rng> {
    /// 随机数生成器
    rng: R,
}

impl Default for Number<ThreadRng> {
    fn default() -> Self {
        Self { rng: rand::rng() }
    }
}

impl<R: Rng> Number<R> {
    pub fn new(rng: R) -> Self {
        Self { rng }
    }

    /// 返回[0-9]的随机数字
    pub fn random_digit(&mut self) -> u8 {
        self.rng.random_range(0..=9)
    }

    /// 返回[1-9]的随机数字
    pub fn random_digit_not_zero(&mut self) -> u8 {
        self.rng.random_range(1..=9)
    }

    /// 返回一个正整数
    pub fn positive(&mut self) -> i32 {
        self.rng.random_range(1..=i32::MAX)
    }

    /// 返回一个负整数
    pub fn negative(&mut self) -> i32 {
        self.rng.random_range(i32::MIN..=-1)
    }

    /// 生成指定[min, max)范围内的随机整数
    pub fn number_between_i32(&mut self, min: i32, max: i32) -> i32 {
        if min == max {
            return min;
        }
        let (real_min, real_max) = if min < max { (min, max) } else { (max, min) };
        self.rng.random_range(real_min..real_max)
    }

    /// 生成指定[min, max)范围内的随机双精度浮点数
    pub fn number_between_f64(&mut self, min: f64, max: f64) -> f64 {
        min + (fastrand::f64() * (max - min))
    }

    /// 生成指定[min, max)范围内的随机长整数
    pub fn number_between_i64(&mut self, min: i64, max: i64) -> i64 {
        if min == max {
            return min;
        }
        let (real_min, real_max) = if min < max { (min, max) } else { (max, min) };
        self.rng.random_range(real_min..real_max)
    }

    /// 生成指定位数的随机数字
    /// number_of_digits - 数字位数
    pub fn random_number(&mut self, number_of_digits: u32) -> u64 {
        if number_of_digits == 0 {
            return 0;
        }

        let min = 10u64.pow(number_of_digits - 1);
        let max = 10u64.pow(number_of_digits);
        self.rng.random_range(min..max)
    }

    /// 生成随机长整数
    pub fn random_long(&mut self) -> i64 {
        self.rng.random_range(i64::MIN..=i64::MAX)
    }

    /// 生成指定小数位数的随机双精度浮点数
    /// number_of_decimals - 小数位数
    /// min - 最小值
    /// max - 最大值
    pub fn random_double(&mut self, _number_of_decimals: u32, min: i64, max: i64) -> f64 {
        let decimal = self.number_between_f64(min as f64, max as f64);
        // FIXME: 小数位数 number_of_decimals
        decimal.round()
    }

    /// 生成指定count长度的随机数字字符串
    pub fn digits(&mut self, count: usize) -> String {
        let mut result = String::with_capacity(count);
        for _ in 0..count {
            let digit = self.random_digit();
            result.push_str(&digit.to_string());
        }
        result
    }

    /// 生成单个随机数字的字符串
    pub fn digit(&mut self) -> String {
        self.digits(1)
    }
}

/// 数字生成器
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
            return Err(Error::StartNotGreaterThanEnd);
        }
        if self.decimal_places > 10 {
            return Err(Error::DecimalPlacesNotGreaterThan10);
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

    /// 生成数字，count为生成数量
    pub fn generate(&mut self, count: usize) -> Result<Vec<Option<String>>> {
        self.check()?;
        let mut res = Vec::with_capacity(count);
        let mut rng = rand::rng();
        for _ in 0..count {
            // 包含默认值
            if let Some(dc) = &self.include_default {
                // 根据默认值出现百分比判断是否应用默认值
                let include_default = rng.random_bool(dc.percent / 100.0);
                if include_default {
                    res.push(Some(dc.default.clone()));
                    continue;
                }
            }
            // 包含NULL值
            if let Some(nc) = &self.include_null {
                // 根据NULL值出现百分比判断是否应用NULL值
                let include_null = rng.random_bool(nc.percent / 100.0);
                if include_null {
                    res.push(None);
                    continue;
                }
            }
            // 不包含默认值和NULL值
            let num_fn = || fastrand::i64(self.start..=self.end);
            let mut num = num_fn();
            // 校验唯一值
            if let Some(unique) = &mut self.unique {
                if unique.value.contains(num.to_string().as_str()) {
                    loop {
                        // 重新生成
                        num = num_fn();
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_digit() {
        let mut provider = Number::default();
        let digit = provider.random_digit();
        assert!(digit <= 9);
    }

    #[test]
    fn test_random_digit_not_zero() {
        let mut provider = Number::default();
        let digit = provider.random_digit_not_zero();
        assert!(digit >= 1 && digit <= 9);
    }

    #[test]
    fn test_positive() {
        let mut provider = Number::default();
        let num = provider.positive();
        assert!(num > 0);
    }

    #[test]
    fn test_negative() {
        let mut provider = Number::default();
        let num = provider.negative();
        assert!(num < 0);
    }

    #[test]
    fn test_number_between_i32() {
        let mut provider = Number::default();
        let num = provider.number_between_i32(5, 10);
        assert!(num >= 5 && num < 10);

        // 测试min > max的情况
        let num = provider.number_between_i32(10, 5);
        assert!(num >= 5 && num < 10);

        // 测试min == max的情况
        let num = provider.number_between_i32(7, 7);
        assert_eq!(num, 7);
    }

    #[test]
    fn test_number_between_i64() {
        let mut provider = Number::default();
        let num = provider.number_between_i64(100, 200);
        assert!(num >= 100 && num < 200);
    }

    #[test]
    fn test_random_number() {
        let mut provider = Number::default();
        let num = provider.random_number(3);
        assert!(num >= 100 && num < 1000);
    }

    #[test]
    fn test_random_number_zero_digits() {
        let mut provider = Number::default();
        let num = provider.random_number(0);
        assert_eq!(num, 0);
    }

    #[test]
    fn test_digits() {
        let mut provider = Number::default();
        let digits = provider.digits(5);
        assert_eq!(digits.len(), 5);
        assert!(digits.chars().all(|c| c.is_ascii_digit()));
    }

    mod generator {
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
                Some(DefaultComponent::new("100".into(), 5.0)),
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
                NumberGenerator::new(0, 100, 0, None, Some(NullComponent::new(50.0)), None, false)
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
                NumberGenerator::new(1, 5, 0, None, None, Some(UniqueComponent::new()), false)
                    .unwrap();

            let mut res = generator.generate(5).unwrap();
            assert_eq!(res.len(), 5);
            dbg!(&res);
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
}
