
use tracing_subscriber::EnvFilter;
use std::sync::mpsc;
use tokio::{io as tio, io::AsyncBufReadExt, select};
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

    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let mut stdin = tio::BufReader::new(tio::stdin()).lines();


    loop {
        //main loop here
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                match line.as_str() {
                    "list all" => {
                        node.p2p.known_nodes.iter().for_each(|(peer_id, _)| {
                            println!("Peer: {:?}", peer_id);
                        });
                        
                    }
                    "ping" => {
                        println!("Pinging all peers");
                        
                    }
                    _ => {
                        println!("Invalid command");
                    }
                }
            }   
            _ = node.p2p.p2phandler() => {
                node.check_inc_queue().await;
            }
            // _ = node.check_inc_queue() => {

            // }
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
