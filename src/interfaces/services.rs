use std::error::Error;

use async_trait::async_trait;
use serenity::futures::future::Select;

pub trait ServiceError: Error + std::fmt::Debug + Send + Sync {}
pub trait Detector {
    type Device;
    type DetectorError: ServiceError;

    /// Detect a device synchronously
    fn wait_for_device(&self) -> Result<Self::Device, Self::DetectorError>;
}

pub trait Registry {
    type Device: Eq;
    type RegistryError: ServiceError;

     /// Register a new device
     fn register_device(&mut self, device: Self::Device) -> Result<(), Self::RegistryError>;

    /// Unregisters an existing device
    /// Returns an error if the device is unknown
    fn unregister_device(&mut self, device: &Self::Device) -> Result<(), Self::RegistryError>;

    /// checks whether devices is registered
    fn registered(&self, device: &Self::Device) -> bool;
}


pub enum AuthenticateResult {
    Allow,
    Deny
}

#[async_trait]
pub trait Authenticate {
    type Device;
    type AuthenticateError: ServiceError;

    /// request an authentiation
    async fn authenticate(&self, device: &Self::Device) -> Result<AuthenticateResult, Self::AuthenticateError>;
}


pub trait Actuator {
    type ActuatorError: ServiceError;

    /// Actuate the opening mechanism
    fn open(&mut self) -> Result<(), Self::ActuatorError>;
}
