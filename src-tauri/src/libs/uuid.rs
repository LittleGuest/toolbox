use uuid::Uuid;

use crate::{Error, Result};

pub fn uuid_v1() -> Result<String> {
    Ok(Uuid::now_v1(&[1, 2, 3, 4, 5, 6]).to_string())
}

pub fn uuid_v3(namespace: &str, name: &str) -> Result<String> {
    Ok(Uuid::new_v3(&Uuid::NAMESPACE_DNS, name.as_bytes())
        .to_string()
        .to_string())
}

pub fn uuid_v4() -> Result<String> {
    Ok(Uuid::new_v4().to_string())
}

pub fn uuid_v5(namespace: &str, name: &str) -> Result<String> {
    Ok(Uuid::new_v5(&Uuid::NAMESPACE_DNS, name.as_bytes()).to_string())
}

pub fn uuid_v6() -> Result<String> {
    // FIXME: slice_as_array unstable
    // fastrand::choose_multiple(0..u8::MAX, 6)
    //     .as_array::<16>()
    //     .unwrap();
    Ok(Uuid::now_v6(&[
        fastrand::u8(..),
        fastrand::u8(..),
        fastrand::u8(..),
        fastrand::u8(..),
        fastrand::u8(..),
        fastrand::u8(..),
    ])
    .to_string())
}

pub fn uuid_v7() -> Result<String> {
    Ok(Uuid::now_v7().to_string())
}

pub fn uuid_v8() -> Result<String> {
    // FIXME: slice_as_array unstable
    // fastrand::choose_multiple(0..u8::MAX, 16)
    //     .as_array::<16>()
    //     .unwrap();
    Ok(Uuid::new_v8([
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
    .to_string())
}
