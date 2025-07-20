mod address;
mod area;
mod datetime;
mod education;
mod emoji;
mod file;
mod internet;
mod name;
mod number;
mod person;
mod regex;
mod sequence;
mod uuid;

pub use address::Address;
pub use education::Education;
pub use emoji::Emoji;
pub use file::File;
pub use internet::Internet;
pub use name::{Name, NameGenerator};
pub use number::{Number, NumberGenerator};
pub use person::*;
pub use sequence::SequenceGenerator;
pub use uuid::Uuid;

/// 随机字符串，count为字符串长度
fn random_str(count: usize) -> String {
    if count == 0 {
        return String::new();
    }
    (0..count).fold(String::with_capacity(count), |mut s: String, _| {
        let char: char = match fastrand::u8(0..3) {
            0 => fastrand::u8(b'a'..=b'z').into(),
            1 => fastrand::u8(b'A'..=b'Z').into(),
            _ => fastrand::u8(48..=57).into(),
        };
        s.push(char);
        s
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_str() {
        let s = random_str(18);
        assert_eq!(s.len(), 18);
    }
}
