use super::random_str;
use crate::datafaker::{providers::Person, Locale};

static DOMAIN_SUFFIX: [&str; 24] = [
    "com", "org", "net", "cn", "io", "im", "info", "mobi", "biz", "pro", "us", "me", "top", "tv",
    "cc", "edu", "gov", "mil", "int", "name", "co", "ai", "app", "xyz",
];

pub struct Internet {
    pub locale: Locale,
}

impl Internet {
    pub fn new() -> Self {
        Self {
            locale: Default::default(),
        }
    }

    pub fn new_with_locale(locale: Locale) -> Self {
        Self { locale }
    }

    pub fn username(&self) -> String {
        let name = Person::new_with_locale(self.locale);
        format!(
            "{}{}",
            name.first_name().to_lowercase(),
            name.last_name().to_lowercase()
        )
    }

    pub fn ipv4(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
        )
    }

    pub fn app_name(&self) -> String {
        random_str(fastrand::usize(4..11))
    }

    pub fn app_bundle_id(&self) -> String {
        format!(
            "{}.{}.{}",
            DOMAIN_SUFFIX[fastrand::usize(0..DOMAIN_SUFFIX.len())],
            random_str(fastrand::usize(2..21)),
            random_str(fastrand::usize(2..21)),
        )
        .to_lowercase()
    }

    pub fn app_version(&self) -> String {
        format!(
            "{}.{}.{}",
            fastrand::u8(1..11),
            fastrand::u8(0..100),
            fastrand::u16(0..1000)
        )
    }
}
