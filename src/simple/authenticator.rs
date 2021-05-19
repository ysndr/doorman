use super::device::SimpleDevice;
use doorman::interfaces::services::{self, Authenticate, Registry, ServiceError};
use log::info;
use std::io::{self, BufRead};
use thiserror::Error;
use async_trait::async_trait;
#[derive(Debug, Error)]
pub enum AuthenticatorError {
    #[error("EOL without device found")]
    EOLError
}

impl ServiceError for AuthenticatorError {}

pub struct Authenticator {

}

#[async_trait]
impl services::Authenticate for Authenticator {
    type Device = SimpleDevice;

    type AuthenticateError = AuthenticatorError;

    async fn authenticate(&self, device: &Self::Device, _: Option<usize>) -> Result<services::AuthenticateResult, Self::AuthenticateError> {

        let stdin = io::stdin();
        println!("Device {:?} device detected.\n open (y)es, (N)o", device);


        let mut input = String::new();
        while input.is_empty() {
            stdin.lock().read_line(&mut input).unwrap();

            if ["yes", "y"].contains(&input.trim()) {
                info!("allow");
                return Ok(services::AuthenticateResult::Allow);
            }
            else if  ["no", "n", ""].contains(&input.trim()) {
                info!("deny");

                return Ok(services::AuthenticateResult::Deny);
            }
            input = String::new();
        }

        Err(AuthenticatorError::EOLError)

    }
}
