use rand::Rng;

pub struct Ip;

impl Ip {
    pub fn ipv4() -> String {
        let mut rng = rand::rng();
        format!(
            "{}.{}.{}.{}",
            rng.random_range(0..u8::MAX),
            rng.random_range(0..u8::MAX),
            rng.random_range(0..u8::MAX),
            rng.random_range(0..u8::MAX),
        )
    }
}
