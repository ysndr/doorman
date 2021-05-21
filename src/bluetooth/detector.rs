use std::sync::Arc;

use async_trait::async_trait;
use futures::lock::Mutex;
use log::{debug, info, warn};
use tokio::time::{sleep, Duration};

use bluez::client::*;
use bluez::interface::controller::*;
use bluez::interface::event::Event;
use bluez::Error as BluezError;

use doorman::interfaces::services::{self, Registry, ServiceError};
use thiserror::Error;

use super::device::BluetoothDevice;

#[derive(Debug, Error)]
pub enum DetectorError {
    #[error("Error in bluez: {0}")]
    Bluez(#[from] BluezError),

    #[error("Couldn't find supported Bluetooth device")]
    NoDevice,
}

impl ServiceError for DetectorError {}

pub struct BluetoothDetector<'a, Reg: Registry + Send + Sync> {
    registry: &'a Reg,
    client: Mutex<BlueZClient<'a>>,
    controller: Controller,
}

impl<'a, Reg: Registry + Send + Sync> BluetoothDetector<'a, Reg> {
    pub async fn new(registry: &'a Reg) -> Result<BluetoothDetector<'a, Reg>, DetectorError> {
        let mut client = BlueZClient::new().unwrap();
        let controllers = client.get_controller_list().await?;

        // find the first controller we can power on
        let mut result = None;
        for controller in controllers.into_iter() {
            let info = client.get_controller_info(controller).await?;
            if info.supported_settings.contains(ControllerSetting::Powered) {
                result = Some((controller, info));
                break;
            }
        }
        let (controller, info) = result.ok_or(DetectorError::NoDevice)?;

        info!("Found controller {}", controller);

        if !info.current_settings.contains(ControllerSetting::Powered) {
            warn!("Bluetooth controller {} powered off", controller);
            info!("Powering on bluetooth controller {}", controller);
            client.set_powered(controller, true).await?;
        }

        Ok(Self {
            registry,
            client: Mutex::new(client),
            controller,
        })
    }
}

#[async_trait]
impl<'a, Reg: Registry<Ident = String, Device = BluetoothDevice> + Send + Sync> services::Detector
    for BluetoothDetector<'a, Reg>
{
    type Device = BluetoothDevice;
    type DetectorError = DetectorError;

    async fn wait_for_device(&self) -> Result<&Self::Device, DetectorError> {
        let mut client = self.client.lock().await;

        // scan for some devices
        // to do this we'll need to listen for the Device Found event
        client
            .start_discovery(
                self.controller,
                AddressTypeFlag::BREDR | AddressTypeFlag::LEPublic | AddressTypeFlag::LERandom,
            )
            .await?;

        // just wait for discovery forever
        loop {
            // process() blocks until there is a response to be had
            let response = client.process().await?;
            debug!("Processing bluetooth event {:?}", response.event);
            match response.event {
                Event::DeviceFound {
                    address,
                    address_type: _,
                    flags: _,
                    rssi,
                    ..
                } => {
                    if let Some(device) = self.registry.check(&address.to_string()) {
                        info!("Registered device {} found with RSSI {}", device, rssi);
                        return Ok(&device);
                    };
                }
                Event::Discovering {
                    discovering,
                    address_type: _,
                } => {
                    // println!("discovering: {} {:?}", discovering, address_type);
                    // if discovery ended, turn it back on
                    if !discovering {
                        client
                            .start_discovery(
                                self.controller,
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
