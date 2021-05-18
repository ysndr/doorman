use doorman::interfaces::services::{self, Authenticate, Registry, ServiceError};
use std::{fmt::Display, io::{self, BufRead}, marker::PhantomData};
use serenity::Error as SerenityError;
use thiserror::Error;
use async_trait::async_trait;

use crate::simple::{device::SimpleDevice};

use super::client::Client;


#[derive(Debug, Error)]
pub enum AuthorizationError {
    #[error("Client Error: {0}")]
    Client(#[from] SerenityError)
}

impl ServiceError for AuthorizationError {}

pub struct DiscordAuth<'a, D> {
    client: &'a Client,
    device: PhantomData<D>,
}

impl<'a, D> DiscordAuth<'a, D> {
    pub fn new(client: &'a Client) -> Self {

        let device = PhantomData;

        Self { client, device }
    }
}

#[async_trait]
impl <D: Send + Sync + Display> services::Authenticate for DiscordAuth<'_, D> {
    type Device = D;
    type AuthenticateError = AuthorizationError;

    async fn authenticate(&self, device: &Self::Device) -> Result<services::AuthenticateResult, Self::AuthenticateError> {
        self.client.authorize(device).await.map_err(AuthorizationError::Client)
    }




}
