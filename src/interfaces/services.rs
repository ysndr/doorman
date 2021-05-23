use std::{collections::HashMap, error::Error, fmt::Debug};

use async_trait::async_trait;

pub trait ServiceError: Error + std::fmt::Debug + Send + Sync {}

#[async_trait]
pub trait Detector {
    type Device: Debug;
    type DetectorError: ServiceError;

    /// Detect a device asynchronously
    async fn wait_for_device(&self) -> Result<&Self::Device, Self::DetectorError>;
}

pub trait Registry {
    type Ident;
    type Device;
    type RegistryError: ServiceError;

    /// Register a new device
    fn register_device_with(
        &mut self,
        ident: Self::Ident,
        device: Self::Device,
    ) -> Result<(), Self::RegistryError>;

    /// Imports a map of devices
    fn from_map<I: Into<Self::Ident>, D: Into<Self::Device> + Clone>(
        &mut self,
        devices: HashMap<I, D>,
    ) -> Result<(), Self::RegistryError> {
        for (ident, device) in devices {
            self.register_device_with(ident.into(), device.into())?
        }
        Ok(())
    }

    /// Register a new device deriving the identifier from the divice
    /// Prefer using this method of there are no naming conflicts
    fn register_device<D: Into<Self::Ident> + Into<Self::Device> + Clone>(
        &mut self,
        device: D,
    ) -> Result<(), Self::RegistryError> {
        self.register_device_with(device.clone().into(), device.into())
    }

    /// Imports a list of devices
    /// This works as long as an identifier can be derived from the Device
    fn from_list<D: Into<Self::Ident> + Into<Self::Device> + Clone>(
        &mut self,
        devices: impl IntoIterator<Item = D>,
    ) -> Result<(), Self::RegistryError> {
        for device in devices.into_iter() {
            self.register_device(device)?
        }
        Ok(())
    }

    /// Unregisters an existing device with a given ident
    /// Returns an error if the device is unknown
    fn unregister_device(&mut self, ident: &Self::Ident) -> Result<(), Self::RegistryError>;

    /// Checks whether devices is registered
    /// returns the device or None
    fn check(&self, ident: &Self::Ident) -> Option<&Self::Device>;

    /// List all registered devices
    fn list(&self) -> Vec<&Self::Device>;
}

pub trait RegistryKnownType: Registry {
    fn import_list(&mut self, devices: Vec<Self::Device>) -> Result<(), Self::RegistryError>;
}

impl<R> RegistryKnownType for R
where
    Self::Device: From<Self::Device> + Clone,
    Self::Ident: From<Self::Device>,
    R: Registry,
{
    fn import_list(&mut self, devices: Vec<Self::Device>) -> Result<(), Self::RegistryError> {
        self.from_list(devices)
    }
}

#[derive(Debug)]
pub enum AuthenticateResult {
    Allow,
    Deny,
}

#[async_trait]
pub trait Authenticate {
    type Device;
    type AuthenticateError: ServiceError;

    /// request an authentiation
    async fn authenticate(
        &self,
        device: &Self::Device,
        timeout: Option<usize>,
    ) -> Result<AuthenticateResult, Self::AuthenticateError>;
}

pub trait Actuator {
    type ActuatorError: ServiceError;

    /// Actuate the opening mechanism
    fn open(&mut self) -> Result<(), Self::ActuatorError>;
}

#[async_trait]
pub trait Locker {
    type LockerError: ServiceError;

    /// Await engagement of the mechanism
    async fn wait_for_lock(&self) -> Result<(), Self::LockerError>;

    /// Confirm lock to the user
    async fn confirm_lock(&self) -> Result<(), Self::LockerError>;
}
