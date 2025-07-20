use std::{ops::Range, sync::LazyLock};

use regex::Regex;

use super::random_str;
use crate::datafaker::{
    Locale,
    providers::{Name, Person},
};

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

static PRIVATE_RANGES_REGEX: LazyLock<[Regex; 8]> = LazyLock::new(|| {
    let local: Regex = Regex::new(r"^10\.").unwrap();
    let shared_addr: Regex = Regex::new(r"^100\.(6[4-9]|[7-9]\d|1[0-1]\d|12[0-7])\.").unwrap();
    let loopback: Regex = Regex::new(r"^127\.").unwrap();
    let local_link_addr: Regex = Regex::new(r"^169\.254\.").unwrap();
    let local_private_172: Regex = Regex::new(r"^172\.(1[6-9]|2\d|3[0-1])\.").unwrap();
    let ietf_protocols: Regex = Regex::new(r"^192\.0\.0\.").unwrap();
    let local_private_192: Regex = Regex::new(r"^192\.168\.").unwrap();
    let benchmark: Regex = Regex::new(r"^198\.(1[8-9])\.").unwrap();

    [
        local,
        shared_addr,
        loopback,
        local_link_addr,
        local_private_172,
        ietf_protocols,
        local_private_192,
        benchmark,
    ]
});

static RESERVED_RANGES_REGEX: LazyLock<[Regex; 7]> = LazyLock::new(|| {
    let local: Regex = Regex::new(r"^0\.").unwrap();
    let test_net_192: Regex = Regex::new(r"^192\.0\.2\.").unwrap();
    let relay: Regex = Regex::new(r"^192\.88\.99\.").unwrap();
    let test_net_198: Regex = Regex::new(r"^198\.51\.100\.").unwrap();
    let test_net_203: Regex = Regex::new(r"^203\.0\.113\.").unwrap();
    let multicast: Regex = Regex::new(r"^(22[4-9]|23\d)\.").unwrap();
    let future_use: Regex = Regex::new(r"^(24\d|25[0-5])\.").unwrap();

    [
        local,
        test_net_192,
        relay,
        test_net_198,
        test_net_203,
        multicast,
        future_use,
    ]
});

static PRIVATE_RANGES: [&[Range<i32>; 4]; 8] = [
    &[10..10, 0..255, 0..255, 1..255],
    &[100..100, 64..127, 0..255, 1..255],
    &[127..127, 0..255, 0..255, 1..255],
    &[169..169, 254..254, 0..255, 1..255],
    &[172..172, 16..31, 0..255, 1..255],
    &[192..192, 0..0, 0..0, 1..255],
    &[192..192, 168..168, 0..255, 1..255],
    &[198..198, 18..19, 0..255, 1..255],
];
static PRIVATE_RANGES_LEN: usize = PRIVATE_RANGES.len();

static PUBLIC_EMAIL_DOMAINS: [&str; 6] =
    ["outlook", "hotmail", "gmail", "yahoo", "protonmail", "zoho"];
static BUSINESS_EMAIL_DOMAINS: [&str; 20] = [
    "google",
    "microsoft",
    "apple",
    "nvidia",
    "meta",
    "facebook",
    "netflix",
    "amazon",
    "slack",
    "amd",
    "intel",
    "hp",
    "ibm",
    "wellsfargo",
    "goldmansachs",
    "janestreet",
    "akumacaptial",
    "sequioacapital",
    "walmart",
    "costco",
];
static GOVERNMENT_EMAIL_DOMAINS: [&str; 4] = ["fbi", "cia", "nsa", "gov"];

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
        let mut name = Name::new_with_locale(rand::rng(), self.locale);

        format!(
            "{}{}",
            name.first_name().to_lowercase(),
            name.last_name().to_lowercase()
        )
    }

    pub fn domain(&self) -> String {
        format!(
            "{}.{}",
            random_str(fastrand::usize(2..11)),
            DOMAIN_SUFFIX[fastrand::usize(0..DOMAIN_SUFFIX.len())],
        )
        .to_lowercase()
    }

    pub fn static_url(&self) -> String {
        let mut prefix = "http://";
        if fastrand::u8(0..101).is_multiple_of(3) {
            prefix = "https://";
        }
        format!(
            "{prefix}{}/{}/{}.{}",
            self.domain(),
            fastrand::usize(1..10000000000),
            random_str(fastrand::usize(8..33)),
            random_str(fastrand::usize(8..33)),
        )
    }

    pub fn standard_generic_email(&self) -> String {
        match fastrand::u8(0..4) {
            0 => self.standard_public_email_alias(),
            1 => self.standard_public_email(),
            2 => self.standard_business_email(),
            3 => self.standard_government_email(),
            _ => String::new(),
        }
    }

    pub fn standard_public_email(&self) -> String {
        let domain =
            PUBLIC_EMAIL_DOMAINS[fastrand::usize(0..PUBLIC_EMAIL_DOMAINS.len())].to_string();
        self.build_email(self.username(), domain)
    }

    pub fn standard_public_email_alias(&self) -> String {
        let username = match fastrand::u8(0..2) {
            0 => format!("{}+{}", self.username(), fastrand::u8(0..100)),
            1 => format!(
                "{}+{}",
                self.username(),
                BUSINESS_EMAIL_DOMAINS[fastrand::usize(0..BUSINESS_EMAIL_DOMAINS.len())]
            ),
            _ => String::new(),
        };

        let domain =
            PUBLIC_EMAIL_DOMAINS[fastrand::usize(0..PUBLIC_EMAIL_DOMAINS.len())].to_string();

        self.build_email(username, domain)
    }

    pub fn standard_business_email(&self) -> String {
        let domain =
            BUSINESS_EMAIL_DOMAINS[fastrand::usize(0..BUSINESS_EMAIL_DOMAINS.len())].to_string();

        self.build_email(self.username(), domain)
    }

    pub fn standard_government_email(&self) -> String {
        let domain = GOVERNMENT_EMAIL_DOMAINS[fastrand::usize(0..GOVERNMENT_EMAIL_DOMAINS.len())]
            .to_string();

        //Some government emails end with .gov domain

        self.build_email(self.username(), domain)
    }

    pub fn standard_account_email(&self) -> String {
        let domain =
            PUBLIC_EMAIL_DOMAINS[fastrand::usize(0..PUBLIC_EMAIL_DOMAINS.len())].to_string();
        self.build_email(self.username(), domain)
    }

    fn build_email(&self, username: String, domain: String) -> String {
        format!("{username}@{domain}.com",)
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

    pub fn ipv4_with_mask(&self) -> String {
        let mut ip = self.ipv4();
        ip.push('/');
        ip.push_str(fastrand::u8(1..=32).to_string().as_str());
        ip
    }

    pub fn ipv4_public(&self) -> String {
        loop {
            let ip = self.ipv4();
            if Self::is_private_network(&ip) || Self::is_reserved_network(&ip) {
                continue;
            }
            return ip;
        }
    }

    fn is_private_network(ip: &str) -> bool {
        for elem in PRIVATE_RANGES_REGEX.iter() {
            if (elem).is_match(ip) {
                return true;
            }
        }
        false
    }

    fn is_reserved_network(ip: &str) -> bool {
        for elem in RESERVED_RANGES_REGEX.iter() {
            if (elem).is_match(ip) {
                return true;
            }
        }
        false
    }

    pub fn ipv4_private(&self) -> String {
        let ranges = PRIVATE_RANGES[fastrand::usize(0..PRIVATE_RANGES.len())];
        let mut slist = vec![];
        for i in ranges {
            if i.start == i.end {
                slist.push(i.start.to_string());
                continue;
            }
            slist.push(fastrand::i32(i.start..i.end).to_string());
        }
        slist.join(".")
    }

    pub fn ipv6(&self) -> String {
        let mut slist: Vec<String> = vec![];
        for _ in 0..8 {
            slist.push(format!("{:x}", fastrand::u16(..)));
        }
        slist.join(":")
    }

    pub fn ipv6_with_mask(&self) -> String {
        let mut ip = self.ipv6();
        ip.push('/');
        ip.push_str(fastrand::u8(1..=127).to_string().as_str());
        ip
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

    pub fn mac(&self) -> String {
        (0..6).fold(String::new(), |mut s, _| {
            s.push_str(&format!("{:02x}", fastrand::u8(..)));
            s
        })
    }

    pub fn port(&self) -> String {
        fastrand::u16(1025..).to_string()
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
