
use tracing_subscriber::EnvFilter;
use std::sync::mpsc;
use tokio::{io as tio, io::AsyncBufReadExt, select};
use std::error::Error;
use libp2p;
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
                        println!("ping one of following peers:");
                        let mut count: usize = 0;
                        node.p2p.known_nodes.iter().for_each(|(peer_id, _)| {
                            let peer_id_parsed = libp2p::PeerId::from_bytes(peer_id).unwrap();
                            println!("Peer {}: {:?}", count, peer_id_parsed);
                            count = count + 1;
                        });
                        let input = stdin.next_line().await.unwrap().unwrap();
                        let parsed_input = input.parse::<usize>().unwrap();
                        if parsed_input >= count {
                            println!("Invalid peer");
                            continue;
                        } else {
                            let remote_peer = node.p2p.known_nodes[parsed_input].0.to_owned();
                            node.ping(remote_peer).await;
                            println!("tried to ping peer wooooo");
                        }
                    }
                    "send block" => {
                        println!("sending test block");
                        let block = node.blockchain.new_local_block(String::from("poopie :D"));
                        let payload = node.create_block_payload(block);
                        node.send_block_to_validator(payload).await.unwrap();                   
                    }
                    "list peers" => {
                        node.p2p.known_nodes.iter().for_each(|(peer_id, _)| {
                            let peer_id_parsed = libp2p::PeerId::from_bytes(peer_id).unwrap();
                            println!("Peer: {:?}", peer_id_parsed);
                        });
                    }
                    "list swarm contacts" => {
                        node.p2p.swarm.external_addresses().for_each(|addr| {
                            println!("Swarm contact: {:?}", addr);
                        });
                    }
                    "test add block" => {
                        println!("adding test block");
                        let block = node.blockchain.new_local_block(String::from("poopie 2 :D"));
                        // let payload = node.create_block_payload(block); // should work, but doesn't, too bad
                        // node.p2p.send_blockbytes_to_nodes(payload.to_bytes()).await.unwrap();
                        node.p2p.send_blockbytes_to_nodes(block.to_bytes()).await.unwrap();
                    }
                    _ => {
                        println!("Invalid command");
                    }
                }
            }
            _ = node.p2p.p2phandler() => {
                // println!("p2phandler did something!");
                node.check_inc_queue().await;
            
            }
            // _ = node.check_inc_queue() => {

        //}
  //  }

        //     default => {
                
        //     }
            
        // t1 => {
        //     // Do work
            
        }
    }
}

    // Main asks user what to do (Menu in a loop)
    // - Do something that requires ownership of node.p2p
    //   - asynchronously while listening for incoming messages

    // while True:
    //    Do work
    //    Listen for incoming messages

    //loop
