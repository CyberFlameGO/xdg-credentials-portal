use super::{Nfc, NfcChannel};

use crate::transport::device::SupportedProtocols;
use crate::transport::error::{Error, TransportError};
use crate::transport::{Channel, Device};

use async_trait::async_trait;
use std::fmt;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::sync::Mutex;
use tracing::{debug, error, instrument, warn};

pub struct NfcDevice {
    name: String,
    connstring: String,
}

unsafe impl Send for NfcDevice {}
unsafe impl Sync for NfcDevice {}

impl NfcDevice {
    pub fn new(name: &str, connstring: &str) -> Pin<Box<Self>> {
        let d = NfcDevice {
            name: String::from(name),
            connstring: String::from(connstring),
        };

        let mut boxed = Box::pin(d);
        boxed
    }
}

impl fmt::Debug for NfcDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.connstring)
    }
}

impl fmt::Display for NfcDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NFC device: {}", self.name)
    }
}

#[async_trait]
impl<'d> Device<'d, Nfc, NfcChannel<'d>> for NfcDevice {
    async fn channel(&'d mut self) -> Result<NfcChannel, Error> {
        let channel = NfcChannel::new(self).await?;
        Ok(channel)
    }

    async fn supported_protocols(&mut self) -> Result<SupportedProtocols, Error> {
        todo!()
    }
}

pub async fn list_devices() -> Result<Vec<Pin<Box<NfcDevice>>>, Error> {
    unimplemented!()
}
