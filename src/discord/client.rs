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
    pub(crate) ctx: Arc<Context>,
    handle: JoinHandle<Result<(), serenity::Error>>,
}
impl ClientState for Initialized {}

pub struct Client<S: ClientState> {
    client: Arc<Mutex<serenity::Client>>,
    pub(crate) user: serenity::model::user::User,
    pub(crate) state: S,
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
}
