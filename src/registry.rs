use std::{collections::HashSet, hash::Hash};

use crate::interfaces::services::{self, ServiceError};
use thiserror::Error;

#[derive(Debug, Default)]
pub struct Registry<Device: Hash + Eq> {
    devices: HashSet<Device>,
}


#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Specified Device Not registered")]
    NotFoundError
}
impl ServiceError for RegistryError {}

impl <Device: Hash + Eq> services::Registry for Registry<Device> {
    type Device = Device;
    type RegistryError = RegistryError;

    fn register_device(&mut self, device: Self::Device) -> Result<(), Self::RegistryError> {
        self.devices.insert(device);
        Ok(())
    }

    fn unregister_device(&mut self, device: &Self::Device) -> Result<(), Self::RegistryError> {
        if !self.devices.remove(&device) {
            return Err(RegistryError::NotFoundError)
        }
        Ok(())
    }

    fn registered(&self, device: &Self::Device) -> bool {
        self.devices.contains(device)
    }
}


impl<Device: Hash + Eq> Registry<Device> {
    pub fn new() -> Self {
        Registry {
            devices: HashSet::new()
        }
    }
}
