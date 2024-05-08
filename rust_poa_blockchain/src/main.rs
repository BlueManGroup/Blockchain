use std::time::Duration;
use tracing_subscriber::EnvFilter;
use std::sync::{mpsc};
use std::io::{self, Write};
use libp2p::PeerId;
mod block;
mod storage;
mod networking;
mod node;


#[async_std::main]
async fn main() { 
    let (inc_tx, inc_rx) = mpsc::channel();
    let (out_tx, out_rx) = mpsc::channel();

    let mut node = node::Node::new(inc_tx, inc_rx, out_tx, out_rx);

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let _ = node.select_validator();


//     async_std::task::spawn(
//         async move {node.p2p.p2phandler().await;}
// );
    
    async_std::task::spawn(async move {
        loop {

            match event:

                console events:


                network events















            //MAIN LOOP GOES HERE
            //Vote queue setup
            //Vote queue full -> Create block -> Add block to blockchain -> Broadcast block

            //test Command line menu
            // Display menu options
            println!("== Command Line Menu ==");
            println!("1. nodes");
            println!("2. ping peer");
            println!("3. Exit");
            print!("Enter your choice: ");
            io::stdout().flush().expect("Error flushing stdout");

            // Read user input
            let mut choice = String::new();
            io::stdin()
                .read_line(&mut choice)
                .expect("Failed to read input");
            // Process user choice
            match choice.trim() {
                "1" => {
                    println!("You selected Option One");
                    println!("Known nodes:");
                    known_nodes.iter().for_each(|x| println!("{:?}",x));
                }
                "2" => {
                    println!("You selected Option Two");
                    print!("Select peer to ping: ");
                    for (index, peerid) in known_nodes.iter().enumerate() {
                        println!("{}. {:?}", index + 1, peerid);
                    };
                    io::stdin().read_line(&mut choice).expect("Failed to read input");
                    match choice.trim().parse::<usize>() {
                        Ok(index) => {
                            if index > 0 && index <= known_nodes.len() {
                                let peerid = &known_nodes[index - 1];
                                node_ref.ping(peerid.1.as_str()).await;
                                println!("Invalid index");
                            }
                            }
                        Err(_) => {
                            println!("Invalid index");
                            }
                        }
                }
                "3" => {
                    println!("Exiting...");
                    break;
                }
                _ => {
                    println!("Invalid choice, please try again.");
                }
            }

        println!(); // Adds a blank line before the menu reappears


            async_std::task::sleep(Duration::from_secs(1)).await;
        }  
    });
    
    
    
    // let mut blockchain = block::Blockchain::new(rx);
    //async_std::task::spawn(async move {node.blockchain.check_queue();});
    
    loop{}
    
}