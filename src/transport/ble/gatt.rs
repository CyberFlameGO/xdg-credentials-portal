use blurz::bluetooth_gatt_characteristic::BluetoothGATTCharacteristic;
use blurz::bluetooth_gatt_service::BluetoothGATTService;
use blurz::bluetooth_session::BluetoothSession;

use crate::transport::error::TransportError;
use blurz::{BluetoothAdapter, BluetoothDevice};

pub fn get_gatt_characteristic<'a>(
    session: &'a BluetoothSession,
    service: &BluetoothGATTService<'a>,
    uuid: &str,
) -> Result<BluetoothGATTCharacteristic<'a>, TransportError> {
    service
        .get_gatt_characteristics()
        .or(Err(TransportError::ConnectionLost))?
        .iter()
        .map(|char_path| BluetoothGATTCharacteristic::new(session, char_path.to_owned()))
        .map(|char| (char, char.get_uuid().unwrap()))
        .find(|(_, char_uuid)| char_uuid == uuid)
        .map(|(char, _)| char)
        .ok_or(TransportError::InvalidEndpoint)
}

pub fn get_gatt_service<'a>(
    session: &'a BluetoothSession,
    device: &BluetoothDevice<'a>,
    uuid: &str,
) -> Result<BluetoothGATTService<'a>, TransportError> {
    device
        .get_gatt_services()
        .or(Err(TransportError::ConnectionLost))?
        .iter()
        .map(|service_path| BluetoothGATTService::new(session, service_path.to_owned()))
        .map(|service| (service, service.get_uuid().unwrap()))
        .find(|(_, service_uuid)| service_uuid == uuid)
        .map(|(service, _)| service)
        .ok_or(TransportError::InvalidEndpoint)
}

pub fn get_devices_by_uuid<'a>(
    session: &'a BluetoothSession,
    uuid: &str,
) -> Result<Vec<BluetoothDevice<'a>>, TransportError> {
    let adapter = BluetoothAdapter::init(session).or(Err(TransportError::TransportUnavailable))?;
    let devices = adapter
        .get_device_list()
        .or(Err(TransportError::TransportUnavailable))?
        .iter()
        .map(|device_id| BluetoothDevice::new(session, device_id.to_string()))
        .map(|device| (device, device.get_uuids().unwrap_or(vec![])))
        .filter(|(_, device_uuids)| device_uuids.contains(&uuid.to_owned()))
        .map(|(device, _)| device)
        .collect();
    Ok(devices)
}
