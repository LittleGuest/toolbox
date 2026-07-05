use rand::{Rng, prelude::IndexedRandom};

use crate::{DefaultComponent, Error, NullComponent, Result};

static CHINESE_TEXT_CHARS: [&str; 96] = [
    "数", "据", "服", "务", "平", "台", "系", "统", "用", "户", "订", "单", "产", "品", "接", "口",
    "配", "置", "日", "志", "任", "务", "审", "核", "状", "态", "消", "息", "通", "知", "统", "计",
    "报", "表", "资", "源", "权", "限", "角", "色", "组", "织", "部", "门", "项", "目", "节", "点",
    "流", "程", "规", "则", "模", "板", "标", "签", "分", "类", "内", "容", "描", "述", "备", "注",
    "测", "试", "示", "例", "文", "本", "信", "息", "详", "情", "中", "心", "管", "理", "云", "智",
    "能", "安", "全", "监", "控", "开", "发", "运", "维", "财", "务", "客", "户", "营", "销", "网",
];

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
        let mut text = String::with_capacity(length * 3);
        for _ in 0..length {
            text.push_str(CHINESE_TEXT_CHARS.choose(&mut self.rng).unwrap());
        }
        text
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
        assert!(s.chars().count() >= 10 && s.chars().count() <= 255);
        let s = text.random_text(Some(100), Some(100));
        assert_eq!(s.chars().count(), 100);
    }
}
