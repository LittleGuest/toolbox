use crate::datafaker::{providers::Person, Locale};

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
}
