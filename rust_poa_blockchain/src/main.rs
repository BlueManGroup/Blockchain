use std::time::Duration;
use tracing_subscriber::EnvFilter;
use std::sync::mpsc;
mod block;
mod storage;
mod networking;


#[async_std::main]
async fn main() {
    let (tx, rx) = mpsc::channel();    

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let mut p2p = networking::p2p::P2p::new(tx);
    async_std::task::spawn(async move {p2p.p2phandler().await;});
    async_std::task::spawn(async move {
        loop {
            async_std::task::sleep(Duration::from_secs(1)).await;
        }  
    });
    let mut blockchain = block::Blockchain::new(rx);
    async_std::task::spawn(async move {blockchain.check_queue();});
    
    loop{}
    
}