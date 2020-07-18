use crate::proto::ctap1::{Ctap1RegisterRequest, Ctap1RegisteredKey, Ctap1SignRequest};
use crate::proto::ctap1::{Ctap1RegisterResponse, Ctap1SignResponse};
use crate::proto::ctap2::{
    Ctap2COSEAlgorithmIdentifier, Ctap2GetAssertionRequest, Ctap2MakeCredentialRequest,
};
use crate::proto::ctap2::{Ctap2GetAssertionResponse, Ctap2MakeCredentialResponse};

use crate::ops::u2f::{RegisterRequest, SignRequest};

use super::downgrade::Downgrade;

use log::debug;
use std::convert::{TryFrom, TryInto};

// FIDO2 operations can be mapped by default to their respective CTAP2 requests.

pub type MakeCredentialRequest = Ctap2MakeCredentialRequest;
pub type MakeCredentialResponse = Ctap2MakeCredentialResponse;
pub type GetAssertionRequest = Ctap2GetAssertionRequest;
pub type GetAssertionResponse = Ctap2GetAssertionResponse;

impl Downgrade<RegisterRequest> for MakeCredentialRequest {
    fn is_downgradable(&self) -> bool {
        self.algorithms
            .iter()
            .find(|&t| t.algorithm == Ctap2COSEAlgorithmIdentifier::ES256)
            .is_some()
            && !self.require_resident_key
            && !self.require_user_verification
    }

    fn downgrade(&self) -> Option<RegisterRequest> {
        if !self.is_downgradable() {
            return None;
        }
        let app_id = &self.relying_party.id;
        let challenge = &self.hash;
        let registered_keys: Vec<Ctap1RegisteredKey> = self
            .exclude
            .iter()
            .flat_map(|v| v)
            .map(|credential| Ctap1RegisteredKey::new_u2f_v2(&credential.id))
            .collect();
        let downgraded = RegisterRequest::new_u2f_v2(app_id, challenge, registered_keys);
        debug!("Downgraded request: {:?}", downgraded);
        Some(downgraded)
    }
}

impl Downgrade<SignRequest> for GetAssertionRequest {
    fn is_downgradable(&self) -> bool {
        false
    }

    fn downgrade(&self) -> Option<SignRequest> {
        if !self.is_downgradable() {
            return None;
        }
        unimplemented!()
    }
}

impl From<Ctap1RegisterResponse> for MakeCredentialResponse {
    fn from(response: Ctap1RegisterResponse) -> Self {
        unimplemented!()
    }
}

impl From<Ctap1SignResponse> for GetAssertionResponse {
    fn from(_: Ctap1SignResponse) -> Self {
        unimplemented!()
    }
}
