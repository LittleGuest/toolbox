use crate::datafaker::Locale;

pub struct Address {
    locale: Locale,
}

impl Address {
    pub fn new() -> Self {
        Self {
            locale: Default::default(),
        }
    }

    pub fn new_with_locale(locale: Locale) -> Self {
        Self { locale }
    }
}
