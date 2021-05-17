use doorman::{manager::Manager, registry::Registry};
use doorman::interfaces::services::Registry as RegistryTrait;
use simple::{actuator, authenticator, device::SimpleDevice};

mod simple;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

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
