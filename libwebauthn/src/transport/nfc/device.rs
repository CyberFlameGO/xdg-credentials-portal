use super::{Nfc, NfcChannel};

use crate::transport::device::SupportedProtocols;
use crate::transport::error::{Error, TransportError};
use crate::transport::{Channel, Device};

use async_trait::async_trait;
use nfc1::{self, Context as Nfc1Context, Device as Nfc1Device};
use std::fmt;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::sync::Mutex;
use tracing::{debug, error, instrument, warn};

pub struct NfcDevice<'c> {
    name: String,
    connstring: String,

    device: Nfc1Device<'c>,
    context: Nfc1Context<'c>,
    _marker: PhantomPinned,
}

unsafe impl Send for NfcDevice<'_> {}
unsafe impl Sync for NfcDevice<'_> {}

impl<'c> NfcDevice<'c> {
    pub fn new(
        name: &str,
        connstring: &str,
        device: Nfc1Device<'c>,
        context: Nfc1Context<'c>,
    ) -> Pin<Box<Self>> {
        let d = NfcDevice {
            name: String::from(name),
            connstring: String::from(connstring),
            device,
            context,
            _marker: PhantomPinned,
        };

        let mut boxed = Box::pin(d);
        boxed
    }

    #[instrument]
    fn open(&self) -> Result<Nfc1Device, Error> {
        let mut context = match Nfc1Context::new() {
            Ok(context) => context,
            Err(err) => {
                error!(%err, "Failed to create NFC reader device context");
                return Err(Error::Transport(TransportError::TransportUnavailable));
            }
        };

        let mut device = match context.open_with_connstring(&self.connstring) {
            Ok(device) => device,
            Err(err) => {
                error!(%err, "Failed to open NFC reader device context");
                return Err(Error::Transport(TransportError::TransportUnavailable));
            }
        };

        Ok(device)
    }

    #[instrument(skip(device))]
    fn set_property(
        &self,
        device: &mut Nfc1Device,
        prop: nfc1::Property,
        value: bool,
    ) -> Result<(), Error> {
        match device.set_property_bool(nfc1::Property::HandleCrc, false) {
            Ok(_) => Ok(()),
            Err(err) => {
                error!(%err, "Failed to set NFC reader property");
                Err(Error::Transport(TransportError::TransportUnavailable))
            }
        }
    }

    pub async fn wait_for_device(&self) -> Result<(), Error> {
        let mut device = self.open()?;

        // Configure the CRC
        self.set_property(&mut device, nfc1::Property::HandleCrc, false)?;
        // Use raw send/receive methods
        self.set_property(&mut device, nfc1::Property::EasyFraming, false)?;
        // Disable 14443-4 autoswitching
        self.set_property(&mut device, nfc1::Property::AutoIso144434, false)?;

        Ok(())
    }
}

impl fmt::Debug for NfcDevice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.connstring)
    }
}

impl fmt::Display for NfcDevice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NFC device: {}", self.name)
    }
}

#[async_trait]
impl<'d> Device<'d, Nfc, NfcChannel<'d>> for NfcDevice<'d> {
    async fn channel(&'d mut self) -> Result<NfcChannel, Error> {
        let channel = NfcChannel::new(self).await?;
        Ok(channel)
    }

    async fn supported_protocols(&mut self) -> Result<SupportedProtocols, Error> {
        todo!()
    }
}

pub async fn list_devices<'d>() -> Result<Vec<Pin<Box<NfcDevice<'d>>>>, Error> {
    let mut context = match Nfc1Context::new() {
        Ok(context) => context,
        Err(err) => {
            error!(%err, "Failed to create nfc1 context");
            return Err(Error::Transport(TransportError::TransportUnavailable));
        }
    };

    let connstrings = match context.list_devices(255) {
        Ok(connstrings) => connstrings,
        Err(err) => {
            error!(%err, "Failed to open nfc1 context to list devices");
            return Err(Error::Transport(TransportError::TransportUnavailable));
        }
    };

    let mut devices = vec![];
    for connstring in connstrings {
        let mut new_context = match Nfc1Context::new() {
            Ok(context) => context,
            Err(err) => {
                error!(%err, "Failed to create nfc1 context");
                return Err(Error::Transport(TransportError::TransportUnavailable));
            }
        };
        let device = NfcDevice::new("noname", &connstring, nfc1_device, new_context);
        debug!(?device, "Discovered NFC device");
        devices.push(device);
    }

    Ok(devices)
}
