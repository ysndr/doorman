use async_trait::async_trait;
use doorman::interfaces::services::{
    self, Authenticate, AuthenticateResult, Registry, ServiceError,
};
use serenity::Error as SerenityError;
use core::time;
use std::{fmt::Display, io::{self, BufRead}, marker::PhantomData, time::Duration};
use thiserror::Error;

use crate::simple::device::SimpleDevice;

use super::client::{Client, ClientState, Initialized};

#[derive(Debug, Error)]
pub enum AuthorizationError {
    #[error("Client Error: {0}")]
    Client(#[from] SerenityError),
}

impl ServiceError for AuthorizationError {}

pub struct DiscordAuth<'a, D> {
    client: &'a Client<Initialized>,
    device: PhantomData<D>,
}

impl<'a, D> DiscordAuth<'a, D> {
    pub fn new(client: &'a Client<Initialized>) -> Self {
        let device = PhantomData;

        Self { client, device }
    }
}

#[async_trait]
impl<D: Send + Sync + Display> services::Authenticate for DiscordAuth<'_, D> {
    type Device = D;
    type AuthenticateError = AuthorizationError;

    async fn authenticate(
        &self,
        device: &Self::Device,
        timeout: Option<usize>
    ) -> Result<services::AuthenticateResult, Self::AuthenticateError> {
        let ctx = self.client.state.ctx.clone();
        let message = self
            .client
            .user
            .direct_message(&*ctx, |m| {
                m.content(format!(
                    "Device close to your door detected: {}\nOpen the door?",
                    device
                ));
                m.reactions(['👍', '🚷'].iter().cloned())
            })
            .await?;

        let mut collect_reaction = message.await_reaction(&*ctx);
        if let Some(timeout) = timeout {
            collect_reaction = collect_reaction.timeout(Duration::from_secs(timeout as u64));
        }

        if let Some(reaction) = collect_reaction.await {
            let react = &reaction.as_inner_ref().emoji;

            return match react.as_data().as_str() {
                "👍" => Ok(AuthenticateResult::Allow),
                _ => Ok(AuthenticateResult::Deny),
            };
        } else {
            let _ = message.reply(&*ctx, "Invalidated.").await?;

            return Ok(AuthenticateResult::Deny);
        }
        // map_err(AuthorizationError::Client)
    }
}
