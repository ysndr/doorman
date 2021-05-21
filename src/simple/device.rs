use derive_more::Display;

#[derive(Hash, PartialEq, Eq, Debug, Display, Clone)]
pub struct SimpleDevice(pub String);

impl<S: AsRef<str>> From<S> for SimpleDevice {
    fn from(s: S) -> Self {
        SimpleDevice(s.as_ref().to_string())
    }
}
