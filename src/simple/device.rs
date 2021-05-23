use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq, Debug, Display, Clone, Serialize, Deserialize)]
pub struct SimpleDevice(pub String);

impl<S: AsRef<str>> From<S> for SimpleDevice {
    fn from(s: S) -> Self {
        SimpleDevice(s.as_ref().to_string())
    }
}
