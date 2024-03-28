mod block;
mod storage;

fn main() {
    let mut blockchain = block::Blockchain::new();
    
    // Simulate adding authority public keys or identifiers
    blockchain.authorities.push("Authority1".to_string());

    
    // Attempt to add a block
    blockchain.add_block("Block 1 Data".to_string(), "Authority1".to_string());
    blockchain.add_block("Block 2 Data".to_string(), "Authority1".to_string());
    blockchain.add_block("Block 3 Data".to_string(), "Authority1".to_string());
    blockchain.add_block("Block 4 Data".to_string(), "Authority1".to_string());
    blockchain.add_block("Block 5 Data".to_string(), "Authority1".to_string());
    blockchain.add_block("Block 6 Data".to_string(), "Authority1".to_string());
    //println!("{:#?}", blockchain);
}
