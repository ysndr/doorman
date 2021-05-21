use crate::interfaces::{
    self,
    services::{Actuator, Authenticate, Detector, ServiceError},
};
use log::{info, log};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManagerError<
    DetectError: ServiceError,
    AuthenticateError: ServiceError,
    ActError: ServiceError,
> {
    #[error("Something happened")]
    General,
    #[error("Detector experienced an Error: {0}")]
    Detector(DetectError),
    #[error("Authenticator experienced an Error: {0}")]
    Authenticate(AuthenticateError),
    #[error("Actuator experienced an Error: {0}")]
    Actuate(ActError),
}

pub struct Manager<'a, Detect, Auth, Act, Device>
where
    Detect: Detector<Device = Device>,
    Auth: Authenticate<Device = Device>,
    Act: Actuator,
{
    detector: &'a Detect,
    auth: &'a Auth,
    act: &'a mut Act,
}

impl<'a, Detect, Auth, Act, Device> Manager<'a, Detect, Auth, Act, Device>
where
    Device: std::fmt::Debug,
    Detect: Detector<Device = Device>,
    Auth: Authenticate<Device = Device>,
    Act: Actuator,
{
    pub fn new(detector: &'a Detect, auth: &'a Auth, act: &'a mut Act) -> Self { Self { detector, auth, act } }

    pub async fn run(&mut self) -> Result<(), ManagerError<Detect::DetectorError, Auth::AuthenticateError, Act::ActuatorError>> {

        info!("Waiting for device...");

        let device = self
            .detector
            .wait_for_device()
            .await
            .map_err(ManagerError::Detector)?;

        info!("Device detected attempting authentication...");

        let authentication = self.auth
            .authenticate(&device, None)
            .await
            .map_err(ManagerError::Authenticate)?;

        match authentication {
            interfaces::services::AuthenticateResult::Allow => self.act.open().map_err(ManagerError::Actuate)?,
            _ => info!("Access with device {:?} denied", device)
        };
        Ok(())
    }
}
