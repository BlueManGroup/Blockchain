use sha2::{Sha256, Digest};
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use crate::storage;


#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub prev_block_hash: String,
    pub hash: String,
    pub data: String,
}

impl Block {
    
    pub fn new(index: u64, timestamp: i64, prev_block_hash: String, data: String) -> Self {
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

    pub fn calculate_hash(&self) -> String {
        let headers = format!("{}{}{}{}", &self.index, &self.timestamp, &self.prev_block_hash, &self.data);
        let mut hasher = Sha256::new();
        hasher.update(headers);
        let hash = hasher.finalize();
        format!("{:x}", hash)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.index.to_be_bytes());
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes.extend_from_slice(self.prev_block_hash.as_bytes());
        bytes.extend_from_slice(self.hash.as_bytes());
        bytes.extend_from_slice(self.data.as_bytes());
        bytes
    }

    pub fn to_block(value: serde_json::Value) -> Block {
        serde_json::from_value(value).unwrap()
    }
}

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub authorities: Vec<String>, // Public keys or identifiers of authorized nodes
    file_tracker: storage::FileTracker,
}

/// Represents a blockchain.
impl Blockchain {
    /// Creates a new instance of the blockchain.
    ///
    /// # Returns
    ///
    /// A new instance of the blockchain.
    pub fn new() -> Self {
        let genesis_block = Block::new(0, Utc::now().timestamp(), String::new(), "Genesis Block".to_string());
            
        let blockchain = Blockchain {
            chain: vec![genesis_block.clone()], // clone the genesis block to keep it in memory
            authorities: Vec::new(), // Initialize with known authorities
            file_tracker: storage::FileTracker::new(1, String::from("blocks")),
        };

        storage::append_blocks_to_file(&[&genesis_block], blockchain.file_tracker.cur_election, blockchain.file_tracker.cur_enum).expect("mangler fil ved path:");

        blockchain
    }

    /// Adds a new block to the blockchain.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to be stored in the new block.
    /// * `authority` - The authority responsible for adding the block.
    /// HUSK AT TAGE TILBAGE TIL STRING PÅ AUTH (FJERNET FOR TESTING PURPOSES)
    pub fn add_block(&mut self, data: String, authority: String, timestamp: i64, index: u64) {
        // if !self.authorities.contains(&authority) {
        //     println!("Authority not recognized.");
        //     return;
        // }

        let prev_block = &self.chain[self.chain.len() - 1];
        let new_block = Block::new(
            index,
            timestamp,
            prev_block.hash.clone(),
            data,
        );
        self.file_tracker.cur_block = self.file_tracker.find_file();

        storage::append_blocks_to_file(&[&new_block], self.file_tracker.cur_election, self.file_tracker.cur_enum).expect("error adding block to local chain");
        self.chain.push(new_block);
        self.file_tracker.cur_block += 1 ;
        println!("{}", self.file_tracker.cur_block);
    }

    
    /// Checks if a given block in the blockchain is valid. needs to be rewritten to check blocks not yet on the chain
    ///
    /// # Arguments
    ///
    /// * `block` - The block to be checked.
    ///
    /// # Returns
    ///
    /// `true` if the block is valid, `false` otherwise.
    pub fn is_block_valid(&self, block: Block) -> bool {
        // if index == 0 {
        //     return true; // Genesis block is always valid
        // }

        // let block = &self.chain[index];
        // let prev_block_index = block.index - 1;
        // let prev_block = &self.chain[prev_block_index as usize];
        // let prev_block_hash = prev_block.hash.clone();
        // let calculated_prev_block_hash = prev_block.calculate_hash();

        // block.prev_block_hash == prev_block_hash && prev_block_hash == calculated_prev_block_hash 

        let last_block_index = self.chain.len() - 1;
        let cur_block = &self.chain[last_block_index];
        if block.index != cur_block.index {
            return false;
        }

        if block.prev_block_hash != cur_block.hash {
            return false;
        }

        if block.calculate_hash() != block.hash {
            return false;
        }

        true
    } 

    // pub fn check_queue(&mut self) {
    //     loop {
    //         if let Ok(received) = self.msg_queue.try_recv() {
    //             let auth = 1;
    //             self.add_block(received, auth);
    //         }
         //}
    //}
}
