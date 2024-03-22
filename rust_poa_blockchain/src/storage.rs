use serde_json::{json};
use std::fs::OpenOptions;
use std::io::prelude::*;
use glob::glob;
use std::fs::File;
use crate::block::Block;

#[derive(Debug)]
pub struct FileTracker {
    pub cur_election: u16,
    pub cur_enum: u16,
    pub cur_block: u64,
    path: String
} 

impl FileTracker{
    pub fn new(election: u16, path: String) -> Self {
        let mut file_tracker = FileTracker {
            cur_election: election,
            cur_enum: 1,
            cur_block: 0,
            path: path
        };

        file_tracker

    }

    pub fn find_file(&self) -> u64 {
        let pattern = format!("{}/e{}f*", self.path, self.cur_election);
        let greatest = glob(&pattern)
            .expect("poopie")
            .filter_map(Result::ok) 
            .filter_map(|entry| entry.file_name().map(|name| name.to_string_lossy().to_string()))
            .filter_map(|name| name.strip_prefix(&format!("e{}f", self.cur_election)).map(|stripped| stripped.parse::<u32>().ok()).flatten())
            .max();

        if let Some(greatest_number) = greatest {
            let filename = format!("{}/e{}f{}", self.path, self.cur_election, greatest_number);
            println!("{}", filename);
            let file_contents = std::fs::read_to_string(&filename)
                .unwrap_or_else(|err| {
                    eprintln!("Error reading file: {}", err);
                    String::new()
                });
                
                
            let mut biggest: u64 = 0;
            for line in file_contents.lines() {
                if let Some(block) = serde_json::from_str::<Block>(&line).ok() {
                    println!("Block index: {}", block.index);
                    biggest = std::cmp::max(biggest, block.index);
                } else {
                    eprintln!("Error parsing JSON: {}", line);
                }
            }
            return biggest;
        } else {
            eprintln!("No files found matching the pattern");
        }
        panic!("oops!");
    }

  
} 

// location til q
// 
pub fn append_blocks_to_file(blocks: &[&Block], cur_election: u16, cur_enum: u16) -> std::io::Result<()> {


    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("blocks/e{}f{}", cur_election, cur_enum))?;
    
    for block in blocks {
        let serialized_block = json!(block).to_string();
        file.write_all(serialized_block.as_bytes())?;
        file.write_all(b"\n")?; 
        
    }
    Ok(())
}
