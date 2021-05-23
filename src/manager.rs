use std::time::Duration;

use crate::interfaces::{
    self,
    services::{Actuator, Authenticate, AuthenticateResult, Detector, Locker, ServiceError},
};
use log::{debug, info};
use thiserror::Error;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum ManagerError<
    DetectError: ServiceError,
    AuthenticateError: ServiceError,
    ActError: ServiceError,
    LockError: ServiceError,
> {
    #[error("Something happened")]
    General,
    #[error("Detector experienced an Error: {0}")]
    Detector(DetectError),
    #[error("Authenticator experienced an Error: {0}")]
    Authenticate(AuthenticateError),
    #[error("Actuator experienced an Error: {0}")]
    Actuate(ActError),
    #[error("Locker experienced an Error: {0}")]
    Lock(LockError),
}

pub struct Manager<'a, Detect, Auth, Act, Lock>
where
    Detect: Detector,
    Auth: Authenticate<Device = Detect::Device>,
    Act: Actuator,
    Lock: Locker,
{
    locker: &'a Lock,
    detector: &'a Detect,
    auth: &'a Auth,
    act: &'a mut Act,
}

impl<'a, Detect, Auth, Act, Lock> Manager<'a, Detect, Auth, Act, Lock>
where
    Detect: Detector,
    Auth: Authenticate<Device = Detect::Device>,
    Act: Actuator,
    Lock: Locker,
{
    pub fn new(detector: &'a Detect, auth: &'a Auth, act: &'a mut Act, locker: &'a Lock) -> Self {
        Self {
            locker,
            detector,
            auth,
            act,
        }
    }

    pub async fn run(
        &mut self,
    ) -> Result<
        AuthenticateResult,
        ManagerError<
            Detect::DetectorError,
            Auth::AuthenticateError,
            Act::ActuatorError,
            Lock::LockerError,
        >,
    > {
        info!("Waiting for device...");

        let device = self
            .detector
            .wait_for_device()
            .await
            .map_err(ManagerError::Detector)?;

        info!("Device detected attempting authentication...");

        let authentication = self
            .auth
            .authenticate(&device, None)
            .await
            .map_err(ManagerError::Authenticate)?;

        match authentication {
            interfaces::services::AuthenticateResult::Allow => {
                self.act.open().map_err(ManagerError::Actuate)?
            }
            _ => info!("Access with device {:?} denied", device),
        };
        Ok(authentication)
    }

    pub async fn daemon(
        &mut self,
    ) -> Result<
        (),
        ManagerError<
            Detect::DetectorError,
            Auth::AuthenticateError,
            Act::ActuatorError,
            Lock::LockerError,
        >,
    > {
        loop {
            self.locker
                .wait_for_lock()
                .await
                .map_err(ManagerError::Lock)?;

            self.locker
                .confirm_lock()
                .await
                .map_err(ManagerError::Lock)?;

            while let AuthenticateResult::Deny = self.run().await? {
                sleep(Duration::from_secs(30)).await;
            }
        }
    }
}
