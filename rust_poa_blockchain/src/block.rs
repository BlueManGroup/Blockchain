use sha2::{Sha256, Digest};
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use crate::storage;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub index: u64,
    timestamp: i64,
    prev_block_hash: String,
    hash: String,
    data: String,
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
}
#[derive(Debug)]
pub struct Blockchain {
    chain: Vec<Block>,
    pub authorities: Vec<String>, // Public keys or identifiers of authorized nodes
    file_tracker: storage::FileTracker
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_block = Block::new(0, Utc::now().timestamp(), String::new(), "Genesis Block".to_string());
        let file_tracker = storage::FileTracker::new(1, String::from("blocks"));
        Blockchain {
            chain: vec![genesis_block],
            authorities: Vec::new(), // Initialize with known authorities
            file_tracker: file_tracker
        }
    }

    pub fn add_block(&mut self, data: String, authority: String) {
        if !self.authorities.contains(&authority) {
            println!("Authority not recognized.");
            return;
        }

        self.file_tracker.find_file();
        println!("current block: {}", self.file_tracker.cur_block);

        let prev_block = &self.chain[self.chain.len() - 1];
        let new_block = Block::new(
            self.file_tracker.cur_block,
            Utc::now().timestamp(),
            prev_block.hash.clone(),
            data,
        );
        

        storage::append_blocks_to_file(&[&new_block], &mut self.file_tracker);
        self.chain.push(new_block);
        println!("tracker in blocks after increment: {}", self.file_tracker.cur_enum);
        self.file_tracker.cur_block += 1 ;
        println!("cur_block: {}", self.file_tracker.cur_block);
    }
}   