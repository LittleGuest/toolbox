use std::{ops::Deref, sync::LazyLock};

use crate::{FakerData, Locale, providers::area::Area};

static COLLEGE_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let area = FakerData::get("college").unwrap();
    String::from_utf8_lossy(&area.data)
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect::<Vec<_>>()
});

static MAJOR_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let area = FakerData::get("college-major").unwrap();
    String::from_utf8_lossy(&area.data)
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect::<Vec<_>>()
});

pub struct Education {
    locale: Locale,
}

impl Education {
    pub fn new() -> Self {
        Self {
            locale: Default::default(),
        }
    }

    pub fn new_with_locale(locale: Locale) -> Self {
        Self { locale }
    }

    pub fn degree(&self) -> String {
        DEGREE[fastrand::usize(0..DEGREE_LEN)].into()
    }

    fn number(&self) -> String {
        NUMBER[fastrand::usize(0..NUMBER_LEN)].into()
    }

    fn school_name(&self, school: &str) -> String {
        let area = Area::new();
        let area = area.area();
        format!("{}第{}{school}", area.city, self.number())
    }

    pub fn primary_school_name(&self) -> String {
        self.school_name("小学")
    }

    pub fn primary_school_grade(&self) -> String {
        format!("{}年级", fastrand::u8(1..7))
    }

    pub fn high_school_name(&self) -> String {
        self.school_name("中学")
    }

    pub fn high_school_grade(&self) -> String {
        let mut grade = fastrand::u8(1..7);
        let prefix = if grade > 3 {
            grade -= 3;
            "高"
        } else {
            "初"
        };
        format!("{prefix}{grade}年级")
    }

    pub fn class_name(&self) -> String {
        format!("{}班", fastrand::u8(1..26))
    }

    pub fn college(&self) -> &str {
        COLLEGE_DATA
            .deref()
            .get(fastrand::usize(0..COLLEGE_DATA.deref().len()))
            .unwrap()
    }

    pub fn major(&self) -> &str {
        MAJOR_DATA
            .deref()
            .get(fastrand::usize(0..MAJOR_DATA.deref().len()))
            .unwrap()
    }
}

static DEGREE: [&str; 8] = [
    "小学",
    "初中",
    "中专/职业高中",
    "高中",
    "大学专科",
    "大学本科",
    "硕士",
    "博士",
];
static DEGREE_LEN: usize = DEGREE.len();

static NUMBER: [&str; 10] = ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"];
static NUMBER_LEN: usize = NUMBER.len();
