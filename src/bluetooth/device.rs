use derive_more::{Constructor, Display};
use serde::{Deserialize, Serialize};

pub type Address = String;

#[derive(Debug, Clone, Constructor, Display, Serialize, Deserialize)]
#[display(fmt = "{}/{} ({})", name, address, rssi_reference)]
pub struct BluetoothDevice {
    name: String,
    address: Address,
    rssi_reference: u64,
}

impl Into<Address> for BluetoothDevice {
    fn into(self) -> Address {
        self.address
    }
}
