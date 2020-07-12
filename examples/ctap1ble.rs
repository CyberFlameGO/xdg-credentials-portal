extern crate backend;
extern crate base64_url;
extern crate log;
extern crate tokio;

use blurz::bluetooth_adapter::BluetoothAdapter as Adapter;
use blurz::bluetooth_device::BluetoothDevice as Device;
use blurz::bluetooth_session::BluetoothSession as Session;

use backend::ops::u2f::{RegisterRequest, SignRequest};
use backend::transport::ble::BleDevicePath;
use backend::Platform;
use sha2::{Digest, Sha256};

fn build_client_data(challenge: &Vec<u8>, app_id: &str) -> (String, Vec<u8>) {
    let challenge_base64url = base64_url::encode(&challenge);
    let version_string = "U2F_V2";

    let client_data = format!(
        "{{\"challenge\": \"{}\", \"version:\": \"{}\", \"appId\": \"{}\"}}",
        challenge_base64url, version_string, app_id
    );

    let mut hasher = Sha256::default();
    hasher.input(client_data.as_bytes());
    let client_data_hash = hasher.result().to_vec();

    (client_data, client_data_hash)
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    const APP_ID: &str = "https://foo.example.org";
    const TIMEOUT: u32 = 20; // Seconds
    let challenge = base64_url::decode("1vQ9mxionq0ngCnjD-wTsv1zUSrGRtFqG2xP09SbZ70").unwrap();
    let (_, client_data_hash) = build_client_data(&challenge, APP_ID);

    let platform = Platform::new();
    let ble_manager = platform.get_ble_manager().unwrap();

    // Selecting a device
    let bt_session = &Session::create_session(None)?;

    if let None = bt_device {
        panic!(
            "BLE pairing and discovery is outside of the scope of this example. Ensure your \
                BLE authenticator is paired, and try again."
        )
    }
    let bt_device = bt_device.unwrap();
    println!(
        "Selected BLE authenticator {} ({})",
        bt_device.get_alias()?,
        bt_device.get_address()?
    );

    let device: BleDevicePath = bt_device.get_id();
    let device = ble_manager.connect(&device).unwrap();

    // Registration ceremony
    println!("Registration request sent (timeout: {} seconds).", TIMEOUT);
    let register_request = RegisterRequest::new_u2f_v2(&APP_ID, &client_data_hash, vec![], TIMEOUT);
    let response = ble_manager.u2f_register(&device, register_request).await?;
    println!("Response: {:?}", response);

    // Signature ceremony
    println!("Signature request sent (timeout: {} seconds).", TIMEOUT);
    let new_key = response.as_registered_key()?;
    let sign_request = SignRequest::new(
        &APP_ID,
        &client_data_hash,
        &new_key.key_handle,
        TIMEOUT,
        true,
    );
    let response = ble_manager.u2f_sign(&device, sign_request).await?;
    println!("Response: {:?}", response);

    Ok(())
}
