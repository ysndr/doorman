use std::{
    collections::{HashMap},
    hash::Hash,
};

use crate::interfaces::services::{self, ServiceError};
use thiserror::Error;

#[derive(Debug, Default)]
pub struct Registry<Ident: Hash + Eq, Device> {
    devices: HashMap<Ident, Device>,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Specified Device Not registered")]
    NotFoundError,
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
}

impl<Ident: Hash + Eq, D> Registry<Ident, D> {
    pub fn new() -> Self {
        Registry {
            devices: HashMap::new(),
        }
    }
}
