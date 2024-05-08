use std::env;
use crate::{block, networking};
use std::sync::mpsc;
use pqcrypto_dilithium::dilithium3;
use pqcrypto_traits::sign::{PublicKey,SecretKey};
use base64::{engine::general_purpose, Engine};
use std::collections::HashMap;
use sha2::{Sha256, Digest};
use rand::seq::SliceRandom;
use std::io;
use crate::block::Block;
use serde::{Serialize, Deserialize};
use libp2p::PeerId; 


#[derive(Serialize, Deserialize)]
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
}

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
}

pub struct Node {
    pub secretkey: dilithium3::SecretKey,
    pub publickey: dilithium3::PublicKey,
   // pub node_list: // array med strings(?)
    pub blockchain: block::Blockchain,
    pub p2p: networking::p2p::P2p,
    

}

impl Node {
    pub fn new(msg_rx: mpsc::Receiver<String>, msg_tx: mpsc::Sender<String> ) -> Self {

        
        // good for PoC, maybe bad for production
        let secretkey: dilithium3::SecretKey;
        let publickey: dilithium3::PublicKey;
        // refactor later to not allow creating keypair yourself
        // below code fetches (or creates new if none present) keypair from env file
        // change this to be error handling instead of keypair generation
        if env::var("SECRETKEY").is_ok() && env::var("PUBLICKEY").is_ok() {
            let secretkey_bytes = env::var("SECRETKEY").unwrap().into_bytes();
            let publickey_bytes = env::var("PUBLICKEY").unwrap().into_bytes();

            secretkey = dilithium3::SecretKey::from_bytes(&secretkey_bytes).unwrap();
            publickey = dilithium3::PublicKey::from_bytes(&publickey_bytes).unwrap();
        } else {
            (publickey, secretkey) = dilithium3::keypair();
            let secretkey_bytes = secretkey.as_bytes();
            let publickey_bytes = publickey.as_bytes();

            env::set_var("SECRETKEY", general_purpose::STANDARD.encode(secretkey_bytes));
            env::set_var("PUBLICKEY", general_purpose::STANDARD.encode(publickey_bytes));
        }

        let blockchain = block::Blockchain::new(msg_rx);

        //create p2p network
        let mut p2p = networking::p2p::P2p::new(msg_tx);

        //return object
        // CHANGE NAME TO PEERID
        let node = Node {
            secretkey,
            publickey,
            blockchain,
            p2p 
        };
        node
    }

    pub fn create_block_payload(&self, block: block::Block) -> Payload {
        let mut payload_msg = Vec::new();
        payload_msg.extend_from_slice(block.hash.as_bytes()); // send hash of block in signature
        payload_msg.extend_from_slice(self.p2p.local_peer_id.to_bytes().as_slice()); //send peerid in signature for further confirmation that im me
        let name_clone = self.p2p.local_peer_id.to_string(); // send name (peerid ?) for easy identification (receiver can just look up pubkey)
        let payload = Payload::new(block, name_clone, dilithium3::sign(payload_msg.as_slice(), &self.secretkey));
        
        payload
    }

    // pub fn sign_payload(&self, payload: Payload) {
    //     // encrypt payload string med private key
            
    // }

    // select a random validator from the list of known nodes
    pub fn select_validator(&self) -> std::io::Result<&(String, String)> {
        if let Some(chosen_validator) = self.p2p.known_nodes.choose(&mut rand::thread_rng()) {
            println!("validator randomly selected woooo: {:?}", &chosen_validator);
            Ok(chosen_validator)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "error finding validator"))
        }
    }

    pub fn send_block_to_validator(&self, payload: Payload, dest: String) -> std::io::Result<()> {
        
        //self.p2p.known_nodes.get(&dest);
        //self.p2p.swarm.dial
        
        // BLOCK CREATOR BURDE MÅSKE CHECKE OM BLOCKEN ER SOM DET SKAL VÆRE INDEN DEN BLIVER SENDT TIL RESTEN AF NETVÆRKET
        // ^ikke helt enig længere, vi burde i stedet checke hos alle at hashet med blocken stemmer overens med hvad der står i signature
        // det kan måske give for meget delay, hvis creator giver en good for alle blocks de har created (ift hvad der sendes, ikke samlet 
        // needed processing). dertil kan receiving nodes heller ikke checke for dem selv hvorvidt en block er good (kan de godt hvis begge
        // gøre, men det vil være endnu mere processing power needed)
        

        Ok(())
    }

    pub fn deserialize_message(message: &[u8]) -> Result<serde_json::Value, serde_json::Error> {
        let payload = serde_json::json!(message);
        Ok(payload)
    }

    pub fn deserialize_validator_message(message: &[u8]) -> Result<serde_json::Value, serde_json::Error> {
        let payload = serde_json::json!(message);
        Ok(payload)
    }

    pub fn validate_block(&mut self, payload: Payload) -> Result<(bool), ()> {
        let validation_result = self.blockchain.is_block_valid(payload.block);
        // INDSÆT STIKPRØVER HER

        if validation_result == false {
            self.p2p.give_node_the_boot((String::from("NULL"), payload.author_id));
        }
        Ok((validation_result)) 
    }

    pub fn create_validator_payload(&self, payload_bytes: &[u8]) -> Result<(ValidatorPayload),()> {
        let payload_msg = Node::deserialize_message(payload_bytes).unwrap();
        let block_msg = Block::new(payload_msg.get("block").unwrap().get("index").un)
        let payload = Payload::new(
            payload_msg.get("block"),               
            payload_msg.get("author_id").unwrap(),
            payload_msg.get("signature").unwrap()     
        );

        let validation_result = self.validate_block(payload).unwrap(); // HUSK MÅSKE AT ÆNDRE

        let validator_payload;
        
        if validation_result {
            let validator_signature = dilithium3::sign(payload.block.hash, &self.secretkey);
            let validator_id = self.p2p.local_peer_id.to_bytes();
            validator_payload = ValidatorPayload::new(payload, validator_id, validator_signature);
        }

        Ok((validator_payload))
    }

    pub fn interpret_message(&self, message: &[u8]) -> Result<(bool),()> {
        // declare (and maybe initialize) vars used in method
        let deserialized_message = Node::deserialize_message(message).unwrap();
        let objectified_message;
        let decrypted_signature;
        let is_block_good: bool = false;
        let block;

        // check if validator_sig exists in the json sent from the message
        // if it does, treat as validated payload 
        // if not, we have to validate the payload
        // check whether if any of deserialized_message.validator_id, deserialized_message.payload.block.author_id, deserialized_message.block.author_id
        if let Some(validator_sig) = deserialized_message.get("validator_id") {

            if self.p2p.known_nodes.iter().any(|( _,v)| *v == deserialized_message.get("validator_id").unwrap() ) {
                if self.p2p.known_nodes.iter().any(|( _,v)| *v == deserialized_message.get("payload").unwrap().get("author_id").unwrap() ) {
                    is_block_good = true;
                } else {
                    Ok(false)
                }
            } else {
                Ok((false))
            }
            // get the public key from the validatorpayload and decrypt the signature
            let validator_pub_key_string = self.p2p.get_key_from_peer_id(PeerId::from_bytes(deserialized_message.validator_id)).unwrap();
            let validator_pub_key: dilithium3::PublicKey = dilithium3::PublicKey::from_bytes(validator_pub_key_string.as_bytes());
            decrypted_signature = dilithium3::open(&deserialized_message.validator_sig, &validator_pub_key);



            // check whether or not the hash in the signature matches with the block itself
            if decrypted_signature == deserialized_message.payload.block.hash {
                is_block_good = true;
            }
        } else {
            //Pass to create_validator_payload
            if self.p2p.known_nodes.iter().any(|(_, v)| *v == deserialized_message.author_id) {
                is_block_good = true;
            } else {
                Ok((false))
            }
            // THEN add to blockchain
            //might create an error here, not sure
            let validator_payload = self.create_validator_payload(deserialized_message).unwrap();
            self.p2p.inc_msg_queue.send(validator_payload);
        }
        
        

       // do things if not is block good
       if !is_block_good {
            // give block boot and some other thingys
            self.p2p.give_node_the_boot((String::from("NULL"), deserialized_message.payload.validator_id));
       }

       Ok(is_block_good)
        
    }

    
}




/*
ids = public keys
Payload: block, creatorID
    Hash: Payload
Signature: Hash + encrypted with private key
Validator Id
validator signature
*/