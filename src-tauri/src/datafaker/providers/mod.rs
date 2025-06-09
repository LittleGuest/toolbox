mod address;
mod area;
mod datetime;
mod education;
mod emoji;
mod file;
mod internet;
mod number;
mod person;
mod uuid;

pub use address::Address;
pub use education::Education;
pub use emoji::Emoji;
pub use file::File;
pub use internet::Internet;
pub use number::Number;
pub use person::*;
pub use uuid::Uuid;

fn random_str(count: usize) -> String {
    let mut str = String::with_capacity(count);
    if count == 0 {
        return str;
    }

    (0..count).for_each(|_| {
        let char: char = match fastrand::u8(0..3) {
            0 => fastrand::u8(b'a'..=b'z').into(),
            1 => fastrand::u8(b'A'..=b'Z').into(),
            _ => fastrand::u8(48..=57).into(),
        };
        str.push(char);
    });

    str
}
