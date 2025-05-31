#![allow(dead_code)]

use std::collections::HashMap;

use providers::{Address, Email, Emoji, Internet, Name, PhoneNumber, Uuid};

mod providers;

#[derive(Default, Clone, Copy)]
pub enum Locale {
    CN,
    #[default]
    En,
}

pub trait Provider {
    fn name(&self) -> String;
}

pub struct Faker {
    pub locale: Locale,
    providers: HashMap<String, Box<dyn Provider>>,
}

impl Faker {
    pub fn new() -> Self {
        Self {
            locale: Default::default(),
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, key: String, value: Box<dyn Provider>) {
        self.providers.insert(key, value);
    }

    pub fn providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect::<Vec<_>>()
    }

    pub fn address(&self) -> Address {
        Address::new_with_locale(self.locale)
    }

    pub fn email(&self) -> Email {
        Email::new_with_locale(self.locale)
    }

    pub fn emoji(&self) -> Emoji {
        Emoji
    }

    pub fn internet(&self) -> Internet {
        Internet::new_with_locale(self.locale)
    }

    pub fn name(&self) -> Name {
        Name::new_with_locale(self.locale)
    }

    pub fn phone_number(&self) -> PhoneNumber {
        PhoneNumber::new_with_locale(self.locale)
    }

    pub fn uuid(&self) -> Uuid {
        Uuid
    }
}
