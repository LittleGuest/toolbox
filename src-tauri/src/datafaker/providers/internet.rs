use super::random_str;
use crate::datafaker::{providers::Person, Locale};

static DOMAIN_SUFFIX: [&str; 24] = [
    "com", "org", "net", "cn", "io", "im", "info", "mobi", "biz", "pro", "us", "me", "top", "tv",
    "cc", "edu", "gov", "mil", "int", "name", "co", "ai", "app", "xyz",
];

static WINDOWS_VERSIONS: [&str; 6] = ["6.0", "6.1", "6.2", "6.3", "10.0", "11.0"];
static IOS_VERSIONS: [&str; 18] = [
    "10_0", "10_1", "10_2", "10_3", "11_0", "11_1", "11_2", "11_3", "11_4", "12_0", "12_4", "13_0",
    "13_7", "14_0", "14_7", "15_0", "15_7", "16_0",
];
static ANDROID_MANUFACTURERS: [&str; 23] = [
    "samsung",
    "sony",
    "huawei",
    "honor",
    "xiaomi",
    "redmi",
    "mi",
    "vivo",
    "oppo",
    "oneplus",
    "lg",
    "lenovo",
    "motorola",
    "nokia",
    "meizu",
    "zte",
    "asus",
    "smartisan",
    "nubia",
    "realme",
    "iqoo",
    "coolpad",
    "gionee",
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

    pub fn port(&self) -> String {
        fastrand::u16(1025..=65535).to_string()
    }

    pub fn user_agent_pc(&self) -> String {
        format!(
            "Mozilla/5.0 (Windows NT {}; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.{}.{} Safari/537.36",
            WINDOWS_VERSIONS[fastrand::usize(0..WINDOWS_VERSIONS.len())],
            fastrand::u8(60..100),
            fastrand::u16(2000..4000),
            fastrand::u8(1..200),
        )
    }

    pub fn user_agent_android(&self) -> String {
        format!(
            "Mozilla/5.0 (Linux; U; Android {}.0.0; zh-cn; {}-{} Build/{}) AppleWebKit/537.36 (KHTML, like Gecko)Version/4.0 Chrome/74.0.3729.157 Mobile Safari/537.36",
            fastrand::u8(7..13),
            ANDROID_MANUFACTURERS[fastrand::usize(0..ANDROID_MANUFACTURERS.len())].to_uppercase(),
            random_str(6).to_uppercase(),
            random_str(6).to_uppercase()
        )
    }

    pub fn user_agent_ios(&self) -> String {
        format!(
            "Mozilla/5.0 (iPhone; CPU iPhone OS {} like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/{}",
            IOS_VERSIONS[fastrand::usize(0..IOS_VERSIONS.len())],
            random_str(6).to_uppercase()
        )
    }
}
