
use std::sync::LazyLock;

use rand::prelude::*;
use serde::Deserialize;

use crate::{DefaultComponent, Error, FakerData, Locale, NullComponent, Result, UniqueComponent};
static MALE_FIRST_NAME_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let area = FakerData::get("male-first-name").unwrap();
    String::from_utf8_lossy(&area.data)
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect::<Vec<_>>()
});
static FEMALE_FIRST_NAME_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let area = FakerData::get("female-first-name").unwrap();
    String::from_utf8_lossy(&area.data)
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect::<Vec<_>>()
});
static LAST_NAME_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let area = FakerData::get("last-name").unwrap();
    String::from_utf8_lossy(&area.data)
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect::<Vec<_>>()
});
static NAME_PREFIX: [&str; 5] = ["Mr.", "Mrs.", "Ms.", "Miss", "Dr."];
static NAME_SUFFIX: [&str; 11] = [
    "Jr.", "Sr.", "I", "II", "III", "IV", "V", "MD", "DDS", "PhD", "DVM",
];
static CN_LAST_NAME: [&str; 40] = [
    "赵", "钱", "孙", "李", "周", "吴", "郑", "王", "冯", "陈", "褚", "卫", "蒋", "沈", "韩", "杨",
    "朱", "秦", "尤", "许", "何", "吕", "施", "张", "孔", "曹", "严", "华", "金", "魏", "陶", "姜",
    "谢", "邹", "喻", "柏", "水", "窦", "章", "云",
];
static CN_FIRST_NAME_CHARS: [&str; 64] = [
    "伟", "刚", "勇", "毅", "俊", "峰", "强", "军", "平", "保", "东", "文", "辉", "力", "明", "永",
    "健", "世", "广", "志", "义", "兴", "良", "海", "山", "仁", "波", "宁", "贵", "福", "生", "龙",
    "元", "全", "国", "胜", "学", "祥", "才", "发", "武", "新", "利", "清", "飞", "彬", "富", "顺",
    "信", "子", "杰", "涛", "昌", "成", "康", "星", "光", "天", "达", "安", "岩", "中", "茜", "琳",
];
static PINYIN_LAST_NAME: [&str; 20] = [
    "Zhao", "Qian", "Sun", "Li", "Zhou", "Wu", "Zheng", "Wang", "Feng", "Chen", "Jiang", "Shen",
    "Han", "Yang", "Zhu", "Qin", "Xu", "He", "Lu", "Zhang",
];
static PINYIN_FIRST_NAME: [&str; 32] = [
    "Wei", "Gang", "Yong", "Jun", "Feng", "Qiang", "Ming", "Jian", "Wen", "Hui", "Li", "Hai", "Bo",
    "Ning", "Long", "Xiang", "Xue", "Xin", "Qing", "Fei", "Bin", "Jie", "Tao", "Kang", "Xing",
    "An", "Lin", "Qian", "Na", "Min", "Lei", "Hao",
];
static TITLE_DESCRIPTOR: [&str; 22] = [
    "Lead",
    "Senior",
    "Direct",
    "Corporate",
    "Dynamic",
    "Future",
    "Product",
    "National",
    "Regional",
    "District",
    "Central",
    "Global",
    "Customer",
    "Investor",
    "Dynamic",
    "International",
    "Legacy",
    "Forward",
    "Internal",
    "Human",
    "Chief",
    "Principal",
];
static TITLE_LEVEL: [&str; 37] = [
    "Solutions",
    "Program",
    "Brand",
    "Security",
    "Research",
    "Marketing",
    "Directives",
    "Implementation",
    "Integration",
    "Functionality",
    "Response",
    "Paradigm",
    "Tactics",
    "Identity",
    "Markets",
    "Group",
    "Division",
    "Applications",
    "Optimization",
    "Operations",
    "Infrastructure",
    "Intranet",
    "Communications",
    "Web",
    "Branding",
    "Quality",
    "Assurance",
    "Mobility",
    "Accounts",
    "Data",
    "Creative",
    "Configuration",
    "Accountability",
    "Interactions",
    "Factors",
    "Usability",
    "Metrics",
];

static TITLE_JOB: [&str; 25] = [
    "Supervisor",
    "Associate",
    "Executive",
    "Liaison",
    "Officer",
    "Manager",
    "Engineer",
    "Specialist",
    "Director",
    "Coordinator",
    "Administrator",
    "Architect",
    "Analyst",
    "Designer",
    "Planner",
    "Orchestrator",
    "Technician",
    "Developer",
    "Producer",
    "Consultant",
    "Assistant",
    "Facilitator",
    "Agent",
    "Representative",
    "Strategist",
];

pub struct Name<R: Rng> {
    rng: R,
    locale: Locale,
}

impl Default for Name<ThreadRng> {
    fn default() -> Self {
        Self {
            rng: rand::rng(),
            locale: Default::default(),
        }
    }
}

impl<R: Rng> Name<R> {
    pub fn new(rng: R) -> Self {
        Self {
            rng,
            locale: Default::default(),
        }
    }

    pub fn new_with_locale(rng: R, locale: Locale) -> Self {
        Self { rng, locale }
    }

    pub fn name(&mut self) -> String {
        if matches!(self.locale, Locale::ZhCn | Locale::ZhTw) {
            return format!("{}{}", self.last_name(), self.first_name());
        }
        if matches!(self.locale, Locale::ZhPinyin) {
            return format!("{} {}", self.first_name(), self.last_name());
        }
        let include_prefix = self.rng.random_bool(0.2);
        let include_suffix = self.rng.random_bool(0.1);
        let mut parts = Vec::new();
        if include_prefix {
            parts.push(NAME_PREFIX.choose(&mut self.rng).unwrap().to_string());
        }
        parts.push(self.first_name());
        parts.push(self.last_name());
        if include_suffix {
            parts.push(NAME_SUFFIX.choose(&mut self.rng).unwrap().to_string());
        }
        parts.join(" ")
    }

    pub fn name_with_middle(&mut self) -> String {
        if matches!(self.locale, Locale::ZhCn | Locale::ZhTw | Locale::ZhPinyin) {
            return self.name();
        }
        let include_prefix = self.rng.random_bool(0.2);
        let include_suffix = self.rng.random_bool(0.1);
        let mut parts = Vec::new();
        if include_prefix {
            parts.push(NAME_PREFIX.choose(&mut self.rng).unwrap().to_string());
        }
        parts.push(self.first_name());
        parts.push(self.first_name());
        parts.push(self.last_name());
        if include_suffix {
            parts.push(NAME_SUFFIX.choose(&mut self.rng).unwrap().to_string());
        }
        parts.join(" ")
    }

    pub fn full_name(&mut self) -> String {
        self.name()
    }

    pub fn first_name(&mut self) -> String {
        if matches!(self.locale, Locale::ZhCn | Locale::ZhTw) {
            let len = self.rng.random_range(1..=2);
            return (0..len)
                .map(|_| {
                    CN_FIRST_NAME_CHARS
                        .choose(&mut self.rng)
                        .unwrap()
                        .to_string()
                })
                .collect::<Vec<_>>()
                .join("");
        }
        if matches!(self.locale, Locale::ZhPinyin) {
            return PINYIN_FIRST_NAME.choose(&mut self.rng).unwrap().to_string();
        }
        MALE_FIRST_NAME_DATA
            .choose(&mut self.rng)
            .unwrap()
            .to_string()
    }

    pub fn female_first_name(&mut self) -> String {
        if matches!(self.locale, Locale::ZhCn | Locale::ZhTw | Locale::ZhPinyin) {
            return self.first_name();
        }
        FEMALE_FIRST_NAME_DATA
            .choose(&mut self.rng)
            .unwrap()
            .to_string()
    }

    pub fn male_first_name(&mut self) -> String {
        if matches!(self.locale, Locale::ZhCn | Locale::ZhTw | Locale::ZhPinyin) {
            return self.first_name();
        }
        MALE_FIRST_NAME_DATA
            .choose(&mut self.rng)
            .unwrap()
            .to_string()
    }

    pub fn last_name(&mut self) -> String {
        if matches!(self.locale, Locale::ZhCn | Locale::ZhTw) {
            return CN_LAST_NAME.choose(&mut self.rng).unwrap().to_string();
        }
        if matches!(self.locale, Locale::ZhPinyin) {
            return PINYIN_LAST_NAME.choose(&mut self.rng).unwrap().to_string();
        }
        LAST_NAME_DATA.choose(&mut self.rng).unwrap().to_string()
    }

    pub fn prefix(&mut self) -> String {
        NAME_PREFIX.choose(&mut self.rng).unwrap().to_string()
    }

    pub fn suffix(&mut self) -> String {
        NAME_SUFFIX.choose(&mut self.rng).unwrap().to_string()
    }

    pub fn title(&mut self) -> String {
        let descriptor = TITLE_DESCRIPTOR.choose(&mut self.rng).unwrap();
        let level = TITLE_LEVEL.choose(&mut self.rng).unwrap();
        let job = TITLE_JOB.choose(&mut self.rng).unwrap();
        format!("{} {} {}", descriptor, level, job)
    }
}

#[derive(Deserialize)]
pub enum NameFormat {
    First,
    Last,
    Full,
}

pub struct NameGenerator {
    pub format: NameFormat,
    pub locales: Vec<Locale>,

    pub include_default: Option<DefaultComponent>,
    pub include_null: Option<NullComponent>,
    pub unique: Option<UniqueComponent>,
    pub forbidden_links: bool,
}

impl Default for NameGenerator {
    fn default() -> Self {
        Self {
            format: NameFormat::Full,
            locales: vec![Locale::ZhCn],
            include_default: None,
            include_null: None,
            unique: None,
            forbidden_links: false,
        }
    }
}

impl NameGenerator {
    pub fn new(format: NameFormat, locals: Vec<Locale>) -> Result<Self> {
        let name = Self {
            format,
            locales: locals,
            include_default: None,
            include_null: None,
            unique: None,
            forbidden_links: false,
        };
        name.check()?;
        Ok(name)
    }

    pub fn check(&self) -> Result<()> {
        if self.locales.is_empty() {
            return Err(Error::InvalidParameter("没有选择语言"));
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
        if percent - 100.0 > 0.0 {
            return Err(Error::PercentNotGreaterThan100);
        }
        Ok(())
    }

    pub fn generate(&mut self, count: usize) -> Result<Vec<Option<String>>> {
        self.check()?;
        let mut res = Vec::with_capacity(count);
        let mut name = Name::new(rand::rng());
        for _ in 0..count {
            let locale = self.locales.choose(&mut name.rng).unwrap();
            name.locale = *locale;
            if let Some(dc) = &self.include_default {
                let include_default = name.rng.random_bool(dc.percent / 100.0);
                if include_default {
                    res.push(Some(dc.default.clone()));
                    continue;
                }
            }
            if let Some(nc) = &self.include_null {
                let include_null = name.rng.random_bool(nc.percent / 100.0);
                if include_null {
                    res.push(None);
                    continue;
                }
            }
            let mut name_fn = || match self.format {
                NameFormat::First => name.first_name(),
                NameFormat::Last => name.last_name(),
                NameFormat::Full => name.full_name(),
            };
            let mut full_name = name_fn();
            if let Some(unique) = &mut self.unique {
                if unique.value.contains(full_name.as_str()) {
                    loop {
                        full_name = name_fn();
                        if !unique.value.iter().any(|v| v.eq(&full_name)) {
                            unique.value.insert(full_name.clone());
                            break;
                        }
                    }
                } else {
                    unique.value.insert(full_name.clone());
                }
            }
            res.push(Some(full_name));
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_generation() {
        let mut name_provider = Name::default();

        let full_name = name_provider.name();
        assert!(!full_name.is_empty());
        assert!(full_name.split_whitespace().count() >= 2);

        let middle_name = name_provider.name_with_middle();
        assert!(!middle_name.is_empty());
        assert!(middle_name.split_whitespace().count() >= 3);

        let first = name_provider.first_name();
        let last = name_provider.last_name();
        assert!(!first.is_empty());
        assert!(!last.is_empty());

        let female = name_provider.female_first_name();
        let male = name_provider.male_first_name();
        assert!(!female.is_empty());
        assert!(!male.is_empty());

        let title = name_provider.title();
        assert!(!title.is_empty());
        assert!(title.split_whitespace().count() >= 3);
    }

    mod generator {
        use super::*;

        #[test]
        fn test_name_generator() {
            let mut generator = NameGenerator::default();
            let res = generator.generate(10);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 10);
            assert!(res.iter().all(|v| v.is_some()));
        }

        #[test]
        fn test_name_generator_with_locale() {
            let generator = NameGenerator::new(NameFormat::Full, vec![Locale::ZhCn, Locale::EnUs]);
            assert!(generator.is_ok());
            let mut generator = generator.unwrap();
            let res = generator.generate(10);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 10);
            assert!(res.iter().all(|v| v.is_some()));
        }

        #[test]
        fn test_name_generator_with_format() {
            let generator = NameGenerator::new(NameFormat::First, vec![Locale::ZhCn]);
            assert!(generator.is_ok());
            let mut generator = generator.unwrap();
            let res = generator.generate(10);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 10);
            assert!(res.iter().all(|v| v.is_some()));
            assert!(res.into_iter().all(|v| {
                let name = v.unwrap();
                !name.is_empty()
                    && name.chars().all(|item| {
                        let item = item.to_string();
                        CN_FIRST_NAME_CHARS.contains(&item.as_str())
                    })
            }));

            let generator = NameGenerator::new(NameFormat::Last, vec![Locale::ZhCn]);
            assert!(generator.is_ok());
            let mut generator = generator.unwrap();
            let res = generator.generate(10);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 10);
            assert!(res.iter().all(|v| v.is_some()));
            assert!(
                res.into_iter()
                    .all(|v| CN_LAST_NAME.contains(&v.unwrap().as_str()))
            );

            let generator = NameGenerator::new(NameFormat::Full, vec![Locale::ZhCn]);
            assert!(generator.is_ok());
            let mut generator = generator.unwrap();
            let res = generator.generate(10);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 10);
            assert!(res.iter().all(|v| v.is_some()));
        }

        #[test]
        fn test_name_generator_with_default() {
            let generator = NameGenerator::new(NameFormat::Full, vec![Locale::ZhCn]);
            assert!(generator.is_ok());
            let mut generator = generator.unwrap();
            generator.include_default = Some(DefaultComponent::new("默认姓名".to_string(), 100.0));
            let res = generator.generate(10);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 10);
            assert!(res.iter().all(|v| v.is_some()));
            assert!(res.iter().all(|v| v.eq(&Some("默认姓名".to_string()))));
        }

        #[test]
        fn test_name_generator_with_null() {
            let generator = NameGenerator::new(NameFormat::Full, vec![Locale::ZhCn]);
            assert!(generator.is_ok());
            let mut generator = generator.unwrap();
            generator.include_null = Some(NullComponent::new(100.0));
            let res = generator.generate(10);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 10);
            assert!(res.iter().all(|v| v.is_none()));
        }

        #[test]
        #[should_panic]
        fn test_name_generator_with_default_null_error() {
            let generator = NameGenerator::new(NameFormat::Full, vec![Locale::ZhCn]);
            assert!(generator.is_ok());
            let mut generator = generator.unwrap();
            generator.include_default = Some(DefaultComponent::new("默认姓名".to_string(), 56.0));
            generator.include_null = Some(NullComponent::new(56.0));
            generator.check().unwrap()
        }

        #[test]
        fn test_name_generator_with_default_null() {
            let generator = NameGenerator::new(NameFormat::Full, vec![Locale::ZhCn]);
            assert!(generator.is_ok());
            let mut generator = generator.unwrap();
            generator.include_default = Some(DefaultComponent::new("默认姓名".to_string(), 80.0));
            generator.include_null = Some(NullComponent::new(20.0));
            let res = generator.generate(1000);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 1000);
            let none_count = res.iter().filter(|v| v.is_none()).count();
            assert!(none_count <= 200);
        }

        #[test]
        fn test_name_generator_with_unique() {
            let generator = NameGenerator::new(NameFormat::Full, vec![Locale::ZhCn]);
            assert!(generator.is_ok());
            let mut generator = generator.unwrap();
            generator.unique = Some(UniqueComponent::new());
            let res = generator.generate(10);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 10);
            assert!(res.iter().all(|v| v.is_some()));
        }
    }
}
