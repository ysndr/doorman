use derive_more::Display;

#[derive(Hash, PartialEq, Eq, Debug, Display)]
pub struct SimpleDevice(pub String);
