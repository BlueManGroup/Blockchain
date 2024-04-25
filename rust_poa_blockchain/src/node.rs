use std::env;
use crate::{block, networking};
use std::sync::mpsc;
use pqcrypto_dilithium::dilithium3;
//use pqcrypto_dilithium::dilithium3::SignedMessage;
use pqcrypto_traits::sign::{PublicKey,SecretKey, };
use base64::{engine::general_purpose, Engine};
//use std::collections::HashMap;
//use sha2::{Sha256, Digest};
use rand::seq::SliceRandom;
use std::io;
use std::sync::mpsc::Receiver;
use crate::block::Block;
use serde::Serialize;

#[derive(Serialize)]
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
    pub validator_id: String,
    pub validator_sig: dilithium3::SignedMessage
}

impl ValidatorPayload {
    pub fn new(payload: Payload, validator_id: String, validator_sig: dilithium3::SignedMessage) -> Self {
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
    pub name: String,
    pub p2p: networking::p2p::P2p,
    pub out_msg_queue: Receiver<Block>
}

impl Node {
    pub fn new() -> Self {

        let (inc_msg_tx, inc_msg_rx) = mpsc::channel();
        let (out_msg_tx, out_msg_rx) = mpsc::channel();
        
        // good for PoC, maybe bad for production
        let secretkey: dilithium3::SecretKey;
        let publickey: dilithium3::PublicKey;
        // refactor later to not allow creating keypair yourself
        // below code fetches (or creates new if none present) keypair from env file
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


        let blockchain = block::Blockchain::new(inc_msg_rx, out_msg_tx);

        //create p2p network
        let p2p = networking::p2p::P2p::new(inc_msg_tx);
        
        //return object
        let node = Node {
            secretkey,
            publickey,
            blockchain,
            name: String::from("Node"),
            p2p,
            out_msg_queue: out_msg_rx
        };
        node
    }

    pub fn create_block_payload(&self, block: block::Block) -> Payload {
        let mut payload_msg = Vec::new();
        payload_msg.extend_from_slice(block.to_bytes().as_slice());
        payload_msg.extend_from_slice(self.name.as_bytes());
        let name_clone = self.name.clone(); // Clone the name string
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

    pub async fn send_block_to_network(&mut self) {
        let block = self.out_msg_queue.recv().expect("error fetching block from q");
        let payload = self.create_block_payload(block);
        let _ = self.p2p.send_block_to_nodes(payload).await;
        println!("sent block to p2p");
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