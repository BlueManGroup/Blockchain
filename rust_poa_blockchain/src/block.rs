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
        let file_tracker = storage::FileTracker::new(1, String::from("blocks"));
        Blockchain {
            chain: vec![genesis_block],
            authorities: Vec::new(), // Initialize with known authorities
            file_tracker: file_tracker
        }
    }

    pub fn add_block(&mut self, block: Block, authority: String) {
        if !self.authorities.contains(&authority) {
            println!("Authority not recognized.");
            return;
        }

        storage::append_blocks_to_file(&[&block], &mut self.file_tracker).unwrap();
        self.chain.push(block);
        println!("tracker in blocks after increment: {}", self.file_tracker.cur_enum);
        self.file_tracker.cur_block += 1 ;
        println!("cur_block: {}", self.file_tracker.cur_block);
    }

    pub fn init_file_tracker(&mut self) {
        self.file_tracker.find_file();
    }

    // call when enough votes to construct a whole block.
    // adds block to local chain, doesn't yet push to other nodes
    pub fn new_local_block(&mut self, data: String) -> Block {
        println!("current block: {}", self.file_tracker.cur_block);

        let prev_block = &self.chain[self.chain.len() - 1];
        let new_block = Block::new(
            self.chain.len() as u64,
            Utc::now().timestamp(),
            prev_block.hash.clone(),
            data,
        );

        storage::append_blocks_to_file(&[&new_block], &mut self.file_tracker).unwrap();
        self.chain.push(new_block.to_owned());
        println!("tracker in blocks after increment: {}", self.file_tracker.cur_enum);
        self.file_tracker.cur_block += 1 ;
        println!("cur_block: {}", self.file_tracker.cur_block);
        new_block
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
    //             self.new_local_block(received, auth);
    //         }
         //}
    //}
}
