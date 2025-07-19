//! 序列生成器

use crate::datafaker::{Error, Result};

/// 序列生成器
pub struct SequenceGenerator {
    /// 开始
    pub start: i64,
    /// 步长
    pub step: i64,
    /// 最大值
    pub max: Option<i64>,
    /// 最小值
    pub min: Option<i64>,
    /// 循环
    pub cycle: bool,

    /// 当前值
    current: i64,
}

impl Default for SequenceGenerator {
    fn default() -> Self {
        Self {
            start: 1,
            step: 1,
            max: None,
            min: None,
            cycle: false,
            current: 0,
        }
    }
}

impl SequenceGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with(
        start: i64,
        step: i64,
        max: Option<i64>,
        min: Option<i64>,
        cycle: bool,
    ) -> Self {
        Self {
            start,
            step,
            max,
            min,
            cycle,
            current: 0,
        }
    }

    pub fn check(&self, count: Option<i64>) -> Result<()> {
        // 开始值必须大于等于最小值
        if let Some(min) = self.min
            && self.start < min
        {
            return Err(Error::InvalidSequenceMinMax);
        }
        // 开始值必须小于等于最大值
        if let Some(max) = self.max
            && self.start > max
        {
            return Err(Error::InvalidSequenceMinMax);
        }
        // 有最小值和最大值,必须开启循环
        if let Some(min) = self.min
            && let Some(max) = self.max
            && let Some(count) = count
            && count > max - min
        {
            return Err(Error::SequenceCountNotEnough);
        }
        Ok(())
    }
}

impl Iterator for SequenceGenerator {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        // 返回开始值
        if self.current == 0 {
            self.current = self.start;
            return Some(self.start);
        }
        let mut current = self.current;
        // 当前值增加步长
        current += self.step;
        // 最大值，没有最大值，默认i64::MAX
        let max = if let Some(max) = self.max {
            max
        } else {
            i64::MAX
        };
        // 最小值，没有最小值，默认i64::MIN
        let min = if let Some(min) = self.min {
            min
        } else {
            i64::MIN
        };
        // 判断是否结束，当前值大于最大值，且不循环，返回None
        if current > max && !self.cycle {
            return None;
        }
        // 步长大于0且当前值大于最大值且循环，当前值等于最小值
        if self.step > 0 && current > max && self.cycle {
            current = min;
        } else if current < min && self.cycle {
            // 当前值小于最小值且循环，当前值等于最大值
            current = max;
        }
        self.current = current;
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_generator() {
        let mut generator = SequenceGenerator::new();
        // 产生从1到10的序列
        for i in 1..=10 {
            assert_eq!(generator.next(), Some(i));
        }
    }

    #[test]
    #[should_panic]
    fn test_sequence_generator_min_max_err()  {
        let mut generator = SequenceGenerator::new();
        generator.start = 5;
        generator.min = Some(5);
        generator.max = Some(10);
        generator.check(Some(10)).unwrap();
    }

    #[test]
    fn test_sequence_generator_cycle() {
        // 开启循环，设置最小值为1，最大值为5
        let mut generator = SequenceGenerator::new();
        generator.cycle = true;
        generator.min = Some(1);
        generator.max = Some(5);

        // 产生10个数字
        let mut seqs = Vec::new();
        (1..=10).for_each(|_| {
            seqs.push(generator.next());
        });
        assert_eq!(seqs.len(), 10);
        assert_eq!(
            seqs,
            vec![
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
            ]
        );
    }

    #[test]
    fn test_sequence_generator_cycle_with_start() {
        // 开启循环，设置最小值为1，最大值为5
        let mut generator = SequenceGenerator::new();
        generator.cycle = true;
        generator.start = 3;
        generator.min = Some(1);
        generator.max = Some(5);

        // 产生10个数字
        let mut seqs = Vec::new();
        (1..=10).for_each(|_| {
            seqs.push(generator.next());
        });
        assert_eq!(seqs.len(), 10);
        assert_eq!(
            seqs,
            vec![
                Some(3),
                Some(4),
                Some(5),
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(1),
                Some(2),
            ]
        );
    }
}
