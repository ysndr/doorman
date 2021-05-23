use std::{collections::HashMap, fs::File, hash::Hash, io::BufReader, path::PathBuf};

use crate::interfaces::services::{self, Registry as RegistryTrait, ServiceError};
use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;

#[derive(Debug, Default)]
pub struct Registry<Ident: Hash + Eq, Device> {
    devices: HashMap<Ident, Device>,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Specified Device Not registered")]
    NotFoundError,

    #[error("Error reading devices list: {0}")]
    IO(#[from] std::io::Error),

    #[error("Error parsing devices list: {0}")]
    Parse(String),
}
impl ServiceError for RegistryError {}

impl<Ident: Hash + Eq, Device> services::Registry for Registry<Ident, Device> {
    type Ident = Ident;
    type Device = Device;
    type RegistryError = RegistryError;

    fn register_device_with(
        &mut self,
        ident: Self::Ident,
        device: Self::Device,
    ) -> Result<(), Self::RegistryError> {
        self.devices.insert(ident, device);
        Ok(())
    }

    fn unregister_device(&mut self, ident: &Self::Ident) -> Result<(), Self::RegistryError> {
        if self.devices.remove(&ident).is_none() {
            return Err(RegistryError::NotFoundError);
        }
        Ok(())
    }

    fn check(&self, ident: &Self::Ident) -> Option<&Self::Device> {
        self.devices.get(ident)
    }

    fn list(&self) -> Vec<&Self::Device> {
        self.devices.values().collect::<Vec<&Self::Device>>()
    }
}

impl<Ident: Hash + Eq, D> Registry<Ident, D> {
    pub fn new() -> Self {
        Registry {
            devices: HashMap::new(),
        }
    }
}

impl<Ident: Hash + Eq, D: Clone + DeserializeOwned + Into<Ident>> Registry<Ident, D> {
    pub fn from_file(
        &mut self,
        path: PathBuf,
    ) -> Result<(), <Self as RegistryTrait>::RegistryError> {
        let devices: Vec<<Self as RegistryTrait>::Device> = {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).map_err(|e| RegistryError::Parse(e.to_string()))?
        };

        self.from_list(devices)
    }
}
