
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
use libp2p::mdns;
use std::sync::mpsc::{Sender, Receiver}; 
use std::env;
//use std::str;x
use dotenv::dotenv;
use crate::networking::behaviour;
use base64;
//use tokio::{io, io::AsyncBufReadExt, select};
//use crate::Payload;
//use crate::block::Block;

pub struct P2p {
    pub swarm: Swarm<behaviour::Behaviour>,
    pub behaviour: behaviour::Behaviour,
    pub local_peer_id: PeerId,
    pub identity_keys: identity::Keypair,
    pub listen_addr: Multiaddr,
    pub floodsub_topic: Topic,
    pub known_nodes: Vec<(Vec<u8>,Vec<u8>)>,
    pub inc_msg_queue: Sender<Vec<u8>>,
    pub out_msg_queue: Receiver<String>
}

fn parse_tuple_string(tuple_str: &str) -> Result<(Vec<u8>, Vec<u8>), &str> {
    let parts: Vec<&str> = tuple_str.split(',').collect();
    if parts.len() != 2 {
        return Err("Invalid tuple format");
    }
    let peerid = general_purpose::STANDARD.decode(parts[0].trim().to_string()).unwrap();
    let pubkey = general_purpose::STANDARD.decode(parts[1].trim().to_string()).unwrap();
    Ok((peerid, pubkey))
}   

impl P2p{
    pub fn new(inc_msg_queue: Sender<Vec<u8>>, out_msg_queue: Receiver<String>) -> Self {
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
        let local_peer_id_b64 = dotenv::var("P2P_PEER_ID").unwrap();
        let local_peer_id_decoded = general_purpose::STANDARD.decode(local_peer_id_b64);
        let local_peer_id = PeerId::from_bytes(&local_peer_id_decoded.unwrap()).unwrap();
        //let local_peer_id = PeerId::from_bytes(general_purpose::STANDARD.decode_slice(dotenv::var("PEER_ID").unwrap().as_bytes()));
        println!("peer_id: {:?}", local_peer_id);
        // let local_peer_id = match local_peer_id {
        //     Ok(peer_id) => peer_id,
        //     // IKKE GODT VVVVV
        //     Err(e) => panic!("{}", e)
        // };

        //get libp2p identity keys from env file, deserialize for further use
        
        let env_identity_keys_b64 = dotenv::var("P2P_IDENTITY_KEYS").unwrap();
        let env_identity_keys_bytes = general_purpose::STANDARD.decode(env_identity_keys_b64).unwrap();
        let identity_keys = Keypair::from_protobuf_encoding(&env_identity_keys_bytes).unwrap();

    //     let env_identity_keys = dotenv::var("P2P_IDENTITY_KEYS").unwrap().as_bytes();
    //   let identity_keys = Keypair::from_protobuf_encoding(env_identity_keys).unwrap();

        let behaviour = behaviour::Behaviour::new(identity_keys.public(), local_peer_id.clone()).expect("Failed to create behaviour");
        let behaviour2 = behaviour::Behaviour::new(identity_keys.public(), local_peer_id.clone()).expect("Failed to create behaviour");
        let listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().expect("Failed to parse listen address");
        let floodsub_topic: Topic = Topic::new("blockchain".to_string());


        // idk publickey og peer_id eller sådan noget - husk at ændre boot method til at have samme format
        let known_node_string = env::var("KNOWN_HOSTS").expect("Environment variable not found");
        let known_nodes_raw: Vec<Result<(Vec<u8>, Vec<u8>), &str>> = known_node_string
            .split(";")
            .map(parse_tuple_string)
            .collect();
        let mut known_nodes: Vec<(Vec<u8>, Vec<u8>)> = known_nodes_raw.into_iter().filter_map(Result::ok).collect();
        // println!("known nodes: {:?}", known_nodes);
        let me = (local_peer_id.to_string(), dotenv::var("PUBLICKEY").unwrap());
        let me_decoded = general_purpose::STANDARD.decode(me.0).unwrap();
        known_nodes.retain(|(peer_id, _)| peer_id != &me_decoded);
        

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
    pub fn get_key_from_peer_id(&self, peer_id: Vec<u8>) -> Option<Vec<u8>> {
        self.known_nodes
            .iter()
            .find(|(_, node_peer)| *node_peer == peer_id)
            .map(|(key, _)| key.clone())
    }

    //((((((reverse version of the above method))))))
    pub fn get_peer_id_from_key(&self, key: Vec<u8>) -> Option<Vec<u8>> {
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
                let msg_vec: Vec<u8> = message.data.to_vec();
                self.inc_msg_queue.send(msg_vec).expect("error passing message to inc queue");
            },

            //handler for Request response
            SwarmEvent::Behaviour(behaviour::BehaviourEvent::Reqres(event)) => {

                match event {
                    libp2p::request_response::Event::InboundFailure { peer, request_id, error } => {
                        println!("Inbound failure: {:?}", error)
                    },
                    
                    libp2p::request_response::Event::OutboundFailure { peer, request_id, error } => {
                        println!("Outbound failure: {:?}", error)
                    },

                    libp2p::request_response::Event::Message { peer, message } => {
                        match message {
                            libp2p::request_response::Message::Request { request_id, request, channel } => {
                                println!("req_id: {:?}, req: {:?}, channel: {:?}, peer: {:?}", request_id, request, channel, peer);
                                let msg_bytes = request.message.as_bytes().to_vec();
                                self.inc_msg_queue.send(msg_bytes).expect("error passing message to inc queue");
                            }
                            libp2p::request_response::Message::Response { request_id, response } => {
                                println!("req_id: {:?}, req: {:?}, peer: {:?}", request_id, response, peer);
                            }
                        }
                    },

                    libp2p::request_response::Event::ResponseSent { peer, request_id } => {
                        println!("Response sent: {:?}", request_id)
                    }
                }
            }

            //Handle inbound Floodsub subscriptions
            SwarmEvent::Behaviour(behaviour::BehaviourEvent::Floodsub(FloodsubEvent::Subscribed {peer_id, topic})) => {

                println!("Peer {:?} subscribed to '{:?}'", &peer_id, &topic);
                //let message_str = format!("Hello {}", peer_id.clone().to_string()).into_bytes();
                //self.swarm.behaviour_mut().floodsub.publish(self.floodsub_topic.clone(), message_str);
            },

            //Mdns Handler, poorly optimised as it dials all known nodes on discovery, but maybe not?
            SwarmEvent::Behaviour(behaviour::BehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                for (peer_id, addr) in peers {
                    println!("Discovered: {:?}", peer_id);
                    let contains_node = self.known_nodes.iter().any(|(ref known_peer, _)| *known_peer == peer_id.to_bytes());
                    
                    // for (peerid, _) in &self.known_nodes {
                    //     println!("peerid in contacts: {:?} \tpeerid discovered: {:?} \t match: {:?}", peerid, peer_id.to_bytes(), peerid == &peer_id.to_bytes());
                    // }

                    if contains_node {
                        println!("Node known");
                        self.swarm.dial(addr).expect("Failed to dial address");
                        self.swarm.behaviour_mut().floodsub.add_node_to_partial_view(peer_id.clone());
                    } else {
                        println!("Node unknown, has not been added to contact list");
                    }   
                }
            },

            SwarmEvent::Behaviour(behaviour::BehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                for (peer_id, addr) in peers {
                    println!("Expired: {:?} at {:?}", peer_id, addr);
                }
            },

            //Handle Identify events
            SwarmEvent::Behaviour(behaviour::BehaviourEvent::Identify(event)) => {
            
                match event {
                    libp2p::identify::Event::Received {info, peer_id} => {
                        println!("Received: {:?} from {:?}", info, peer_id);                    

                        // let message_str = format!("Hello {}", peer_id.clone().to_string()).into_bytes();
                        // self.swarm.behaviour_mut().floodsub.publish_any(self.floodsub_topic.clone(), message_str);

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
    //
    pub async fn give_node_the_boot(&mut self, target: (Vec<u8>, Vec<u8>)) -> Result<(),()> {
        let target_ip; 
        let target_key;
        if target.0.is_empty() {
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
        let target_public_key = identity::PublicKey::try_decode_protobuf(&target_key).expect("error getting key from bytes");
        let target_peer_id = PeerId::from_public_key(&target_public_key);
        
        println!("node boot done :sunglasses:");
        
        self.swarm.behaviour_mut().floodsub.remove_node_from_partial_view(&target_peer_id);

        

        Ok(())
    }

}
//2 p || !2 p