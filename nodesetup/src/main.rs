use std::env;
use dotenv;
use std::path::{Path};
use libp2p::identity;
use libp2p::identity::Keypair;
use libp2p::PeerId; 
use base64::{engine::general_purpose, Engine};
use pqcrypto_dilithium::dilithium3;
use pqcrypto_traits::sign::{PublicKey,SecretKey, SignedMessage};
use std::fs::OpenOptions;
use std::io::Write;
use std::fs::File;


fn main() {
    // construct absolute path for .env file
    let my_path = env::home_dir().and_then(|a| Some(a.join("/.env"))).unwrap();
    dotenv::dotenv().ok();
    let path = "././rust_poa_blockchain/src/.env";

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("./src/.env")
        .expect("Unable to open file");
    println!("{:?}", file);
    

    //set up Identity keys for libp2p
    
     let identity_keys = Keypair::generate_ed25519();
    //  let identity_keys_bytes = identity_keys.to_protobuf_encoding();
    //  env::set_var("P2P_IDENTITY_KEYS", identity_keys_bytes.unwrap());
    
    let identity_keys_bytes = identity_keys.to_protobuf_encoding().unwrap();
    let identity_keys_string = base64::encode(&identity_keys_bytes);
    file.write_all(format!("P2P_IDENTITY_KEYS={}\n", identity_keys_string).as_bytes());

    //env::set_var("P2P_IDENTITY_KEYS", identity_keys_string);

    //set up peer id for libp2p (derived from libp2p identity keys)
    let peer_id_bytes = identity_keys.public().to_peer_id().to_bytes();
    let peer_id = general_purpose::STANDARD.encode(&peer_id_bytes);
    file.write_all(format!("P2P_PEER_ID={}\n", peer_id).as_bytes());

    //set up dilithium keys
    let (publickey, secretkey) = dilithium3::keypair();
    let secretkey_bytes = secretkey.as_bytes();
    let publickey_bytes = publickey.as_bytes();


    file.write_all(format!("SECRETKEY={}\n", general_purpose::STANDARD.encode(secretkey_bytes)).as_bytes());
    file.write_all(format!("PUBLICKEY={}\n", general_purpose::STANDARD.encode(publickey_bytes)).as_bytes());


    //set up known nodes table (peer id, dilithium pub key)
    //is done manually for now :(
}