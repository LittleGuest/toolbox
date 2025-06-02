pub struct Number;

impl Number {
    pub fn random_digit(&self) -> u8 {
        fastrand::u8(0..10)
    }

    pub fn random_digit_without_zero(&self) -> u8 {
        fastrand::u8(1..10)
    }

    pub fn positive(&self) -> i32 {
        fastrand::i32(0..)
    }

    pub fn negative(&self) -> i32 {
        fastrand::i32(i32::MIN..=0)
    }

    pub fn random_double(&self) -> f64 {
        fastrand::f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert!(Number.positive() > 0);
        assert!(Number.negative() <= 0);
    }
}
