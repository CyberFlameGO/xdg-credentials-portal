use crate::transport::channel::ChannelStatus;
use crate::transport::device::SupportedProtocols;
use crate::transport::Channel;
use crate::webauthn::Error;

use async_trait::async_trait;
use std::fmt::Display;

use super::NfcDevice;

pub struct NfcChannel<'d> {
    device: &'d NfcDevice,
}

impl Display for NfcChannel<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NFC adapter channel")
    }
}

impl<'d> NfcChannel<'d> {
    pub async fn new(device: &'d NfcDevice) -> Result<NfcChannel<'d>, Error> {
        Ok(Self { device })
    }
}

#[async_trait]
impl<'d> Channel for NfcChannel<'d> {
    async fn supported_protocols(&self) -> Result<SupportedProtocols, Error> {
        // No discoverability until time of operation.
        Ok(SupportedProtocols::all())
    }

    async fn status(&self) -> ChannelStatus {
        ChannelStatus::Ready
    }

    async fn close(&self) {
        // No-op
    }

    async fn apdu_send(
        &self,
        request: &crate::proto::ctap1::apdu::ApduRequest,
        timeout: std::time::Duration,
    ) -> Result<(), crate::webauthn::Error> {
        todo!()
    }

    async fn apdu_recv(
        &self,
        timeout: std::time::Duration,
    ) -> Result<crate::proto::ctap1::apdu::ApduResponse, crate::webauthn::Error> {
        todo!()
    }

    async fn cbor_send(
        &self,
        request: &crate::proto::ctap2::cbor::CborRequest,
        timeout: std::time::Duration,
    ) -> Result<(), crate::webauthn::Error> {
        todo!()
    }

    async fn cbor_recv(
        &self,
        timeout: std::time::Duration,
    ) -> Result<crate::proto::ctap2::cbor::CborResponse, crate::webauthn::Error> {
        todo!()
    }
}
