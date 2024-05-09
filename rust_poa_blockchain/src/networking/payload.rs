use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::block;
use pqcrypto_dilithium::dilithium3;

#[derive(Serialize, Deserialize, Clone)]
pub struct Payload {
    pub block: block::Block,
    pub signature: dilithium3::SignedMessage,
    pub author_id: String,
}

impl Payload {
    pub fn new (block: block::Block, author_id: String, signature: dilithium3::SignedMessage) -> Self {

        let payload = Payload {
            block,
            signature,
            author_id,
        };
        payload
    }

    pub fn to_payload(value: serde_json::Value) -> Payload {
        serde_json::from_value(value).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ValidatorPayload {
    pub payload: Payload,
    pub validator_id: Vec<u8>,
    pub validator_sig: dilithium3::SignedMessage
}

impl ValidatorPayload {
    pub fn new(payload: Payload, validator_id: Vec<u8>, validator_sig: dilithium3::SignedMessage) -> Self {
        let validator_payload = ValidatorPayload {
            payload,
            validator_id,
            validator_sig
        };
        validator_payload
    }

    pub fn to_validator_payload(value: serde_json::Value) -> ValidatorPayload {
        serde_json::from_value(value).unwrap()
    }
}
