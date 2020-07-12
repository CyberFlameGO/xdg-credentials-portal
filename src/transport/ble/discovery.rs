extern crate blurz;
extern crate mockall;

use crate::transport::ble::device::{ConnectedDevice, KnownDevice, FIDO_PROFILE_UUID};
use crate::transport::ble::BleDevicePath;
use crate::transport::error::TransportError;

use blurz::BluetoothAdapter;
use blurz::BluetoothSession;

use self::blurz::BluetoothDevice;
use crate::transport::ble::gatt::get_devices_by_uuid;
use mockall::automock;

#[derive(Debug)]
pub struct DiscoverySession {
    session: BluetoothSession,
}

#[automock]
impl DiscoverySession {
    pub fn new() -> Self {
        Self {
            session: BluetoothSession::create_session(None).unwrap(),
        }
    }

    pub fn is_discovering(&self) -> Result<bool, TransportError> {
        let adapter =
            BluetoothAdapter::init(&self.session).or(Err(TransportError::TransportUnavailable))?;
        adapter
            .is_discovering()
            .or(Err(TransportError::TransportUnavailable))
    }

    pub fn start_discovery(&self) -> Result<(), TransportError> {
        let adapter =
            BluetoothAdapter::init(&self.session).or(Err(TransportError::TransportUnavailable))?;
        if adapter
            .is_discovering()
            .or(Err(TransportError::TransportUnavailable))?
        {
            return Err(TransportError::AlreadyInProgress);
        }
        adapter
            .start_discovery()
            .or(Err(TransportError::TransportUnavailable))?;
        Ok(())
    }

    pub fn stop_discovery(&self) -> Result<(), TransportError> {
        let adapter =
            BluetoothAdapter::init(&self.session).or(Err(TransportError::TransportUnavailable))?;
        adapter
            .stop_discovery()
            .or(Err(TransportError::TransportUnavailable))?;
        Ok(())
    }

    pub fn devices(&self) -> Result<Vec<KnownDevice>, TransportError> {
        let fido_devices = get_devices_by_uuid(&self.session, &FIDO_PROFILE_UUID)?;
        let known_devices = fido_devices
            .iter()
            .map(|device| KnownDevice::new(&self.session, &device.get_id()))
            .collect();
        Ok(known_devices)
    }
}
