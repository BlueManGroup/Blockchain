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
        // Serialize the struct to a JSON string
        let json_string = serde_json::to_string(self);
        // Convert the JSON string to bytes
        json_string.unwrap().into_bytes()
    }

    pub fn to_block(value: serde_json::Value) -> Block {
        serde_json::from_value(value).unwrap()
    }
}

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
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
            file_tracker: file_tracker
        }
    }

    pub fn add_block(&mut self, block: Block) {
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
        if block.index != cur_block.index + 1{
            print!("{:?}", block.index);
            print!("{:?}", cur_block.index);
            println!("index mismatch");
            return false;
            
        }

        if block.prev_block_hash != cur_block.hash {
            print!("prev hash mismatch");
            return false;
        }

        if block.calculate_hash() != block.hash {
            print!("hash mismatch");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_block() {
        let block = Block::new(1, 1627836000, String::from("1ee04c825754c08520edd070f7c1cbab79a91098ebd4f3afe994f37fbb659bf6"), String::from("data"));
        assert_eq!(block.index, 1);
        assert_eq!(block.timestamp, 1627836000);
        assert_eq!(block.prev_block_hash, "1ee04c825754c08520edd070f7c1cbab79a91098ebd4f3afe994f37fbb659bf6");
        assert_eq!(block.data, "data");
    }

    #[test]
    fn test_calculate_hash() {
        let block = Block::new(1, 1627836000, String::from("1ee04c825754c08520edd070f7c1cbab79a91098ebd4f3afe994f37fbb659bf6"), String::from("data"));
        let hash = block.calculate_hash();
        assert_eq!(hash.len(), 64); // SHA256 hash length is 64 characters
    }

    #[test]
    fn test_to_bytes() {
        let block = Block::new(1, 1627836000, String::from("1ee04c825754c08520edd070f7c1cbab79a91098ebd4f3afe994f37fbb659bf6"), String::from("data"));
        let bytes = block.to_bytes();
        assert_eq!(bytes.len(), 148); // 8 bytes for index, 8 bytes for timestamp, 124 bytes for prev_block_hash, 32 bytes for hash, 16 bytes for data
    }

    #[test]
    fn test_to_block() {
        let json = serde_json::json!({
            "index": 1,
            "timestamp": 1627836000,
            "prev_block_hash": "1ee04c825754c08520edd070f7c1cbab79a91098ebd4f3afe994f37fbb659bf6",
            "hash": "block_hash",
            "data": "data"
        });
        let block = Block::to_block(json);
        assert_eq!(block.index, 1);
        assert_eq!(block.timestamp, 1627836000);
        assert_eq!(block.prev_block_hash, "1ee04c825754c08520edd070f7c1cbab79a91098ebd4f3afe994f37fbb659bf6");
        assert_eq!(block.hash, "block_hash");
        assert_eq!(block.data, "data");
    }

    #[test]
    fn test_new_blockchain() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1); // Genesis block should be added
        assert_eq!(blockchain.file_tracker.cur_block, 0);
    }

    #[test]
    fn test_add_block() {
        let mut blockchain = Blockchain::new();
        let block = Block::new(1, 1627836000, String::from("1ee04c825754c08520edd070f7c1cbab79a91098ebd4f3afe994f37fbb659bf6"), String::from("data"));
        blockchain.add_block(block);
        assert_eq!(blockchain.chain.len(), 2); // New block should be added
        assert_eq!(blockchain.file_tracker.cur_block, 1);
    }

    #[test]
    fn test_new_local_block() {
        let mut blockchain = Blockchain::new();
        let block = blockchain.new_local_block(String::from("data"));
        assert_eq!(blockchain.chain.len(), 2); // New block should be added
        assert_eq!(blockchain.file_tracker.cur_block, 1);
        assert_eq!(block.data, "data");
    }

    #[test]
    fn test_is_block_valid() {
        let mut blockchain = Blockchain::new();
        blockchain.new_local_block("data".to_string());
        let block2 = Block::new(2, 1627836001, String::from(&blockchain.chain[1].hash), String::from("data"));
        assert!(blockchain.is_block_valid(block2));
    }
}