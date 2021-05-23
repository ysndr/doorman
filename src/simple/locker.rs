use std::io::{self, BufRead, stdin};

use async_trait::async_trait;

use doorman::interfaces::services::{self, ServiceError};
use thiserror::Error;


#[derive(Debug, Error)]
pub enum LockerError {
    #[error("Input reached EOF ({0})")]
    EOF(#[from] io::Error)
}

impl ServiceError for LockerError {}

pub struct Locker;

impl Locker {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl services::Locker for Locker {

    type LockerError = LockerError;

    async fn wait_for_lock(&self) -> Result<(), Self::LockerError> {
        println!("Press [Enter] to lock...");
        let mut buf = String::new();
        stdin().lock().read_line(&mut buf)?;
        Ok(())
    }

    async fn confirm_lock(&self) -> Result<(), Self::LockerError> {
        println!("locked!");
        Ok(())
    }




}
