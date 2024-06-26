use std::env;
use crate::{block, networking};
use std::sync::mpsc::{Sender, Receiver};
use pqcrypto_dilithium::dilithium3;
use pqcrypto_traits::sign::{PublicKey,SecretKey};
use base64::{engine::general_purpose, Engine};
use rand::seq::SliceRandom;
//use std::io;
// use sha2::{Digest};
// use crate::block::Block;
// use serde_json::Value;
use serde::{Serialize, Deserialize};
use libp2p::PeerId; 
use crate::networking::reqres;


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
        println!("topayload tries this: {:?}", value);
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

pub struct Node {
    pub secretkey: dilithium3::SecretKey,
    pub publickey: dilithium3::PublicKey,
   // pub node_list: // array med strings(?)
    pub blockchain: block::Blockchain,
    pub p2p: networking::p2p::P2p,
    pub out_msg_tx: Sender<String>,
    pub in_msg_rx: Receiver<Vec<u8>>

}

impl Node {
    pub fn new(in_msg_tx: Sender<Vec<u8>>, in_msg_rx: Receiver<Vec<u8>>,  out_msg_tx: Sender<String>, out_msg_rx: Receiver<String> ) -> Self {

        
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

        let mut blockchain = block::Blockchain::new();
        blockchain.init_file_tracker();
        //create p2p network
        let mut p2p = networking::p2p::P2p::new(in_msg_tx, out_msg_rx);

        //return object
        // CHANGE NAME TO PEERID
        let node = Node {
            secretkey,
            publickey,
            blockchain,
            p2p,
            out_msg_tx,
            in_msg_rx
        };
        node
    }

    pub fn create_block_payload(&self, block: block::Block) -> Payload {
        let mut payload_msg = Vec::new();
        payload_msg.extend_from_slice(); // send hash of block in signature
        let name_clone = self.p2p.local_peer_id.to_string(); // send name (peerid ?) for easy identification (receiver can just look up pubkey)
        let payload = Payload::new(block, name_clone, dilithium3::sign(block.hash.as_bytes(), &self.secretkey));
        
        payload
    }

    // pub fn sign_payload(&self, payload: Payload) {
    //     // encrypt payload string med private key
            
    // }

    // select a random validator from the list of known nodes
    pub fn select_validator(&self) -> std::io::Result<((Vec<u8>, Vec<u8>), (Vec<u8>, Vec<u8>))> {
        // initialize random number generator
        let mut rng = rand::thread_rng();
        // select 2 validators randomly and make them to a vec of tuples
        let validators: Vec<(Vec<u8>, Vec<u8>)> = self.p2p.known_nodes.choose_multiple(&mut rng, 2).cloned().collect(); 

        // return the vec of tuples
        Ok((validators[0].clone(), validators[1].clone()))
        
    }

    // oh enough til en block!! woow
    // lav blcok whoaaa
    // lav en payload på denne bloc kwoooooot
    // randomly vælg 2(?) seje guys
    // send ud til disse seje guys!!! haha
    // disse seje guys modtager og validerer dem!!!!!!!!!
    // hvis ikke er du blevet bortvist!!!!!!
    // de sender dem herefter videre til de resterende (mindre) seje guys

    pub async fn send_block_to_validator(&mut self, payload: Payload) -> std::io::Result<()> {
        let validators = self.select_validator();
        
        let serialized_payload = serde_json::to_string(&payload).unwrap();
        let (validator1,validator2) = validators.unwrap();
        self.send_message(&validator1.0, serialized_payload.as_str());
        self.send_message(&validator2.0, serialized_payload.as_str()).await;
        
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

    pub async fn validate_block(&mut self, payload: Payload) -> Result<bool, ()> {
        let validation_result = self.blockchain.is_block_valid(payload.block);
        // INDSÆT STIKPRØVER HER

        if validation_result == false {
            self.p2p.give_node_the_boot(("NULL".as_bytes().to_owned(), payload.author_id.as_bytes().to_owned())).await.unwrap();
        }
        Ok(validation_result) 
    }

    pub async fn create_validator_payload(&mut self, payload_bytes: &[u8]) -> Result<ValidatorPayload,()> {
        let deseralized_payload = Node::deserialize_message(payload_bytes).unwrap();
        //let block_msg = Block::to_block(deseralized_payload.get("block").unwrap().to_owned());
        // let sig_msg_val = payload_msg.get("signature").unwrap();
        
        let payload = Payload::to_payload(deseralized_payload);

        let validation_result = self.validate_block(payload.clone()).await.unwrap(); // HUSK MÅSKE AT ÆNDRE

        let validator_payload;
        
        if validation_result {
            let validator_signature = dilithium3::sign(payload.block.hash.as_bytes(), &self.secretkey);
            let validator_id = self.p2p.local_peer_id.to_bytes();
            validator_payload = ValidatorPayload::new(payload, validator_id, validator_signature);
        } else {
           return Err(());
        }

        

        Ok(validator_payload)
    }

    pub async fn interpret_message(&mut self, message: &[u8]) -> Result<bool, ()> {
        // declare (and maybe initialize) vars used in method
        let deserialized_message = Node::deserialize_message(message).unwrap();
        let decrypted_signature;
        let mut is_block_good: bool = false;

        let validator_payload: ValidatorPayload;
        let payload: Payload;
        let mut out_msg: String = String::new();

        // if validator payload enter this abomination
        if let Some(validator_sig) = deserialized_message.get("validator_id") {
            validator_payload = ValidatorPayload::to_validator_payload(deserialized_message.to_owned());
            payload = validator_payload.payload.to_owned();

            if self.p2p.known_nodes.iter().any(|( _,v)| *v == validator_sig.to_owned().to_string().as_bytes() ) {
                if self.p2p.known_nodes.iter().any(|( _,v)| *v == payload.author_id.to_owned().as_bytes()) {
                    is_block_good = true;
                } else {
                    return Ok(false)
                }
            } else {
                return Ok(false)
            }
            // get the public key from the validatorpayload and decrypt the signature
        
            let validator_pub_key_string = self.p2p.get_key_from_peer_id(validator_payload.validator_id.to_owned()).unwrap();
            let validator_pub_key: dilithium3::PublicKey = dilithium3::PublicKey::from_bytes(&validator_pub_key_string).unwrap();

            decrypted_signature = dilithium3::open(&validator_payload.validator_sig, &validator_pub_key).unwrap();

            // check whether or not the hash in the signature matches with the block itself
            if String::from_utf8(decrypted_signature).unwrap() == payload.block.hash {
                is_block_good = true;
            }
        } else {
            // if not validator payload, enter this lesser abomination
            // validates block and creates a validator payload for further transmission
            payload = Payload::to_payload(deserialized_message);

            if self.p2p.known_nodes.iter().any(|(_, v)| *v == payload.author_id.to_owned().as_bytes()) {
                is_block_good = true;
                println!("block is good!!");
            } else {
                return Ok(false)
            }
            // THEN add to blockchain
            //might create an error here, not sure
        
            validator_payload = self.create_validator_payload(message).await.unwrap();
            
        }
        
        
       // do things if not is block good
       if !is_block_good {
            // give block boot and some other thingys
            self.p2p.give_node_the_boot(("NULL".as_bytes().to_owned(), payload.author_id.as_bytes().to_owned())).await.unwrap();
            println!("not is block good!");
       }


       self.blockchain.add_block(payload.block.to_owned(), payload.author_id.to_owned());
       out_msg = serde_json::to_string(&validator_payload).unwrap();
       println!("message sent to node(s): {:?}", out_msg);
       self.out_msg_tx.send(serde_json::to_string(&out_msg).unwrap()).unwrap();
       // self.p2p.send_block_to_nodes(payload); IDK MAKE METHOD TO SEND VALIDATOR BLOCK TO NODES
       // add validator payload to queue that sends out

       Ok(is_block_good)
    }

    pub async fn check_inc_queue(&mut self) {
        let msg = self.in_msg_rx.try_recv().unwrap_or_default();
        if msg.is_empty() {
            return;
        }
        let res = self.interpret_message(&msg).await.unwrap();
        println!("block validated: {:?}", res);
    }

    pub async fn ping(&mut self, peerid_bytes: Vec<u8>) {
        
        let payload = reqres::GreetRequest {
            message: "ping".to_string(),
        };
        
        let peerid = PeerId::from_bytes(&peerid_bytes).unwrap();
        self.p2p.behaviour.reqres.send_request(&peerid, payload);
    
    }

    pub async fn send_message(&mut self, peerid_bytes: &Vec<u8>, message: &str) {
        
        let payload = reqres::GreetRequest {
            message: message.to_string()
        };
        
        let peerid = PeerId::from_bytes(peerid_bytes).unwrap();
        self.p2p.behaviour.reqres.send_request(&peerid, payload);
    
    }

    
}
