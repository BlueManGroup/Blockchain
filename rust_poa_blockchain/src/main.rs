use futures::prelude::*;
use libp2p::{ping, Multiaddr};
use libp2p::swarm::SwarmEvent;
use std::error::Error;
use std::time::Duration;
use tracing_subscriber::EnvFilter;
use libp2p::floodsub::{Floodsub, FloodsubEvent, Topic};
use libp2p::identity;
use libp2p::identity::Keypair;
use libp2p::PeerId;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    
    let local_key = Keypair::generate_ed25519();
    let floodsub_topic: Topic = Topic::new("blockchain".to_string());
    
        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_async_std()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::tls::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|_| ping::Behaviour::default())?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(30)))
        .build();


    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    if let Some(addr) = std::env::args().nth(1) {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {addr}")
    }


    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {address:?}"),
            SwarmEvent::Behaviour(event) => println!("{event:?}"),
            _ => {}
        }
    }
    

    //     loop for testing blockchain locally
    //     println!("Please choose an option:");
    //     println!("1. Add a block");
    //     println!("2. Print one block");
    //     println!("3. Print all blocks");
    //     println!("4. validate block");
    //     println!("5. Exit");
    

    // let mut choice = String::new();
    // let mut indexstring = String::new();
    // std::io::stdin().read_line(&mut choice).expect("Failed to read line");

    //     match choice.trim().parse() {
    //         Ok(1) => {
    //             println!("Enter data for the new block:");
    //             let mut data = String::new();
    //             std::io::stdin().read_line(&mut data).expect("Failed to read line");
    //             blockchain.add_block(data, "Authority1".to_string());
    //         },
    //         Ok(2) => {
    //             std::io::stdin().read_line(&mut indexstring).expect("Failed to read line");
    //             let index = indexstring.trim().parse::<usize>().unwrap();
    //             println!("{:?}", &blockchain.chain[index]);
            
    //         },
    //         Ok(3) => {
    //             for block in &blockchain.chain {
    //                 println!("{:?}", block);
    //             }
    //         },
    //         Ok(4) => {
    //             std::io::stdin().read_line(&mut indexstring).expect("Failed to read line");
    //             let index = indexstring.trim().parse::<usize>().unwrap();
    //             if blockchain.is_block_valid(index) {
    //                 println!("Block is valid.");
    //             } else {
    //                 println!("Block is invalid.");
                
    //             }
    //         },
    //         _ => {
    //             println!("Invalid option. Please enter a number between 1 and 3.");
    //         },
    //     }
}
