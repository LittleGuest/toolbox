use providers::{AddressProvider, EmojiProvider, InternetProvider, NameProvider};

mod providers;

pub struct Faker {
    locale: String,
}

impl Faker {
    pub fn new() -> Self {
        Self {
            locale: "en".into(),
        }
    }

    pub fn address(&self) -> AddressProvider {
        AddressProvider::new()
    }

    pub fn internet(&self) -> InternetProvider {
        InternetProvider
    }

    pub fn name(&self) -> NameProvider {
        NameProvider::new()
    }

    pub fn emoji(&self) -> EmojiProvider {
        EmojiProvider
    }
}

#[cfg(test)]
mod tests {
    use super::Faker;

    #[test]
    fn test_internet() {
        let ipv4 = Faker::new().internet().ipv4();
        println!("test_internet => {ipv4}");
        assert!(!ipv4.is_empty())
    }
}
