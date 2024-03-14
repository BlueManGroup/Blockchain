use chrono::prelude::*;
use sha2::{Sha256, Digest};

#[derive(Debug)]
struct BlockData {
    election: String, // could also just be int. identifies which election this block was created for
    vote: String, // what the user voted for. should be in something not plaintext
    user: Stirng, // user identifier. should be anonymized
    validator: String, // identity of the validator. could be integer as well
}

struct Block {
    index: u64, // block index
    timestamp: i64, // self explanatory
    prev_block_hash: String, // also self explanatory
    hash: String, // block's own hash
    data: BlockData //
}

impl Block {
    fn new(index: u64, timestamp: i64, prev_block_hash: String, data: BlockData) -> Self {
        let mut block = Block {
            index,
            timestamp,
            prev_block_hash,
            hash: String::new(),
            data,
        };
        block.hash = block.calculate_hash();
        block
    }
//Benis
    fn calculate_hash(&self) -> String {
        let headers = format!("{}{}{}{}", &self.index, &self.timestamp, &self.prev_block_hash, &self.data);
        let mut hasher = Sha256::new();
        hasher.update(headers);
        let hash = hasher.finalize();
        format!("{:x}", hash)
    }
}
#[derive(Debug)]
struct Blockchain {
    chain: Vec<Block>,
    authorities: Vec<String>, // Public keys or identifiers of authorized nodes
}

impl Blockchain {
    fn new() -> Self {
        let genesis_block = Block::new(0, Utc::now().timestamp(), String::new(), "Genesis Block".to_string());
        Blockchain {
            chain: vec![genesis_block],
            authorities: Vec::new(), // Initialize with known authorities
        }
    }

    fn add_block(&mut self, data: String, authority: String) {
        // edit to check some key thingy 
        if !self.authorities.contains(&authority) {
            println!("Authority not recognized.");
            return;
        }

        let prev_block = &self.chain[self.chain.len() - 1];
        let new_block = Block::new(
            self.chain.len() as u64,
            Utc::now().timestamp(),
            prev_block.hash.clone(),
            data,
        );
        self.chain.push(new_block);
    }

    // Add more methods as needed, such as validating the chain, adding authorities, etc.
}
fn main() {
    let mut blockchain = Blockchain::new();
    
    // Simulate adding authority public keys or identifiers
    blockchain.authorities.push("Authority1".to_string());
    
    // Attempt to add a block
    blockchain.add_block("Block 1 Data".to_string(), "Authority1".to_string());
    
    println!("{:#?}", blockchain);
}
