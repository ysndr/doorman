use async_trait::async_trait;
use doorman::interfaces::services::{self, ServiceError};
use futures::{StreamExt, future};
use serenity::{Error as SerenityError, model::channel::Message};
use std::{fmt::Display, marker::PhantomData, time::Duration};
use thiserror::Error;

use super::client::{Client, Initialized};

#[derive(Debug, Error)]
pub enum LockerError {
    #[error("Client Error: {0}")]
    Client(#[from] SerenityError),

    #[error("Failed to lock")]
    LockFailure
}

impl ServiceError for LockerError {}

pub struct DiscordLocker<'a> {
    client: &'a Client<Initialized>,
}

impl<'a> DiscordLocker<'a> {
    pub fn new(client: &'a Client<Initialized>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl services::Locker for DiscordLocker<'_> {

    type LockerError = LockerError;

    async fn wait_for_lock(&self) -> Result<(), Self::LockerError> {
        let ctx = self.client.state.ctx.clone();
        let message = self
            .client
            .user
            .direct_message(&*ctx, |m| {
                m.content("React to close door...".to_string());
                m.reactions(['🔒'].iter().cloned())
            })
            .await?;

        message.await_reaction(&*ctx).filter(
            |reaction| {
                let react = &reaction.as_ref().emoji;
                return matches!(react.as_data().as_str(), "🔒");
            }
        ).await.ok_or(LockerError::LockFailure )?;

        Ok(())
    }

    async fn confirm_lock(&self) -> Result<(), Self::LockerError> {
        let ctx = self.client.state.ctx.clone();

        self
            .client
            .user
            .direct_message(&*ctx, |m| {
                m.content("Locked!".to_string())
            })
            .await?;

        Ok(())
    }


}
