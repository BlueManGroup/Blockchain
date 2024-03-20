use serde_json::{json};
use std::fs::OpenOptions;
use std::io::prelude::*;
use crate::block::Block;

// location til q
// 
pub fn append_blocks_to_file(blocks: &[&Block]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("test.dat")?;

    for block in blocks {
        let serialized_block = json!(block).to_string();
        file.write_all(serialized_block.as_bytes())?;
        file.write_all(b"\n")?; 
    }
    Ok(())
}
