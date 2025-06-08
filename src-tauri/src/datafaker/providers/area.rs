use std::{collections::HashMap, ops::Deref, sync::LazyLock};

use serde::Deserialize;

use crate::datafaker::{FakerData, Locale};

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
            let x = fastrand::u8(0..11);
            if x % 3 == 0 {
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
    use super::*;

    #[test]
    fn test_address() {}
}

// use crate::datafaker::Locale;

// pub struct PhoneNumber {
//     locale: Locale,
// }

// impl PhoneNumber {
//     pub fn new() -> Self {
//         Self {
//             locale: Default::default(),
//         }
//     }

//     pub fn new_with_locale(locale: Locale) -> Self {
//         Self { locale }
//     }

//     pub fn phone_number_with_country_code(&self) -> String {
//         format!(
//             "+{} {}",
//             COUNTRYCODES[fastrand::usize(0..COUNTRYCODES_LEN)],
//             self.phone_number()
//         )
//     }

//     pub fn phone_number(&self) -> String {
//         let format = fastrand::u8(0..6);

//         let exchange_code = format!(
//             "{}{}{}",
//             fastrand::u8(0..10),
//             fastrand::u8(0..10),
//             fastrand::u8(0..10),
//         );

//         let line_code = format!(
//             "{}{}{}{}",
//             fastrand::u8(0..10),
//             fastrand::u8(0..10),
//             fastrand::u8(0..10),
//             fastrand::u8(0..10),
//         );

//         match format {
//             0 => format!(
//                 "{}{}{}",
//                 AREACODES[fastrand::usize(0..AREACODES_LEN)],
//                 exchange_code,
//                 line_code
//             ),

//             1 => format!(
//                 "{} {} {}",
//                 AREACODES[fastrand::usize(0..AREACODES_LEN)],
//                 exchange_code,
//                 line_code
//             ),

//             2 => format!(
//                 "{}-{}-{}",
//                 AREACODES[fastrand::usize(0..AREACODES_LEN)],
//                 exchange_code,
//                 line_code
//             ),

//             3 => format!(
//                 "({}) {}-{}",
//                 AREACODES[fastrand::usize(0..AREACODES_LEN)],
//                 exchange_code,
//                 line_code
//             ),

//             4 => format!(
//                 "{}.{}.{}",
//                 AREACODES[fastrand::usize(0..AREACODES_LEN)],
//                 exchange_code,
//                 line_code
//             ),

//             5 => format!(
//                 "({}) {} {}",
//                 AREACODES[fastrand::usize(0..AREACODES_LEN)],
//                 exchange_code,
//                 line_code
//             ),

//             _ => String::new(),
//         }
//     }
// }

// static AREACODES: [&str; 290] = [
//     "907", "205", "251", "256", "334", "479", "501", "870", "480", "520", "602", "623", "928",
//     "209", "213", "310", "323", "408", "415", "510", "530", "559", "562", "619", "626", "650",
//     "661", "707", "714", "760", "805", "818", "831", "909", "916", "925", "949", "951", "303",
//     "719", "970", "203", "860", "202", "302", "239", "305", "321", "352", "386", "407", "561",
//     "727", "772", "813", "850", "863", "904", "941", "954", "229", "404", "478", "706", "770",
//     "912", "808", "319", "515", "563", "641", "712", "208", "217", "309", "312", "618", "630",
//     "708", "773", "815", "847", "219", "260", "317", "574", "765", "812", "316", "620", "785",
//     "913", "270", "502", "606", "859", "225", "318", "337", "504", "985", "413", "508", "617",
//     "781", "978", "301", "410", "207", "231", "248", "269", "313", "517", "586", "616", "734",
//     "810", "906", "989", "218", "320", "507", "612", "651", "763", "952", "314", "417", "573",
//     "636", "660", "816", "228", "601", "662", "406", "252", "336", "704", "828", "910", "919",
//     "701", "308", "402", "603", "201", "609", "732", "856", "908", "973", "505", "575", "702",
//     "775", "212", "315", "516", "518", "585", "607", "631", "716", "718", "845", "914", "216",
//     "330", "419", "440", "513", "614", "740", "937", "405", "580", "918", "503", "541", "215",
//     "412", "570", "610", "717", "724", "814", "401", "803", "843", "864", "605", "423", "615",
//     "731", "865", "901", "931", "210", "214", "254", "281", "325", "361", "409", "432", "512",
//     "713", "806", "817", "830", "903", "915", "936", "940", "956", "972", "979", "435", "801",
//     "276", "434", "540", "703", "757", "804", "802", "206", "253", "360", "425", "509", "262",
//     "414", "608", "715", "920", "304", "307", "204", "226", "236", "249", "250", "263", "289",
//     "306", "343", "354", "365", "367", "368", "403", "416", "418", "431", "437", "438", "450",
//     "468", "474", "506", "514", "519", "548", "579", "581", "584", "587", "604", "613", "639",
//     "647", "672", "683", "705", "709", "742", "753", "778", "780", "782", "807", "819", "825",
//     "867", "873", "902", "905",
// ];
// static AREACODES_LEN: usize = AREACODES.len();

// static COUNTRYCODES: [&str; 29] = [
//     "1", "86", "91", "7", "81", "44", "49", "82", "55", "33", "92", "90", "62", "39", "34", "84",
//     "20", "30", "62", "63", "64", "65", "66", "852", "46", "41", "55", "54", "31",
// ];
// static COUNTRYCODES_LEN: usize = COUNTRYCODES.len();

// #[cfg(test)]
// mod tets {
//     use super::*;

//     #[test]
//     fn test_phone_number() {
//         let pn = PhoneNumber::new().phone_number();
//         println!("{pn}");
//     }
// }
