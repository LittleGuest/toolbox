pub struct InternetProvider;

impl InternetProvider {
    pub fn ipv4(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
            fastrand::u8(..),
        )
    }
}
