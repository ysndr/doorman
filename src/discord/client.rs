use core::time;
use std::{borrow::BorrowMut, fmt::Display, sync::{Arc}, thread};

use doorman::interfaces::services::AuthenticateResult;
use serenity::{Client as SerenityClient, client::{Context, EventHandler, bridge::gateway::{ShardId, ShardManager}}, framework::StandardFramework, futures::lock::Mutex, http::CacheHttp, model::prelude::Ready};
use tokio::{sync::mpsc::{self, Receiver}, task::JoinHandle};

use async_trait::async_trait;

struct ReadyHandler(tokio::sync::mpsc::Sender<Context>);


#[async_trait]
impl EventHandler for ReadyHandler {
    async fn ready(&self, ctx: Context, _: Ready) {
        println!("Connected");
        self.0.send(ctx).await;
    }
}


pub struct Client {
    client: Arc<Mutex<serenity::Client>>,
    user: serenity::model::user::User,
    ready: Receiver<Context>,
    ctx: Option<Arc<Context>>
}

impl Client {
    pub async fn run(&mut self) -> JoinHandle<Result<(), serenity::Error>> {

        let client = self.client.clone();
        let handle = tokio::spawn(async move {
            client.lock().await.start().await
        });

        // let client = self.client.clone();
        // let shard_manager = client.clone().lock().await.shard_manager.clone();
        // tokio::spawn(async move {
        //     loop {
        //         let shards = shard_manager.lock().await.shards_instantiated().await;

        //         if ! dbg!(shards).is_empty() {
        //             return;
        //         }
        //     }
        // }).await.unwrap();

        self.ctx = self.ready.recv().await.map(Arc::new);

        println!("ready");

        return handle
    }

    pub async fn new(token: impl AsRef<str>, user_id: u64) -> Self {
        let (tx, mut rx) = mpsc::channel(1);

        let client =
            SerenityClient::builder(token)
                .framework(StandardFramework::new())
                .event_handler(ReadyHandler(tx))
                .await
                .unwrap();

        let user = client
            .cache_and_http
            .http
            .get_user(user_id)
            .await
            .unwrap();

            let client = Arc::new(Mutex::new(client));


        Self { client, user, ready: rx, ctx: None }
    }

    pub async fn authorize<D: Display>(&self, device: D) -> Result<AuthenticateResult, serenity::Error> {



        let ctx = self.ctx.as_ref().unwrap().clone();

        println!("here");


        let message = self
            .user
            .direct_message(&*ctx, |m| {
                m.content(format!("Device close to your door detected: {}\nOpen the door?", device));
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
            let _ = message
                .reply(&*ctx, "Invalidated.")
                .await?;
            return Ok(AuthenticateResult::Deny);
        }
    }
}
