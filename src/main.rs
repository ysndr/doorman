#[cfg(feature = "discord_base")]
mod discord;
use std::{path::PathBuf, time::Duration};

#[cfg(feature = "discord_base")]
use discord::{authenticator::DiscordAuth, client, locker::DiscordLocker};

#[cfg(feature = "bluetooth")]
mod bluetooth;
#[cfg(feature = "bluetooth")]
use bluetooth::{detector::BluetoothDetector, device::BluetoothDevice};

use clap::Clap;
use doorman::{interfaces::services::Registry as RegistryTrait, manager};
use doorman::{manager::Manager, registry::Registry};
use log::{debug, LevelFilter};
use simple::{actuator, authenticator, device::SimpleDevice};

use crate::{
    simple::{detector::Detector, locker::Locker},
};

mod rpi_servo;

mod simple;

#[cfg(feature = "discord_base")]
type DiscordArgs = discord::cli::Args;
#[derive(Clap, Debug, Clone)]
#[cfg(not(feature = "discord_base"))]
struct DiscordArgs;

#[cfg(feature = "bluetooth")]
type BluetoothArgs = bluetooth::cli::Args;
#[derive(Clap, Debug, Clone)]
#[cfg(not(feature = "bluetooth"))]
struct BluetoothArgs;

#[derive(Clap, Debug, Clone)]
struct ManagerConfig {
    /// Authorization timeout. How long until an authorization has to be issued (in sec)
    #[clap(short, long, env = "AUTH_TIMEOUT")]
    timeout: Option<u64>,

    /// Time between authorization attempts (in sec)
    #[clap(short, long, env = "COOLDOWN_TIMEOUT", default_value="30")]
    cooldown: u64,
}

#[derive(Clap, Debug, Clone)]
#[clap()]
struct Args {
    #[clap(flatten)]
    discord_args: DiscordArgs,

    #[clap(flatten)]
    bluetooth_args: BluetoothArgs,

    #[clap(flatten)]
    manager_config: ManagerConfig,

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

    /// Devices that are allowed authorize
    #[clap(short, long)]
    devices: PathBuf,
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
    registry.from_file(args.devices)?;

    debug!("Registered Devices: {:?}", registry.list());

    cfg_if::cfg_if! {
        if #[cfg(feature="bluetooth")] {
            let detector = BluetoothDetector::new(&registry).await?;
        } else {
            let detector= simple::detector::Detector::new(&registry);
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature="discord_base")] {
            let client = client::Client::new(args.discord_args.token, args.discord_args.user).await;
            let client = client.run().await?;
            let auth = DiscordAuth::new(&client);
            let locker = DiscordLocker::new(&client);
        }
        else {
            let auth = authenticator::Authenticator::new();
            let locker = Locker::new();
        }
    }

    let mut act = rpi_servo::actuator::Actuator::new()?;

    let config = manager::Config {
        authorize_timeout: args.manager_config.timeout.map(Duration::from_secs),
        reauthorize_timeout: Duration::from_secs(args.manager_config.cooldown)
    };

    let mut manager = Manager::new(&detector, &auth, &mut act, &locker, config);

    manager.daemon().await?;

    Ok(())
}
