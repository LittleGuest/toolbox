use std::{ops::Deref, sync::LazyLock};

use crate::datafaker::{FakerData, Locale};

pub struct AreaData {
    pub province: String,
    pub city: String,
    pub country: String,
    pub zip_code: String,
}

impl From<&str> for AreaData {
    fn from(v: &str) -> Self {
        let s = v.split(",").collect::<Vec<_>>();
        Self {
            province: s[0].into(),
            city: s[1].into(),
            country: s[2].into(),
            zip_code: s[3].into(),
        }
    }
}

static DATA: LazyLock<Vec<AreaData>> = LazyLock::new(|| {
    let area = FakerData::get("area.csv").unwrap();
    String::from_utf8_lossy(&area.data)
        .lines()
        .map(AreaData::from)
        .collect::<Vec<_>>()
});

pub struct Area {
    locale: Locale,
}

impl Area {
    pub fn new() -> Self {
        Self {
            locale: Default::default(),
        }
    }

    pub fn province(&self) -> String {
        self.random().province.clone()
    }

    pub fn random(&self) -> &AreaData {
        DATA.deref()
            .get(fastrand::usize(0..DATA.deref().len()))
            .unwrap()
    }
}
