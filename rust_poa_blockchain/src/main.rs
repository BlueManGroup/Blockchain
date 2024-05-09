
use std::time::Duration;
use futures::{
    future::FutureExt, // for `.fuse()`
    pin_mut,
    select,
};
use tracing_subscriber::EnvFilter;
use std::sync::{mpsc};
use std::io::{self, Write};
use libp2p::PeerId;
use tokio::{io, io::AsyncBufReadExt, select};
use std::error::Error;
mod block;
mod storage;
mod networking;
mod node;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{ 
    let (inc_tx, inc_rx) = mpsc::channel();
    let (out_tx, out_rx) = mpsc::channel();

    let mut node = node::Node::new(inc_tx, inc_rx, out_tx, out_rx);

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let mut stdin = io::BufReader::new(io::stdin()).lines();


    loop {
        //main loop here
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                    
                }   
            msg = node.p2p.p2phandler() => {
                
                }

            }

        //     default => {
                
        //     }
            
        // t1 => {
        //     // Do work
            
    }
}

    // Main asks user what to do (Menu in a loop)
    // - Do something that requires ownership of node.p2p
    //   - asynchronously while listening for incoming messages

    // while True:
    //    Do work
    //    Listen for incoming messages

    //loop
