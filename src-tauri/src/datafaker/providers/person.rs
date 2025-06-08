use std::{ops::Deref, sync::LazyLock};

use crate::datafaker::{FakerData, Locale, Provider};

static MALE_FIRST_NAME_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let area = FakerData::get("male-first-name").unwrap();
    String::from_utf8_lossy(&area.data)
        .lines()
        .map(String::from)
        .collect::<Vec<_>>()
});

static FEMALE_FIRST_NAME_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let area = FakerData::get("female-first-name").unwrap();
    String::from_utf8_lossy(&area.data)
        .lines()
        .map(String::from)
        .collect::<Vec<_>>()
});

static LAST_NAME_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let area = FakerData::get("last-name").unwrap();
    String::from_utf8_lossy(&area.data)
        .lines()
        .map(String::from)
        .collect::<Vec<_>>()
});

static QQ_NICK_NAME_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let area = FakerData::get("qq-nick-name").unwrap();
    String::from_utf8_lossy(&area.data)
        .lines()
        .map(String::from)
        .collect::<Vec<_>>()
});

static PREFIX: [&str; 5] = ["Mr.", "Mrs.", "Ms.", "Miss", "Dr."];
static PREFIX_LEN: usize = PREFIX.len();

static SUFFIX: [&str; 11] = [
    "Jr.", "Sr.", "I", "II", "III", "IV", "V", "MD", "DDS", "PhD", "DVM",
];
static SUFFIX_LEN: usize = SUFFIX.len();

pub enum Sex {
    Female = 1,
    Male = 0,
    Unknown = -1,
}

pub struct Person {
    locale: Locale,
}

impl Provider for Person {
    fn name(&self) -> String {
        std::any::type_name_of_val(self).into()
    }
}

impl Person {
    pub fn new() -> Self {
        Self {
            locale: Default::default(),
        }
    }

    pub fn new_with_locale(locale: Locale) -> Self {
        Self { locale }
    }

    pub fn prefix(&self) -> String {
        PREFIX[fastrand::usize(0..PREFIX_LEN)].into()
    }

    pub fn suffix(&self) -> String {
        SUFFIX[fastrand::usize(0..SUFFIX_LEN)].into()
    }

    pub fn first_name(&self) -> String {
        let name = MALE_FIRST_NAME_DATA.deref();
        name.get(fastrand::usize(0..name.len())).unwrap().into()
    }

    pub fn last_name(&self) -> String {
        let name = LAST_NAME_DATA.deref();
        name.get(fastrand::usize(0..name.len())).unwrap().into()
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name(), self.last_name())
    }

    pub fn full_name_with_prefix(&self) -> String {
        format!(
            "{} {} {}",
            self.prefix(),
            self.first_name(),
            self.last_name()
        )
    }

    pub fn full_name_with_suffix(&self) -> String {
        format!(
            "{} {} {}",
            self.first_name(),
            self.last_name(),
            self.suffix()
        )
    }

    pub fn name_with_middle(&self) -> String {
        format!(
            "{} {} {}",
            self.first_name(),
            self.last_name(),
            self.last_name(),
        )
    }

    pub fn name_with_middle_prefix(&self) -> String {
        format!(
            "{} {} {} {}",
            self.prefix(),
            self.first_name(),
            self.last_name(),
            self.last_name(),
        )
    }

    pub fn name_with_middle_suffix(&self) -> String {
        format!(
            "{} {} {} {}",
            self.first_name(),
            self.last_name(),
            self.last_name(),
            self.suffix()
        )
    }

    pub fn qq(&self) -> String {
        fastrand::u64(10000..=100000000000).to_string()
    }

    pub fn qq_nick_name(&self) -> String {
        let name = QQ_NICK_NAME_DATA.deref();
        name.get(fastrand::usize(0..name.len())).unwrap().into()
    }

    pub fn nick_name(&self) -> String {
        Self::random_str(fastrand::usize(4..10))
    }

    fn random_str(count: usize) -> String {
        let mut str = String::with_capacity(count);
        if count == 0 {
            return str;
        }

        (0..count).for_each(|_| {
            let char: char = match fastrand::u8(0..3) {
                0 => fastrand::u8(b'a'..=b'z').into(),
                1 => fastrand::u8(b'A'..=b'Z').into(),
                _ => fastrand::u8(48..=57).into(),
            };
            str.push(char);
        });

        str
    }
}

pub enum CreditCardType {
    Visa,
    MasterCard,
    Amex,
    UnionPay,
    Jcb,
}

#[cfg(test)]
mod tests {
    use super::Person;

    #[test]
    fn test_prefix() {
        assert!(!Person::new().prefix().is_empty())
    }

    #[test]
    fn test_suffix() {
        assert!(!Person::new().suffix().is_empty())
    }

    #[test]
    fn test_first_name() {
        assert!(!Person::new().first_name().is_empty())
    }

    #[test]
    fn test_last_name() {
        assert!(!Person::new().last_name().is_empty())
    }

    #[test]
    fn test_full_name() {
        assert!(!Person::new().full_name().is_empty())
    }

    #[test]
    fn test_full_name_with_prefix() {
        assert!(!Person::new().full_name_with_prefix().is_empty())
    }

    #[test]
    fn test_full_name_with_suffix() {
        assert!(!Person::new().full_name_with_suffix().is_empty())
    }

    #[test]
    fn test_name_with_middle() {
        assert!(!Person::new().name_with_middle().is_empty())
    }

    #[test]
    fn test_name_with_middle_prefix() {
        assert!(!Person::new().name_with_middle_prefix().is_empty())
    }

    #[test]
    fn test_name_with_middle_suffix() {
        assert!(!Person::new().name_with_middle_suffix().is_empty())
    }

    #[test]
    fn test_random_str() {
        let str = Person::random_str(8);
        assert_eq!(str.len(), 8)
    }
}
