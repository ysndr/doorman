use std::error::Error;

use async_trait::async_trait;

pub trait ServiceError: Error + std::fmt::Debug + Send + Sync {}

#[async_trait]
pub trait Detector {
    type Device;
    type DetectorError: ServiceError;

    /// Detect a device asynchronously
    async fn wait_for_device(&self) -> Result<&Self::Device, Self::DetectorError>;
}

pub trait Registry {
    type Ident;
    type Device;
    type RegistryError: ServiceError;

    /// Register a new device
    fn register_device_with(&mut self, ident: Self::Ident, device: Self::Device) -> Result<(), Self::RegistryError>;

    /// Register a new device deriving the identifier from the divice
    /// Prefer using this method of there are no naming conflicts
    fn register_device<D: Into<Self::Ident> + Into<Self::Device> + Clone>(&mut self, device: D) -> Result<(), Self::RegistryError> {
        self.register_device_with(device.clone().into(), device.into())
    }


    /// Unregisters an existing device with a given ident
    /// Returns an error if the device is unknown
    fn unregister_device(&mut self, ident: &Self::Ident) -> Result<(), Self::RegistryError>;

    /// checks whether devices is registered
    /// returns the device or None
    fn check(&self, ident: &Self::Ident) -> Option<&Self::Device>;
}


#[derive(Debug)]
pub enum AuthenticateResult {
    Allow,
    Deny
}

#[async_trait]
pub trait Authenticate {
    type Device;
    type AuthenticateError: ServiceError;

    /// request an authentiation
    async fn authenticate(&self, device: &Self::Device, timeout: Option<usize>) -> Result<AuthenticateResult, Self::AuthenticateError>;
}


pub trait Actuator {
    type ActuatorError: ServiceError;

    /// Actuate the opening mechanism
    fn open(&mut self) -> Result<(), Self::ActuatorError>;
}
