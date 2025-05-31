use uuid::Uuid as UUID;

use super::Name;

pub struct Uuid;

impl Uuid {
    pub fn uuid_v1(&self) -> String {
        UUID::now_v1(&[1, 2, 3, 4, 5, 6]).to_string()
    }

    pub fn uuid_v3(&self) -> String {
        UUID::new_v3(&UUID::NAMESPACE_DNS, self.name().as_bytes()).to_string()
    }

    pub fn uuid_v4(&self) -> String {
        UUID::new_v4().to_string()
    }

    pub fn uuid_v5(&self) -> String {
        UUID::new_v5(&UUID::NAMESPACE_DNS, self.name().as_bytes()).to_string()
    }

    pub fn uuid_v6(&self) -> String {
        // FIXME: slice_as_array unstable
        // fastrand::choose_multiple(0..u8::MAX, 6)
        //     .as_array::<16>()
        //     .unwrap();
        UUID::now_v6(&[
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
        ])
        .to_string()
    }

    pub fn uuid_v7(&self) -> String {
        UUID::now_v7().to_string()
    }

    pub fn uuid_v8(&self) -> String {
        // FIXME: slice_as_array unstable
        // fastrand::choose_multiple(0..u8::MAX, 16)
        //     .as_array::<16>()
        //     .unwrap();
        UUID::new_v8([
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
        ])
        .to_string()
    }

    fn name(&self) -> String {
        Name::new().last_name()
    }
}

#[test]
fn test() {
    let uuid = Uuid;
    println!("{}", uuid.uuid_v1());
    println!("{}", uuid.uuid_v3());
    println!("{}", uuid.uuid_v4());
    println!("{}", uuid.uuid_v5());
    println!("{}", uuid.uuid_v6());
    println!("{}", uuid.uuid_v7());
    println!("{}", uuid.uuid_v8());
}
