use std::{ops::Deref, sync::LazyLock};

use super::Name;
use crate::datafaker::{FakerData, Locale};

static STREET_SUFFIX_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let area = FakerData::get("street-suffix").unwrap();
    String::from_utf8_lossy(&area.data)
        .lines()
        .map(String::from)
        .collect::<Vec<_>>()
});

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

    pub fn city_prefix(&self) -> String {
        CITYPREFIXES[fastrand::usize(0..CITYPREFIXES_LEN)].to_string()
    }

    pub fn city_suffix(&self) -> String {
        CITYSUFFIXES[fastrand::usize(0..CITYSUFFIXES_LEN)].to_string()
    }

    pub fn street_suffix(&self) -> String {
        let name = STREET_SUFFIX_DATA.deref();
        name.get(fastrand::usize(0..name.len())).unwrap().into()
    }

    pub fn street_name(&self) -> String {
        let mut name = Name::new_with_locale(rand::rng(), self.locale);
        match fastrand::u8(0..2) {
            0 => format!("{} {}", name.first_name(), self.street_suffix()),
            1 => format!("{} {}", name.last_name(), self.street_suffix()),
            _ => String::new(),
        }
    }

    pub fn street_address(&self) -> String {
        match fastrand::usize(3..6) {
            5 => format!("{} {}", fastrand::u32(10000..99999), self.street_name()),
            4 => format!("{} {}", fastrand::u32(1000..9999), self.street_name()),
            3 => format!("{} {}", fastrand::u32(100..999), self.street_name()),
            _ => String::new(),
        }
    }

    pub fn full_address(&self) -> String {
        let state = self.state();
        let zipcode = self.zip_code(&state);
        match fastrand::u8(0..2) {
            0 => format!(
                "{}, {}, {} {}",
                self.street_address(),
                self.city(),
                state,
                zipcode
            ),
            1 => format!(
                "{} {}, {}, {} {}",
                self.street_address(),
                self.secondary_address(),
                self.city(),
                state,
                zipcode
            ),
            _ => String::new(),
        }
    }

    pub fn state(&self) -> String {
        STATES[fastrand::usize(0..STATES_LEN)].to_string()
    }

    pub fn city(&self) -> String {
        let mut name = Name::new_with_locale(rand::rng(), self.locale);
        match fastrand::u8(0..4) {
            0 => format!(
                "{} {}{}",
                self.city_prefix(),
                name.first_name(),
                self.city_suffix()
            ),
            1 => format!("{} {}", self.city_prefix(), name.first_name()),
            2 => format!("{}{}", name.first_name(), self.city_suffix()),
            3 => format!("{}{}", name.last_name(), self.city_suffix()),
            _ => String::new(),
        }
    }

    pub fn zip_code(&self, state: &str) -> String {
        let zip_format = match state {
            "AL" => "350",
            "AK" => "995",
            "AS" => "967",
            "AZ" => "850",
            "AR" => "717",
            "CA" => "900",
            "CO" => "800",
            "CT" => "061",
            "DC" => "204",
            "DE" => "198",
            "FL" => "322",
            "GA" => "301",
            "HI" => "967",
            "ID" => "832",
            "IL" => "600",
            "IN" => "463",
            "IA" => "510",
            "KS" => "666",
            "KY" => "404",
            "LA" => "701",
            "ME" => "042",
            "MD" => "210",
            "MA" => "026",
            "MI" => "480",
            "MN" => "555",
            "MS" => "387",
            "MO" => "650",
            "MT" => "590",
            "NE" => "688",
            "NV" => "898",
            "NH" => "036",
            "NJ" => "076",
            "NM" => "880",
            "NY" => "122",
            "NC" => "288",
            "ND" => "586",
            "OH" => "444",
            "OK" => "730",
            "OR" => "979",
            "PA" => "186",
            "RI" => "029",
            "SC" => "299",
            "SD" => "577",
            "TN" => "383",
            "TX" => "798",
            "UT" => "847",
            "VT" => "050",
            "VA" => "222",
            "WA" => "990",
            "WV" => "247",
            "WI" => "549",
            "WY" => "831",
            _ => "",
        };

        format!("{}{}", zip_format, fastrand::u8(10..99))
    }

    pub fn secondary_address(&self) -> String {
        format!(
            "{} {}",
            SECONDARY_FORMATS[fastrand::usize(0..SECONDARY_FORMATS_LEN)],
            fastrand::u16(100..999)
        )
    }
}

static CITYPREFIXES: [&str; 7] = ["North", "East", "West", "South", "New", "Lake", "Port"];
static CITYPREFIXES_LEN: usize = CITYPREFIXES.len();

static CITYSUFFIXES: [&str; 19] = [
    "town", "ton", "land", "ville", "berg", "burgh", "borough", "bury", "view", "port", "mouth",
    "stad", "furt", "chester", "mouth", "fort", "haven", "side", "shire",
];
static CITYSUFFIXES_LEN: usize = CITYSUFFIXES.len();

static STATES: [&str; 50] = [
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
    "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY",
    "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV",
    "WI", "WY",
];
static STATES_LEN: usize = STATES.len();

static SECONDARY_FORMATS: [&str; 2] = ["Suite", "Apt."];
static SECONDARY_FORMATS_LEN: usize = SECONDARY_FORMATS.len();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_city() {
        println!("{}", Address::new().city());
    }
}
