use std::{borrow::BorrowMut, sync::{Arc, Mutex}, thread, usize};

#[cfg(feature = "discord_base")]
mod discord;
#[cfg(feature = "discord_base")]
use discord::{authenticator::DiscordAuth, client};

#[cfg(feature = "bluetooth")]
mod bluetooth;
#[cfg(feature = "bluetooth")]
use bluetooth::{detector::BluetoothDetector};

use doorman::{manager::Manager, registry::Registry};
use doorman::interfaces::services::Registry as RegistryTrait;
use log::{LevelFilter, info, warn};
use simple::{actuator, authenticator, device::SimpleDevice};
use clap::Clap;
use dotenv;

mod simple;

#[cfg(feature = "discord_base")]
type DiscordArgs = discord::cli::Args;
#[derive(Clap, Debug, Clone)]
#[cfg(not(feature = "discord_base"))]
struct DiscordArgs;


#[derive(Clap, Debug, Clone)]
#[clap()]
struct Args {

    #[clap(flatten)]
    discord_args: DiscordArgs,

    /** How much logging to enable (
        -v: warn,
        -vv: info,
        -vvv: debug,
        -vvvv: trace,
        default: error
    )
    */
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
    registry.register_device(SimpleDevice("c0:bd:c8:80:01:9e".to_string()))?;

    cfg_if::cfg_if! {
        if #[cfg(feature="bluetooth")] {
            let detector = BluetoothDetector::new(&registry);
        } else {
            let detector = simple::detector::Detector::new(&registry);
        }
    }


    cfg_if::cfg_if! {
        if #[cfg(feature="discord_base")] {
            let client = client::Client::new(args.discord_args.token, args.discord_args.user).await;
            let client = client.run().await?;
            let auth = DiscordAuth::new(&client);
        }
        else {
            let auth = authenticator::Authenticator {};
        }
    }

    let act = actuator::Actuator;

    let mut manager = Manager::new(detector, auth, act);

    manager.run().await?;

    Ok(())


}
