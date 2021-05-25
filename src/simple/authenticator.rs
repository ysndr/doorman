use async_trait::async_trait;

use doorman::interfaces::services::{self, ServiceError};
use log::info;
use std::{fmt::Display, io::{self, BufRead}, marker::PhantomData, time::Duration};
use thiserror::Error;
#[derive(Debug, Error)]
pub enum AuthenticatorError {
    #[error("EOL without device found")]
    EOLError,
}

impl ServiceError for AuthenticatorError {}

pub struct Authenticator<D: Display> {
    marker: PhantomData<D>,
}

impl<D: Display> Authenticator<D> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

#[async_trait]
impl<D: Display + std::marker::Sync> services::Authenticate for Authenticator<D> {
    type Device = D;

    type AuthenticateError = AuthenticatorError;

    async fn authenticate(
        &self,
        device: &Self::Device,
        _: Option<Duration>,
    ) -> Result<services::AuthenticateResult, Self::AuthenticateError> {
        let stdin = io::stdin();
        println!("Device {} device detected.\n open (y)es, (N)o", device);

        let mut input = String::new();
        while input.is_empty() {
            stdin.lock().read_line(&mut input).unwrap();

            if ["yes", "y"].contains(&input.trim()) {
                info!("allow");
                return Ok(services::AuthenticateResult::Allow);
            } else if ["no", "n", ""].contains(&input.trim()) {
                info!("deny");

                return Ok(services::AuthenticateResult::Deny);
            }
            input = String::new();
        }

        Err(AuthenticatorError::EOLError)
    }
}
