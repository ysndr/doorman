use doorman::{manager::Manager, registry::Registry};
use doorman::interfaces::services::Registry as RegistryTrait;
use simple::{actuator, authenticator, device::SimpleDevice};
use clap::Clap;

mod simple;

#[derive(Clap, Debug, Clone)]
#[clap()]
struct Args {


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
    .init();

    let mut registry = Registry::new();
    registry.register_device(SimpleDevice("OnePlus 5".to_string()))?;
    registry.register_device(SimpleDevice("Yannik's MacBook Pro".to_string()))?;


    let detector = simple::detector::Detector::new(&registry);
    let auth = authenticator::Authenticator {};
    let act = actuator::Actuator;

    let mut manager = Manager::new(detector, auth, act);

    manager.run().await?;

    Ok(())


}
