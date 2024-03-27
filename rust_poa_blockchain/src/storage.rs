use serde_json::{json};
use std::fs::OpenOptions;
use std::io::prelude::*;
use glob::glob;
use std::fs::File;
use crate::block::Block;
use std::fs;

struct FileTracker {
    cur_election: u16,
    cur_enum: u16,
    cur_block: u32,
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

    pub fn find_file(&self) {
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
    
            for line in file_contents.lines() {
                if let Some(block) = serde_json::from_str::<Block>(&line).ok() {
                    println!("Block index: {}", block.index);
                } else {
                    eprintln!("Error parsing JSON: {}", line);
                }
            }
        } else {
            eprintln!("No files found matching the pattern");
        }
        
    }

  
} 

// create a new file. do so when current file too big
pub fn create_new_file(file_tracker: &FileTracker) -> std::io::Result<()> {
    let file = format!("{}/e{}f{}", file_tracker.path, file_tracker.cur_election, file_tracker.cur_enum);
    //File::create(file_tracker.path);
    Ok(())
}

// location til q
// 
pub fn append_blocks_to_file(blocks: &[&Block]) -> std::io::Result<()> {
    let file_tracker = FileTracker::new(1, String::from("blocks"));
    file_tracker.find_file();

    let file_path = format!("blocks/e{}f{}", file_tracker.cur_election, file_tracker.cur_enum);
    let mut file = File::create(&file_path)?;

    for block in blocks {
        let serialized_block = json!(block).to_string();
        file.write_all(serialized_block.as_bytes())?;
        file.write_all(b"\n")?;
    }
    file.sync_all()?;
    //if (file.metadata().unwrap().len() >= )
    println!("size: {}", file.metadata().unwrap().len());
    Ok(())
}
