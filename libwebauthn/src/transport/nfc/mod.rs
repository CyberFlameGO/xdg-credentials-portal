use std::fmt::Display;

mod channel;
mod device;

pub use channel::NfcChannel;
pub use device::{list_devices, NfcDevice};

use super::Transport;

pub struct Nfc {}
impl Transport for Nfc {}
unsafe impl Send for Nfc {}
unsafe impl Sync for Nfc {}

impl Display for Nfc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Nfc")
    }
}
