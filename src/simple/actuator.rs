use doorman::interfaces::services::{self, ServiceError};

use thiserror::Error;


pub struct Actuator;


#[derive(Debug, Error)]
pub enum ActuatorError {

}

impl ServiceError for ActuatorError {}


impl services::Actuator for Actuator {
    type ActuatorError = ActuatorError;

    fn open(&mut self) -> Result<(), Self::ActuatorError> {
        println!("Sesam öffne sich");
        Ok(())
    }
}
