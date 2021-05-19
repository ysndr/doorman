use std::{borrow::BorrowMut, sync::{Arc, Mutex}, thread, usize};

use discord::{authenticator::DiscordAuth, client};
use doorman::{manager::Manager, registry::Registry};
use doorman::interfaces::services::Registry as RegistryTrait;
use log::{LevelFilter, info, warn};
use simple::{actuator, authenticator, device::SimpleDevice};
use clap::Clap;
use dotenv;

mod simple;
mod discord;

#[derive(Clap, Debug, Clone)]
#[clap()]
struct Args {

    /// Discord UserID
    #[clap(short, long, env="DISCORD_USER_ID")]
    user: u64,

    /// Discord Bot Token
    #[clap(short, long, env="DISCORD_TOKEN")]
    token: String,

    /// How much logging to enable
    /// -v:    warn
    /// -vv:   info
    /// -vvv:  debug
    /// -vvvv: trace
    /// default: error
    #[clap(short, parse(from_occurrences))]
    verbosity: u8,

}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let args = Args::parse();


    env_logger::Builder::from_default_env()
    .filter_level(match args.verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    })
    // .filter_module("doorman", log::LevelFilter::Debug)
    .init();

    let mut registry = Registry::new();
    registry.register_device(SimpleDevice("OnePlus 5".to_string()))?;
    registry.register_device(SimpleDevice("Yannik's MacBook Pro".to_string()))?;


    let detector = simple::detector::Detector::new(&registry);
    // let auth = authenticator::Authenticator {};




    let client = client::Client::new(args.token, args.user).await;
    let client = client.run().await?;

    let auth = DiscordAuth::new(&client);
    let act = actuator::Actuator;

    let mut manager = Manager::new(detector, auth, act);

    manager.run().await?;

    Ok(())


}
