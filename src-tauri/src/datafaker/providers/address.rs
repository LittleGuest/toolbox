pub struct AddressProvider {
    locale: String,
}
impl AddressProvider {
    pub fn new() -> Self {
        Self {
            locale: "en".into(),
        }
    }
}
