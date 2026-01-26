use std::{fmt::Display, ops::Deref, sync::LazyLock};

use super::random_str;
use crate::{FakerData, Locale, Provider};

static QQ_NICK_NAME_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let area = FakerData::get("qq-nick-name").unwrap();
    String::from_utf8_lossy(&area.data)
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect::<Vec<_>>()
});
static SPECIAL_CHARS: [&str; 16] = [
    "!", ".", "_", "@", "#", "$", "%", "^", "&", ",", "(", ")", "`", "[", "]", "*",
];

static MOBILE_PREFIX: [&str; 7] = ["13", "147", "15", "16", "17", "18", "19"];

pub enum Sex {
    Female,
    Male,
}

impl Display for Sex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sex::Female => f.write_str("female"),
            Sex::Male => f.write_str("male"),
        }
    }
}

pub struct Person {
    locale: Locale,
}

impl Provider for Person {
    fn name(&self) -> String {
        std::any::type_name_of_val(self).into()
    }
}

impl Person {
    pub fn new() -> Self {
        Self {
            locale: Default::default(),
        }
    }

    pub fn new_with_locale(locale: Locale) -> Self {
        Self { locale }
    }

    pub fn gender(&self) -> String {
        let mut s = [Sex::Male, Sex::Female];
        fastrand::shuffle(&mut s);
        format!("{}", s[0])
    }

    pub fn qq(&self) -> String {
        fastrand::u64(10000..=100000000000).to_string()
    }

    pub fn qq_nick_name(&self) -> String {
        let name = QQ_NICK_NAME_DATA.deref();
        name.get(fastrand::usize(0..name.len())).unwrap().into()
    }

    pub fn nick_name(&self) -> String {
        random_str(fastrand::usize(4..10))
    }

    pub fn strong_password(&self) -> String {
        let cap = fastrand::usize(8..20);
        let mut pwd = Vec::with_capacity(cap);
        let count = cap / 3;
        (0..count).for_each(|_| {
            pwd.push(random_str(1));
            pwd.push(random_str(1));
        });

        (0..fastrand::usize(1..count)).for_each(|_| {
            pwd.push(SPECIAL_CHARS[fastrand::usize(0..SPECIAL_CHARS.len())].into());
        });

        if pwd.len() < pwd.capacity() {
            pwd.push(random_str(cap - pwd.len()));
        }
        fastrand::shuffle(&mut pwd);
        pwd.join("")
    }

    pub fn mobile(&self) -> String {
        let prefix = MOBILE_PREFIX[fastrand::usize(0..MOBILE_PREFIX.len())];
        let mut mobile = String::with_capacity(11);
        mobile.push_str(prefix);
        (0..mobile.capacity() - prefix.len()).for_each(|_| {
            mobile.push(fastrand::u8(0..10).into());
        });
        mobile
    }

    pub fn id_card(&self) -> String {
        todo!()
    }
}

pub enum CreditCardType {
    Visa,
    MasterCard,
    Amex,
    UnionPay,
    Jcb,
}
