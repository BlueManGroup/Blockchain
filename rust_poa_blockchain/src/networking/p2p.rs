
use base64::{engine::general_purpose, Engine};
use futures::prelude::*;
use libp2p::Multiaddr;
use libp2p::swarm::SwarmEvent;
use libp2p::swarm::Swarm;
//use std::collections::HashMap;
use std::time::Duration;
use libp2p::floodsub::{FloodsubEvent, Topic};
use libp2p::identity;
use libp2p::identity::Keypair;
use libp2p::PeerId; 
use std::sync::mpsc::{Sender, Receiver}; 
use std::env;
//use std::str;x
use dotenv::dotenv;
use crate::networking::behaviour;
use base64;
use tokio::{io, io::AsyncBufReadExt, select};
//use crate::Payload;
//use crate::block::Block;

pub struct P2p {
    pub swarm: Swarm<behaviour::Behaviour>,
    pub behaviour: behaviour::Behaviour,
    pub local_peer_id: PeerId,
    pub identity_keys: identity::Keypair,
    pub listen_addr: Multiaddr,
    pub floodsub_topic: Topic,
    pub known_nodes: Vec<(String, String)>,
    pub inc_msg_queue: Sender<&[u8]>,
    pub out_msg_queue: Receiver<String>
}

impl P2p{
    pub fn new(inc_msg_queue: Sender<String>, out_msg_queue: Receiver<String>) -> Self {
        //let identity_keys= Keypair::generate_ed25519();
        dotenv().ok();

        // BELOW STATEMENT COMMENTED OUT AS PEER_ID IS ASSUMED TO BE PRESENT WHEN RUNNING THIS PROGRAM
        // SHOULD PROBABLY BE UNCOMMENTED BUT CHANGED TO BE ERROR HANDLING INSTEAD
        // if env::var("PEER_ID").is_err() {
        //     // create peerid from key and store it in .env
        //     let peer_id_bytes = identity_keys.public().to_peer_id().to_bytes();
        //     let peer_id = general_purpose::STANDARD.encode(&peer_id_bytes);
        //     env::set_var("PEER_ID", peer_id);
        // };

        // fetch peer id (stored as b64) and convert it to peerid object
        let local_peer_id_b64 = dotenv::var("PEER_ID").unwrap();
        let local_peer_id_decoded = general_purpose::STANDARD.decode(local_peer_id_b64);
        let local_peer_id = PeerId::from_bytes(&local_peer_id_decoded.unwrap());
        //let local_peer_id = PeerId::from_bytes(general_purpose::STANDARD.decode_slice(dotenv::var("PEER_ID").unwrap().as_bytes()));
        let local_peer_id = match local_peer_id {
            Ok(peer_id) => peer_id,
            // IKKE GODT VVVVV
            Err(e) => panic!("{}", e)
        };

    

        //get libp2p identity keys from env file, deserialize for further use
        
        let env_identity_keys_str = dotenv::var("P2P_IDENTITY_KEYS").unwrap();
        let env_identity_keys = env_identity_keys_str.as_bytes();
        let identity_keys = Keypair::from_protobuf_encoding(env_identity_keys).unwrap();

    //     let env_identity_keys = dotenv::var("P2P_IDENTITY_KEYS").unwrap().as_bytes();
    //   let identity_keys = Keypair::from_protobuf_encoding(env_identity_keys).unwrap();

        let behaviour = behaviour::Behaviour::new(identity_keys.public(), local_peer_id.clone()).expect("Failed to create behaviour");
        let behaviour2 = behaviour::Behaviour::new(identity_keys.public(), local_peer_id.clone()).expect("Failed to create behaviour");
        let listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().expect("Failed to parse listen address");
        let floodsub_topic: Topic = Topic::new("blockchain".to_string());


        // idk publickey og peer_id eller sådan noget - husk at ændre boot method til at have samme format
        let mut known_nodes: Vec<(String, String)> = {
            let mut known_nodes = Vec::new();
            known_nodes.push((dotenv::var("DILITHIUM3").unwrap(), dotenv::var("PEERID").unwrap()));
            known_nodes};
        // add random node values for debugging purposes
        // REMEMBER TO REMOVE
        // PUBLIC KEY DILITHIUM3, PEERID

        

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(identity_keys.clone())
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::tls::Config::new,
                libp2p::yamux::Config::default,
            ).unwrap()
            .with_behaviour(|_| behaviour).unwrap()
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();
        swarm.behaviour_mut().floodsub.subscribe(floodsub_topic.clone());
        libp2p::Swarm::listen_on(&mut swarm, listen_addr.clone()).expect("Failed to listen on address");
        
        let p2p = P2p {
            swarm,
            behaviour: behaviour2,
            local_peer_id: local_peer_id.clone(),
            identity_keys,
            listen_addr,
            floodsub_topic,
            known_nodes,
            inc_msg_queue,
            out_msg_queue
        };
        p2p
    }

    //iterate over vector to find entry where ip == ip we're looking for
    //then get the peer id at the same index (technically not that but w/e) 
    pub fn get_key_from_peer_id(&self, peer_id: String) -> Option<String> {
        self.known_nodes
            .iter()
            .find(|(_, node_peer)| *node_peer == peer_id)
            .map(|(key, _)| key.clone())
    }

    //((((((reverse version of the above method))))))
    pub fn get_peer_id_from_key(&self, key: String) -> Option<String> {
        self.known_nodes
            .iter()
            .find(|(node_key, _)| *node_key == key)
            .map(|(_, peer_id)| peer_id.clone())
    }

    pub async fn p2phandler(&mut self) {
        match self.swarm.select_next_some().await {
            
            //Write to console when listening on address
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {address:?}"),

            //Handle inbound Floodsub messages
            SwarmEvent::Behaviour(behaviour::BehaviourEvent::Floodsub(FloodsubEvent::Message (message))) => {
                println!("message received");
                println!("Received:'{:?}' from {:?}", String::from_utf8_lossy(&message.data), &message.source);
                let test: &[u8] = &message.data;
                self.inc_msg_queue.send(*test).expect("ooppssssiiiieeees");
            },

            //Handle inbound Floodsub subscriptions
            SwarmEvent::Behaviour(behaviour::BehaviourEvent::Floodsub(FloodsubEvent::Subscribed {peer_id, topic})) => {

                println!("Peer {:?} subscribed to '{:?}'", &peer_id, &topic);
                let message_str = format!("Hello {}", peer_id.clone().to_string()).into_bytes();
                self.swarm.behaviour_mut().floodsub.publish(self.floodsub_topic.clone(), message_str);
            },

            // //Mdns Handler, poorly optimised as it dials all known nodes on discovery, but maybe not?
            // SwarmEvent::Behaviour(behaviour::BehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
            //     for (peer_id, addr) in peers {
            //         println!("Discovered: {:?} at {:?}", peer_id, addr);
            //         self.known_nodes.push((peer_id.clone().to_string(), addr.to_string()));
            //         self.swarm.dial(addr).expect("Failed to dial address");
            //     }
            // },

            // SwarmEvent::Behaviour(behaviour::BehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
            //     for (peer_id, addr) in peers {
            //         println!("Expired: {:?} at {:?}", peer_id, addr);
            //     }
            // },

            //Handle Identify events
            SwarmEvent::Behaviour(behaviour::BehaviourEvent::Identify(event)) => {
            
                match event {
                    libp2p::identify::Event::Received {info, peer_id} => {
                        println!("Received: {:?} from {:?}", info, peer_id);
                        self.swarm.behaviour_mut().floodsub.add_node_to_partial_view(peer_id.clone());
                        self.known_nodes.push((peer_id.clone().to_string(), info.agent_version));

                        let message_str = format!("Hello {}", peer_id.clone().to_string()).into_bytes();
                        self.swarm.behaviour_mut().floodsub.publish_any(self.floodsub_topic.clone(), message_str);

                    },
                    libp2p::identify::Event::Sent {peer_id} => {
                        println!("Sent: {:?}", peer_id);
                    },
                    libp2p::identify::Event::Pushed {info, peer_id} => {
                        println!("Pushed: {:?} from {:?}", info, peer_id )   
                    },
                    libp2p::identify::Event::Error { peer_id, error } => {
                        println!("Error: {:?} from {:?}", error, peer_id);
                    },
                };
                    
                // swarm.behaviour_mut().floodsub.add_node_to_partial_view(local_peer_id);
            },

            //handler for Request response
            SwarmEvent::Behaviour(behaviour::BehaviourEvent::Reqres(event)) => {

                match event {
                    libp2p::request_response::Event::InboundFailure { peer, request_id, error } => {
                        print!("Inbound failure: {:?}", error)
                    },
                    
                    libp2p::request_response::Event::OutboundFailure { peer, request_id, error } => {
                        print!("Outbound failure: {:?}", error)
                    },

                    libp2p::request_response::Event::Message { peer, message } => {
                        print!("Message: {:?}", message)
                    },

                    libp2p::request_response::Event::ResponseSent { peer, request_id } => {
                        print!("Response sent: {:?}", request_id)
                    }
                }
            }
            _ => {}
        }
    
    }

    pub async fn send_block_to_nodes(&mut self, msg: String) -> Result<(),()> {
        //let block_str = serde_json::to_string(&msg).unwrap();
        self.swarm.behaviour_mut().floodsub.publish(self.floodsub_topic.clone(), msg.as_bytes().to_owned());
        println!("sent msg: {:?}", msg);
        Ok(())
    }

    // pub async fn 
    // target = (dilithiumkey, peerid)
    
    pub async fn give_node_the_boot(&mut self, target: (String, String)) -> Result<(),()> {
        let target_ip; 
        let target_key;
        if target.0 != "NULL" {
            target_ip = self.get_peer_id_from_key(target.0.clone()).unwrap();
            target_key = target.0;
        } else {
            target_key = self.get_key_from_peer_id(target.1.clone()).unwrap();
            target_ip = target.1;
        };
        
        // let local_peer_id_decoded = general_purpose::STANDARD.decode(local_peer_id_b64);
        // let local_peer_id = PeerId::from_bytes(&local_peer_id_decoded.unwrap());
        // //let local_peer_id = PeerId::from_bytes(general_purpose::STANDARD.decode_slice(dotenv::var("PEER_ID").unwrap().as_bytes()));
        // let local_peer_id = match local_peer_id {
        //     Ok(peer_id) => peer_id,
        //     // IKKE GODT VVVVV
        //     Err(e) => panic!("{}", e)
        // };
        println!("node boot wooo maybe vi ser");
        if let Some(index) = self.known_nodes.iter().position(|(k, v)| *k == target_key || *v == target_ip) {
            self.known_nodes.remove(index);
            println!("node removed!!");
        }
        println!("known nodes: {:?}", self.known_nodes);
        let target_public_key = identity::PublicKey::try_decode_protobuf(target_key.as_bytes()).expect("error getting key from bytes");
        let target_peer_id = PeerId::from_public_key(&target_public_key);
        
        println!("node boot done :sunglasses:");
        
        self.swarm.behaviour_mut().floodsub.remove_node_from_partial_view(&target_peer_id);

        

        Ok(())
    }

}
//2 p || !2 p