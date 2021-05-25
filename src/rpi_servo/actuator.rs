use std::{thread::{self, sleep}, time::Duration};

use doorman::interfaces::services::{self, ServiceError};

use log::error;
use rppal::pwm::{self, Channel, Pwm};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ActuatorError {
    #[error("PWM failed to initialize：{0}")]
    PwmInit(pwm::Error),

    #[error("Could not control pwm: {0}")]
    PwmControl(pwm::Error),
}

impl ServiceError for ActuatorError {}

pub struct Actuator {
    pwm: Pwm,
}

impl Actuator {
    pub fn new() -> Result<Self, ActuatorError> {
        match Self::init_pwm().map_err(ActuatorError::PwmInit) {
            Ok(pwm) => Ok(Self { pwm }),
            Err(why) => {
                error!("PWM error: {}", why);
                Err(why)
            }
        }
    }

    fn init_pwm() -> Result<Pwm, pwm::Error> {
        let pwm = Pwm::new(Channel::Pwm1)?;
        pwm.set_period(Duration::from_millis(20))?;
        pwm.set_pulse_width(Duration::from_micros(1_500))?;
        pwm.enable()?;


        // Sleep for 500 ms while the servo moves into position.
        thread::sleep(Duration::from_millis(500));

        Ok(pwm)
    }
}

impl services::Actuator for Actuator {
    type ActuatorError = ActuatorError;

    fn open(&mut self) -> Result<(), Self::ActuatorError> {
        self.pwm.set_pulse_width(Duration::from_micros(1_000)).map_err(ActuatorError::PwmControl)?;
        self.pwm.enable().map_err(ActuatorError::PwmControl)?;
        sleep(Duration::from_secs(2));
        self.pwm.set_pulse_width(Duration::from_micros(1_500)).map_err(ActuatorError::PwmControl)?;
        Ok(())
    }
}
