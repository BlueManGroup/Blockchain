use futures::prelude::*;
use libp2p::Multiaddr;
use libp2p::swarm::SwarmEvent;
use libp2p::swarm::Swarm;
use std::time::Duration;
use libp2p::floodsub::{FloodsubEvent, Topic};
use libp2p::identity;
use libp2p::identity::Keypair;
use libp2p::PeerId; 
use std::sync::mpsc::Sender; 
use crate::networking::behaviour;



pub struct P2p {
    pub swarm: Swarm<behaviour::Behaviour>,
    pub behaviour: behaviour::Behaviour,
    pub local_peer_id: PeerId,
    pub identity_keys: identity::Keypair,
    pub listen_addr: Multiaddr,
    pub floodsub_topic: Topic,
    pub msg_queue: Sender<String>,
}

impl P2p{
    pub fn new(msg_queue: Sender<String>) -> Self {
        let identity_keys= Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(identity_keys.public());
        let behaviour = behaviour::Behaviour::new(identity_keys.public(), local_peer_id.clone()).expect("Failed to create behaviour");
        let behaviour2 = behaviour::Behaviour::new(identity_keys.public(), local_peer_id.clone()).expect("Failed to create behaviour");
        let listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().expect("Failed to parse listen address");
        let floodsub_topic: Topic = Topic::new("blockchain".to_string());
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
            msg_queue,
        };
        p2p
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