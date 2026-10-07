use std::{collections::HashMap, ops::Deref, sync::LazyLock};

use serde::Deserialize;

use crate::{FakerData, Locale};

#[derive(Clone)]
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
        .filter(|l| !l.is_empty())
        .map(AreaData::from)
        .collect::<Vec<_>>()
});

static PHONE_CODE_DATA: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| {
    #[derive(Clone, Deserialize)]
    struct PhoneCode {
        area: String,
        code: Vec<String>,
    }

    let phone_code = FakerData::get("phone-code.json").unwrap();
    let phone_code =
        serde_json::from_str::<Vec<PhoneCode>>(&String::from_utf8_lossy(&phone_code.data)).unwrap();
    let size = phone_code.len();
    phone_code
        .into_iter()
        .fold(HashMap::with_capacity(size), |mut map, pc| {
            map.insert(pc.area, pc.code);
            map
        })
});

static ADDRESS_WORD_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let data = FakerData::get("address-word").unwrap();
    String::from_utf8_lossy(&data.data)
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect::<Vec<_>>()
});

static COMMUNITY_NAME_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let data = FakerData::get("community-name").unwrap();
    String::from_utf8_lossy(&data.data)
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect::<Vec<_>>()
});

static COMMUNITY_SUFFIX_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let data = FakerData::get("community-suffix").unwrap();
    String::from_utf8_lossy(&data.data)
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect::<Vec<_>>()
});

static TOWN_SUFFIX: [&str; 2] = ["乡", "镇"];
static DIRECTION: [&str; 5] = ["东", "西", "南", "北", "中"];

pub struct Area {
    locale: Locale,
}

impl Area {
    pub fn new() -> Self {
        Self {
            locale: Default::default(),
        }
    }

    pub fn new_with_locale(locale: Locale) -> Self {
        Self { locale }
    }

    pub fn area(&self) -> AreaData {
        DATA.deref()
            .get(fastrand::usize(0..DATA.deref().len()))
            .cloned()
            .unwrap()
    }

    pub fn zip_code(&self) -> String {
        self.area().zip_code
    }

    pub fn province(&self) -> String {
        self.area().province
    }

    pub fn city(&self, separator: &str) -> String {
        let area = self.area();
        format!("{}{}{}", area.province, separator, area.city)
    }

    pub fn address(&self) -> String {
        self.address_by_area(self.area())
    }

    pub fn address_by_area(&self, area: AreaData) -> String {
        let prefix = format!("{}{}{}", area.province, area.city, area.country);
        let awd = ADDRESS_WORD_DATA.deref();

        if prefix.ends_with("县") || prefix.ends_with("旗") {
            let first = fastrand::usize(0..awd.len());
            let second = fastrand::usize(0..awd.len());
            let third = fastrand::usize(0..TOWN_SUFFIX.len());
            let town = format!(
                "{}{}{}",
                awd.get(first).unwrap(),
                awd.get(second).unwrap(),
                TOWN_SUFFIX.get(third).unwrap()
            );

            let first = fastrand::usize(0..awd.len());
            let second = fastrand::usize(0..awd.len());
            let village = format!("{}{}村", awd.get(first).unwrap(), awd.get(second).unwrap(),);

            let first = fastrand::usize(0..awd.len());
            let second = fastrand::usize(0..awd.len());
            let group = format!("{}{}组", awd.get(first).unwrap(), awd.get(second).unwrap(),);

            format!("{prefix}{town}{village}{group}{}号", fastrand::u8(1..100))
        } else {
            let first = fastrand::usize(0..awd.len());
            let second = fastrand::usize(0..awd.len());
            let third = fastrand::usize(0..DIRECTION.len());
            let road = format!(
                "{}{}{}",
                awd.get(first).unwrap(),
                awd.get(second).unwrap(),
                DIRECTION.get(third).unwrap()
            );

            let first = fastrand::usize(0..COMMUNITY_NAME_DATA.len());
            let second = fastrand::usize(0..COMMUNITY_SUFFIX_DATA.len());
            let community = format!(
                "{}{}",
                COMMUNITY_NAME_DATA.get(first).unwrap(),
                COMMUNITY_SUFFIX_DATA.get(second).unwrap(),
            );
            let mut extra = "";
            if fastrand::u8(0..11).is_multiple_of(3) {
                extra = DIRECTION.get(fastrand::usize(0..DIRECTION.len())).unwrap();
            }

            let building = format!("{}栋", fastrand::u8(1..20));
            let unit = format!("{}单元", fastrand::u8(1..5));
            let room = format!("{:02}{:02}房", fastrand::u8(1..31), fastrand::u8(1..5));
            format!(
                "{prefix}{road}路{}号{community}{extra}{building}{unit}{room}",
                fastrand::u16(0..1000)
            )
        }
    }

    pub fn lat(&self) -> String {
        let start = 3.86;
        let end = 53.55;
        (start + ((end - start) * fastrand::f64())).to_string()
    }

    pub fn lon(&self) -> String {
        let start = 73.66;
        let end = 135.05;
        (start + ((end - start) * fastrand::f64())).to_string()
    }

    pub fn phone_code(&self, province: &str) -> String {
        let province = province
            .replace("省", "")
            .replace("市", "")
            .replace("自治区", "");
        let pcs = PHONE_CODE_DATA.deref();
        if !pcs.contains_key(&province) {
            return String::new();
        }
        let Some(codes) = pcs.get(&province) else {
            return String::new();
        };
        codes
            .get(fastrand::usize(0..codes.len()))
            .cloned()
            .unwrap_or_default()
    }

    pub fn phone_number(&self, province: &str, mut delimiter: &str) -> String {
        let code = self.phone_code(province);
        if delimiter.is_empty() {
            delimiter = " ";
        }
        format!("{code}{delimiter}{}", fastrand::u64(10000000..=99999999))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_address() {}
}
