use std::time::Duration;
use tracing_subscriber::EnvFilter;
use std::sync::mpsc;
mod block;
mod storage;
mod networking;
mod node;


#[async_std::main]
async fn main() {
    let (tx, rx) = mpsc::channel();

    let mut node = node::Node::new(rx,tx);

    // Fetch keypair from Env variable - if not present, generate a new keypair

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    async_std::task::spawn(async move {node.p2p.p2phandler().await;});
    async_std::task::spawn(async move {
        loop {
            //MAIN LOOP GOES HERE
            

            //Vote queue setup
            //Vote queue full -> Create block -> Add block to blockchain -> Broadcast block




            async_std::task::sleep(Duration::from_secs(1)).await;
        }  
    });
    
    
    
    // let mut blockchain = block::Blockchain::new(rx);
    async_std::task::spawn(async move {node.blockchain.check_queue();});
    
    loop{}
    
}