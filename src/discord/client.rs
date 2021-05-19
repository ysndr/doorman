use core::time;
use std::{borrow::BorrowMut, fmt::Display, marker::PhantomData, sync::Arc, thread};

use async_trait::async_trait;
use doorman::interfaces::services::AuthenticateResult;
use log::{debug, error, info};
use serenity::{
    client::{
        bridge::gateway::{ShardId, ShardManager},
        Context, EventHandler,
    },
    framework::StandardFramework,
    futures::lock::Mutex,
    http::CacheHttp,
    model::prelude::Ready,
    Client as SerenityClient,
};
use thiserror::Error;
use tokio::{
    sync::mpsc::{self, Receiver},
    task::JoinHandle,
};

struct ReadyHandler(tokio::sync::mpsc::Sender<Context>);

#[async_trait]
impl EventHandler for ReadyHandler {
    async fn ready(&self, ctx: Context, _: Ready) {
        info!("Connected");
        self.0.send(ctx).await;
    }
}
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Could not run client, failed to retrieve Context")]
    ContextMissing,
}

pub trait ClientState {}
pub struct Uninitialized {
    ready: Receiver<Context>,
}
impl ClientState for Uninitialized {}

pub struct Initialized {
    ctx: Arc<Context>,
    handle: JoinHandle<Result<(), serenity::Error>>,
}
impl ClientState for Initialized {}

pub struct Client<S: ClientState> {
    client: Arc<Mutex<serenity::Client>>,
    user: serenity::model::user::User,
    state: S,
}

impl Client<Uninitialized> {
    pub async fn new(token: impl AsRef<str>, user_id: u64) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let client = SerenityClient::builder(token)
            .framework(StandardFramework::new())
            .event_handler(ReadyHandler(tx))
            .await
            .unwrap();

        let user = client.cache_and_http.http.get_user(user_id).await.unwrap();

        let client = Arc::new(Mutex::new(client));

        Self {
            client,
            user,
            state: Uninitialized { ready: rx },
        }
    }

    pub async fn run(mut self) -> Result<Client<Initialized>, ClientError> {
        let client = self.client.clone();
        let handle = tokio::spawn(async move { client.lock().await.start().await });

        if let Some(ctx) = self.state.ready.recv().await.map(Arc::new) {
            let initialized: Client<Initialized> = Client {
                client: self.client,
                user: self.user,
                state: Initialized { ctx, handle },
            };

            debug!("Context Received");

            return Ok(initialized);
        } else {
            handle.abort();
            error!("Could not retrieve context");
            return Err(ClientError::ContextMissing);
        }
    }
}

impl Client<Initialized> {
    pub async fn authorize<D: Display>(
        &self,
        device: D,
    ) -> Result<AuthenticateResult, serenity::Error> {
        let ctx = self.state.ctx.clone();
        let message = self
            .user
            .direct_message(&*ctx, |m| {
                m.content(format!(
                    "Device close to your door detected: {}\nOpen the door?",
                    device
                ));
                m.reactions(['👍', '🚷'].iter().cloned())
            })
            .await
            .unwrap();
        // // TODO: Clean this up if possible?
        // let shards = client.shard_manager.lock().await;
        // let runners = shards.runners.lock().await;

        // dbg!(&shards.shards_instantiated().await);
        // let dm_shard = runners.get(&ShardId(0)).unwrap();

        if let Some(reaction) = message.await_reaction(&*ctx).await {
            let react = &reaction.as_inner_ref().emoji;

            return match react.as_data().as_str() {
                "👍" => Ok(AuthenticateResult::Allow),
                _ => Ok(AuthenticateResult::Deny),
            };
        } else {
            let _ = message.reply(&*ctx, "Invalidated.").await?;
            return Ok(AuthenticateResult::Deny);
        }
    }
}
