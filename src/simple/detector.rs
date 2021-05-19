use super::device::SimpleDevice;
use doorman::interfaces::services::{self, Registry, ServiceError};
use std::io::{self, BufRead};
use thiserror::Error;
use async_trait::async_trait;
#[derive(Debug, Error)]
pub enum DetectorError {
    #[error("EOL without device found")]
    EOLError
}

impl ServiceError for DetectorError {}

pub struct Detector<'a, Reg: Registry<Device = SimpleDevice> + Send + Sync> {
    registry: &'a Reg,
}

impl<'a, Reg: Registry<Device = SimpleDevice> + Send + Sync> Detector<'a, Reg> {
    pub fn new(registry: &'a Reg) -> Self { Self { registry } }
}

#[async_trait]
impl<'a, Reg: Registry<Device = SimpleDevice> + Send + Sync> services::Detector for Detector<'a, Reg> {
    type Device = SimpleDevice;
    type DetectorError = DetectorError;

    async fn wait_for_device(&self) -> Result<Self::Device, Self::DetectorError> {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let device = SimpleDevice(line.unwrap());

            if self.registry.registered(&device) {
                return Ok(device);
            };

        }
        Err(DetectorError::EOLError)
    }
}
