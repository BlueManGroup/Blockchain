use base64::{engine::general_purpose, Engine};
use futures::prelude::*;
use libp2p::Multiaddr;
use libp2p::swarm::SwarmEvent;
use libp2p::swarm::Swarm;
use std::collections::HashMap;
use std::time::Duration;
use libp2p::floodsub::{FloodsubEvent, Topic};
use libp2p::identity;
use libp2p::identity::Keypair;
use libp2p::PeerId; 
use std::sync::mpsc::Sender; 
use std::env;
use std::str;
use dotenv::dotenv;
use crate::networking::behaviour;
use base64;



pub struct P2p {
    pub swarm: Swarm<behaviour::Behaviour>,
    pub behaviour: behaviour::Behaviour,
    pub local_peer_id: PeerId,
    pub identity_keys: identity::Keypair,
    pub listen_addr: Multiaddr,
    pub floodsub_topic: Topic,
    pub known_nodes: Vec<(String, String)>,
    pub msg_queue: Sender<String>,
}

impl P2p{
    pub fn new(msg_queue: Sender<String>) -> Self {
        let identity_keys= Keypair::generate_ed25519();
        dotenv().ok();

        if env::var("PEER_ID").is_err() {
            // create peerid from key and store it in .env
            let peer_id_bytes = identity_keys.public().to_peer_id().to_bytes();
            let peer_id = general_purpose::STANDARD.encode(&peer_id_bytes);
            env::set_var("PEER_ID", peer_id);
        };
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

        let behaviour = behaviour::Behaviour::new(identity_keys.public(), local_peer_id.clone()).expect("Failed to create behaviour");
        let behaviour2 = behaviour::Behaviour::new(identity_keys.public(), local_peer_id.clone()).expect("Failed to create behaviour");
        let listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().expect("Failed to parse listen address");
        let floodsub_topic: Topic = Topic::new("blockchain".to_string());
        let mut known_nodes: Vec<(String, String)> = Vec::new(); 
        // add random node values for debugging purposes
        // REMEMBER TO REMOVE
        known_nodes.push((String::from("test"), String::from("test2")));
        known_nodes.push((String::from("pis"), String::from("pis2")));
        known_nodes.push((String::from("shit"), String::from("shit2")));
        known_nodes.push((String::from("validat"), String::from("validat")));
        known_nodes.push((String::from("johannes"), String::from("johannes2")));

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(identity_keys.clone())
            .with_async_std()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::tls::Config::new,
                libp2p::yamux::Config::default,
            ).unwrap()
            .with_behaviour(|_| behaviour).unwrap()
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();
        swarm.behaviour_mut().floodsub.subscribe(floodsub_topic.clone());
        
        let p2p = P2p {
            swarm,
            behaviour: behaviour2,
            local_peer_id: local_peer_id.clone(),
            identity_keys,
            listen_addr,
            floodsub_topic,
            known_nodes,
            msg_queue,
        };
        p2p
    }

    //iterate over vector to find entry where ip == ip we're looking for
    //then get the peer id at the same index (technically not that but w/e) 
    pub fn get_key_from_ip(&self, ip_address: String) -> Option<String> {
        self.known_nodes
            .iter()
            .find(|(_, node_ip)| *node_ip == ip_address)
            .map(|(peer_id, _)| peer_id.clone())
    }

    //((((((reverse version of the above method))))))
    pub fn get_ip_by_key(&self, key: String) -> Option<String> {
        self.known_nodes
            .iter()
            .find(|(node_key, _)| *node_key == key)
            .map(|(_, ip_address)| ip_address.clone())
    }
    

    pub async fn p2phandler(&mut self) {
    libp2p::Swarm::listen_on(&mut self.swarm, self.listen_addr.clone()).expect("Failed to listen on address");

    if let Some(addr) = std::env::args().nth(1) {
        let remote: Multiaddr = addr.parse().expect("Failed to parse remote address");
        self.swarm.dial(remote).expect("Failed to dial remote address");
        println!("Dialed {addr}")
    };

        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {address:?}"),
                SwarmEvent::Behaviour(behaviour::BehaviourEvent::Floodsub(FloodsubEvent::Message (message))) => {
                    println!("message received");
                    println!("Received:'{:?}' from {:?}", String::from_utf8_lossy(&message.data), &message.source);
                    let message_str = String::from_utf8_lossy(&message.data).into_owned();
                    self.msg_queue.send(message_str).expect("ooppssssiiiieeees");
                },
                SwarmEvent::Behaviour(behaviour::BehaviourEvent::Floodsub(FloodsubEvent::Subscribed {peer_id, topic})) => {

                    println!("Peer {:?} subscribed to '{:?}'", &peer_id, &topic);
                    let message_str = format!("Hello {}", peer_id.clone().to_string()).into_bytes();
                    self.swarm.behaviour_mut().floodsub.publish(self.floodsub_topic.clone(), message_str);
                },
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
                _ => {}
            }
        
        }
    
    }
}
//2 p || !2 p