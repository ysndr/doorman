use super::device::SimpleDevice;
use doorman::interfaces::services::{self, Registry, ServiceError};
use std::io::{self, BufRead};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DetectorError {
    #[error("EOL without device found")]
    EOLError
}

impl ServiceError for DetectorError {}

pub struct Detector<'a, Reg: Registry<Device = SimpleDevice>> {
    registry: &'a Reg,
}

impl<'a, Reg: Registry<Device = SimpleDevice>> Detector<'a, Reg> {
    pub fn new(registry: &'a Reg) -> Self { Self { registry } }
}

impl<'a, Reg: Registry<Device = SimpleDevice>> services::Detector for Detector<'a, Reg> {
    type Device = SimpleDevice;
    type DetectorError = DetectorError;

    fn wait_for_device(&self) -> Result<Self::Device, Self::DetectorError> {
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
