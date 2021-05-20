use async_trait::async_trait;
use log::{debug, info, warn};
use tokio::time::{sleep, Duration};

use bluez::client::*;
use bluez::interface::controller::*;
use bluez::interface::event::Event;
use bluez::Error as BluezError;

use crate::simple::device::SimpleDevice;
use doorman::interfaces::services::{self, Registry, ServiceError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DetectorError {
    #[error("Error in bluez: {0}")]
    Bluez(#[from] BluezError),

    #[error("Couldn't find supported Bluetooth device")]
    NoDevice,
}

impl ServiceError for DetectorError {}

pub struct BluetoothDetector<'a, Reg: Registry<Device = SimpleDevice> + Send + Sync> {
    registry: &'a Reg,
}

impl<'a, Reg: Registry<Device = SimpleDevice> + Send + Sync> BluetoothDetector<'a, Reg> {
    pub fn new(registry: &'a Reg) -> Self { Self { registry } }
}

#[async_trait]
impl<'a, Reg: Registry<Device = SimpleDevice> + Send + Sync> services::Detector for BluetoothDetector<'a, Reg> {
    type Device = SimpleDevice;
    type DetectorError = DetectorError;

    async fn wait_for_device(&self) -> Result<Self::Device, DetectorError> {
        let mut client = BlueZClient::new().unwrap();
        let controllers = client.get_controller_list().await?;

        // find the first controller we can power on
        let (controller, info) = controllers
            .into_iter()
            .filter_map(|controller| {
                let info = block_on(client.get_controller_info(controller)).ok()?;

                if info.supported_settings.contains(ControllerSetting::Powered) {
                    Some((controller, info))
                } else {
                    None
                }
            })
            .nth(0)
            .ok_or(DetectorError::NoDevice)?;

        info!("Found controller {}", controller);

        if !info.current_settings.contains(ControllerSetting::Powered) {
            warn!("Bluetooth controller {} powered off", controller);
            info!("Powering on bluetooth controller {}", controller);
            client.set_powered(controller, true).await?;
        }

        // scan for some devices
        // to do this we'll need to listen for the Device Found event
        client
            .start_discovery(
                controller,
                AddressTypeFlag::BREDR | AddressTypeFlag::LEPublic | AddressTypeFlag::LERandom,
            )
            .await?;

        // just wait for discovery forever
        loop {
            // process() blocks until there is a response to be had
            let response = client.process().await?;
            debug!("Processing bluetooth event {}", response);

            match response.event {
                Event::DeviceFound {
                    address,
                    address_type: _,
                    flags: _,
                    rssi,
                    ..
                } => {
                    let device = SimpleDevice(address.to_string());
                    if self.registry.registered(&device) {
                        info!("Registered device {} found with RSSI {}", device, rssi);

                        return Ok(device);
                    };
                }
                Event::Discovering {
                    discovering,
                    address_type,
                } => {
                    // println!("discovering: {} {:?}", discovering, address_type);
                    // if discovery ended, turn it back on
                    if !discovering {
                        client
                            .start_discovery(
                                controller,
                                AddressTypeFlag::BREDR
                                    | AddressTypeFlag::LEPublic
                                    | AddressTypeFlag::LERandom,
                            )
                            .await?;
                    }
                }
                _ => (),
            }

            sleep(Duration::from_millis(50)).await;
        }
    }
}
