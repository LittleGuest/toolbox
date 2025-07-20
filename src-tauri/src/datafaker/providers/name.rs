//! 姓名生成器模块
//! 提供各种随机姓名生成功能，包括全名、名、姓、头衔等

use std::sync::LazyLock;

use rand::prelude::*;
use serde::Deserialize;

use crate::datafaker::{
    DefaultComponent, Error, FakerData, Locale, NullComponent, Result, UniqueComponent,
};
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
/// 姓名前缀
static NAME_PREFIX: [&str; 5] = ["Mr.", "Mrs.", "Ms.", "Miss", "Dr."];
/// 姓名后缀
static NAME_SUFFIX: [&str; 11] = [
    "Jr.", "Sr.", "I", "II", "III", "IV", "V", "MD", "DDS", "PhD", "DVM",
];
/// 头衔描述
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
/// 头衔等级
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

/// 头衔岗位
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

/// 姓名Provider
pub struct Name<R: Rng> {
    /// 随机数生成器
    rng: R,
    /// 语言
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

    /// 生成包含可选前缀、名和姓的全名
    pub fn name(&mut self) -> String {
        // 随机决定是否包含前缀和后缀
        // 20%概率包含前缀
        let include_prefix = self.rng.random_bool(0.2);
        // 10%概率包含后缀
        let include_suffix = self.rng.random_bool(0.1);
        let mut parts = Vec::new();
        // 添加前缀
        if include_prefix {
            parts.push(NAME_PREFIX.choose(&mut self.rng).unwrap().to_string());
        }
        // 添加名和姓
        parts.push(self.first_name());
        parts.push(self.last_name());
        // 添加后缀
        if include_suffix {
            parts.push(NAME_SUFFIX.choose(&mut self.rng).unwrap().to_string());
        }
        parts.join(" ")
    }

    /// 生成包含中间名的全名
    pub fn name_with_middle(&mut self) -> String {
        // 随机决定是否包含前缀和后缀
        // 20%概率包含前缀
        let include_prefix = self.rng.random_bool(0.2);
        // 10%概率包含后缀
        let include_suffix = self.rng.random_bool(0.1);
        let mut parts = Vec::new();
        if include_prefix {
            parts.push(NAME_PREFIX.choose(&mut self.rng).unwrap().to_string());
        }
        // 名 + 中间名 + 姓
        parts.push(self.first_name());
        // 中间名使用随机名
        parts.push(self.first_name());
        parts.push(self.last_name());
        // 添加后缀
        if include_suffix {
            parts.push(NAME_SUFFIX.choose(&mut self.rng).unwrap().to_string());
        }
        parts.join(" ")
    }

    /// 返回与name()相同的值
    pub fn full_name(&mut self) -> String {
        self.name()
    }

    /// 返回随机名（如Aaliyah, Aaron, Abagail）
    pub fn first_name(&mut self) -> String {
        MALE_FIRST_NAME_DATA
            .choose(&mut self.rng)
            .unwrap()
            .to_string()
    }

    /// 返回随机女性名
    pub fn female_first_name(&mut self) -> String {
        FEMALE_FIRST_NAME_DATA
            .choose(&mut self.rng)
            .unwrap()
            .to_string()
    }

    /// 返回随机男性名
    pub fn male_first_name(&mut self) -> String {
        MALE_FIRST_NAME_DATA
            .choose(&mut self.rng)
            .unwrap()
            .to_string()
    }

    /// 返回随机姓（如Smith, Jones, Baldwin）
    pub fn last_name(&mut self) -> String {
        LAST_NAME_DATA.choose(&mut self.rng).unwrap().to_string()
    }

    /// 返回随机头衔前缀（如Mr., Mrs., Ms., Dr.）
    pub fn prefix(&mut self) -> String {
        NAME_PREFIX.choose(&mut self.rng).unwrap().to_string()
    }

    /// 返回随机头衔后缀（如Jr., Sr., III, MD, PhD）
    pub fn suffix(&mut self) -> String {
        NAME_SUFFIX.choose(&mut self.rng).unwrap().to_string()
    }

    /// 生成三部分组成的职位头衔
    /// 格式: [描述符] [级别] [职位]
    pub fn title(&mut self) -> String {
        let descriptor = TITLE_DESCRIPTOR.choose(&mut self.rng).unwrap();
        let level = TITLE_LEVEL.choose(&mut self.rng).unwrap();
        let job = TITLE_JOB.choose(&mut self.rng).unwrap();
        format!("{} {} {}", descriptor, level, job)
    }
}

/// 姓名格式
#[derive(Deserialize)]
pub enum NameFormat {
    /// 姓名
    FirstName,
    /// 姓氏
    LastName,
    /// 姓名+姓氏
    FullName,
}

/// 姓名生成器
pub struct NameGenerator {
    /// 姓名格式
    pub format: NameFormat,
    /// 语言
    pub locales: Vec<Locale>,

    /// 包含默认值
    pub include_default: Option<DefaultComponent>,
    /// 包含NULL值
    pub include_null: Option<NullComponent>,
    /// 唯一值
    pub unique: Option<UniqueComponent>,
    /// 禁用链接
    pub forbidden_links: bool,
}

impl Default for NameGenerator {
    fn default() -> Self {
        Self {
            format: NameFormat::FullName,
            locales: vec![Locale::ZhCn],
            include_default: None,
            include_null: None,
            unique: None,
            forbidden_links: false,
        }
    }
}

impl NameGenerator {
    pub fn new(format: NameFormat, locals: Vec<Locale>) -> Self {
        Self {
            format,
            locales: locals,
            include_default: None,
            include_null: None,
            unique: None,
            forbidden_links: false,
        }
    }

    /// 校验参数
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
        // 检查出现百分比是否大于100
        if percent - 100.0 > 0.0 {
            return Err(Error::PercentNotGreaterThan100);
        }
        Ok(())
    }

    /// 生成姓名，count为生成数量
    pub fn generate(&mut self, count: usize) -> Result<Vec<Option<String>>> {
        self.check()?;
        let mut res = Vec::with_capacity(count);
        let mut name = Name::new(rand::rng());
        for _ in 0..count {
            let locale = self.locales.choose(&mut name.rng).unwrap();
            name.locale = *locale;
            // 包含默认值
            if let Some(dc) = &self.include_default {
                // 根据默认值出现百分比判断是否应用默认值
                let include_default = name.rng.random_bool(dc.percent / 100.0);
                if include_default {
                    res.push(Some(dc.default.clone()));
                    continue;
                }
            }
            // 包含NULL值
            if let Some(nc) = &self.include_null {
                // 根据NULL值出现百分比判断是否应用NULL值
                let include_null = name.rng.random_bool(nc.percent / 100.0);
                if include_null {
                    res.push(None);
                    continue;
                }
            }
            // 不包含默认值和NULL值
            let mut name_fn = || match self.format {
                NameFormat::FirstName => name.first_name(),
                NameFormat::LastName => name.last_name(),
                NameFormat::FullName => name.full_name(),
            };
            let mut full_name = name_fn();
            // 校验唯一值
            if let Some(unique) = &mut self.unique {
                if unique.value.contains(full_name.as_str()) {
                    loop {
                        // 重新生成
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

        // 测试全名生成
        let full_name = name_provider.name();
        assert!(!full_name.is_empty());
        assert!(full_name.split_whitespace().count() >= 2);

        // 测试带中间名的全名
        let middle_name = name_provider.name_with_middle();
        assert!(!middle_name.is_empty());
        assert!(middle_name.split_whitespace().count() >= 3);

        // 测试名和姓
        let first = name_provider.first_name();
        let last = name_provider.last_name();
        assert!(!first.is_empty());
        assert!(!last.is_empty());

        // 测试性别特异性名
        let female = name_provider.female_first_name();
        let male = name_provider.male_first_name();
        assert!(!female.is_empty());
        assert!(!male.is_empty());

        // 测试头衔
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
            let mut generator =
                NameGenerator::new(NameFormat::FullName, vec![Locale::ZhCn, Locale::EnUs]);
            let res = generator.generate(10);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 10);
            assert!(res.iter().all(|v| v.is_some()));
        }

        #[test]
        fn test_name_generator_with_format() {
            let mut generator = NameGenerator::new(NameFormat::FirstName, vec![Locale::ZhCn]);
            let res = generator.generate(10);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 10);
            assert!(res.iter().all(|v| v.is_some()));
            assert!(
                res.into_iter()
                    .all(|v| MALE_FIRST_NAME_DATA.contains(&v.unwrap()))
            );

            let mut generator = NameGenerator::new(NameFormat::LastName, vec![Locale::ZhCn]);
            let res = generator.generate(10);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 10);
            assert!(res.iter().all(|v| v.is_some()));
            assert!(
                res.into_iter()
                    .all(|v| LAST_NAME_DATA.contains(&v.unwrap()))
            );

            let mut generator = NameGenerator::new(NameFormat::FullName, vec![Locale::ZhCn]);
            let res = generator.generate(10);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 10);
            assert!(res.iter().all(|v| v.is_some()));
        }

        #[test]
        fn test_name_generator_with_default() {
            let mut generator = NameGenerator::new(NameFormat::FullName, vec![Locale::ZhCn]);
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
            let mut generator = NameGenerator::new(NameFormat::FullName, vec![Locale::ZhCn]);
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
            let mut generator = NameGenerator::new(NameFormat::FullName, vec![Locale::ZhCn]);
            generator.include_default = Some(DefaultComponent::new("默认姓名".to_string(), 56.0));
            generator.include_null = Some(NullComponent::new(56.0));
            generator.check().unwrap()
        }

        #[test]
        fn test_name_generator_with_default_null() {
            let mut generator = NameGenerator::new(NameFormat::FullName, vec![Locale::ZhCn]);
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
            let mut generator = NameGenerator::new(NameFormat::FullName, vec![Locale::ZhCn]);
            generator.unique = Some(UniqueComponent::new());
            let res = generator.generate(10);
            assert!(res.is_ok());
            let res = res.unwrap();
            assert_eq!(res.len(), 10);
            assert!(res.iter().all(|v| v.is_some()));
        }
    }
}
