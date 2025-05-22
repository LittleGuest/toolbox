//! UUID生成器

use uuid::Uuid;

use crate::{Error, Result};

pub fn uuid1() -> String {
    Uuid::now_v1(&[8, 8, 8, 8, 8, 8]).to_string()
}

pub fn uuid3(namespace: &str, name: &str) -> Result<String> {
    Ok(Uuid::new_v3(
        &Uuid::from_slice(namespace.as_bytes()).map_err(|e| Error::UuidErr(e.to_string()))?,
        name.as_bytes(),
    )
    .to_string())
}

pub fn uuid4() -> String {
    Uuid::new_v4().to_string()
}

pub fn uuid5(namespace: &str, name: &str) -> Result<String> {
    Ok(Uuid::new_v5(
        &Uuid::from_slice(namespace.as_bytes()).map_err(|e| Error::UuidErr(e.to_string()))?,
        name.as_bytes(),
    )
    .to_string())
}
