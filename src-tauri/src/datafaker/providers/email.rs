use super::Internet;
use crate::datafaker::Locale;

pub struct Email {
    pub locale: Locale,
}

impl Email {
    pub fn new() -> Self {
        Self {
            locale: Default::default(),
        }
    }

    pub fn new_with_locale(locale: Locale) -> Self {
        Self { locale }
    }

    pub fn standard_generic_email(&self) -> String {
        match fastrand::u8(0..4) {
            0 => self.standard_public_email_alias(),
            1 => self.standard_public_email(),
            2 => self.standard_business_email(),
            3 => self.standard_government_email(),
            _ => "".to_string(),
        }
    }

    pub fn standard_public_email(&self) -> String {
        let domain = PUBLIC_EMAIL_DOMAINS[fastrand::usize(0..PUBLIC_EMAIL_DOMAINS_LEN)].to_string();
        self.build_email(self.username(), domain)
    }

    pub fn standard_public_email_alias(&self) -> String {
        let username = match fastrand::u8(0..2) {
            0 => format!("{}+{}", self.username(), fastrand::u8(0..100)),
            1 => format!(
                "{}+{}",
                self.username(),
                BUSINESS_EMAIL_DOMAINS[fastrand::usize(0..BUSINESS_EMAIL_DOMAINS_LEN)]
            ),
            _ => "".to_string(),
        };

        let domain = PUBLIC_EMAIL_DOMAINS[fastrand::usize(0..PUBLIC_EMAIL_DOMAINS_LEN)].to_string();

        self.build_email(username, domain)
    }

    pub fn standard_business_email(&self) -> String {
        let domain =
            BUSINESS_EMAIL_DOMAINS[fastrand::usize(0..BUSINESS_EMAIL_DOMAINS_LEN)].to_string();

        self.build_email(self.username(), domain)
    }

    pub fn standard_government_email(&self) -> String {
        let domain =
            GOVERNMENT_EMAIL_DOMAINS[fastrand::usize(0..GOVERNMENT_EMAIL_DOMAINS_LEN)].to_string();

        //Some government emails end with .gov domain

        self.build_email(self.username(), domain)
    }

    pub fn standard_account_email(&self, first_name: &str, last_name: &str) -> String {
        let domain = PUBLIC_EMAIL_DOMAINS[fastrand::usize(0..PUBLIC_EMAIL_DOMAINS_LEN)].to_string();
        self.build_email(self.username(), domain)
    }

    fn build_email(&self, username: String, domain: String) -> String {
        format!("{username}@{domain}.com",)
    }

    fn username(&self) -> String {
        Internet::new_with_locale(self.locale).username()
    }
}

static PUBLIC_EMAIL_DOMAINS: [&str; 6] =
    ["outlook", "hotmail", "gmail", "yahoo", "protonmail", "zoho"];
static PUBLIC_EMAIL_DOMAINS_LEN: usize = PUBLIC_EMAIL_DOMAINS.len();
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
static BUSINESS_EMAIL_DOMAINS_LEN: usize = BUSINESS_EMAIL_DOMAINS.len();
static GOVERNMENT_EMAIL_DOMAINS: [&str; 4] = ["fbi", "cia", "nsa", "gov"];
static GOVERNMENT_EMAIL_DOMAINS_LEN: usize = GOVERNMENT_EMAIL_DOMAINS.len();
