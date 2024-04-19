use libp2p::{
    floodsub::Floodsub,
    swarm::NetworkBehaviour,
    PeerId, identify, 
};
use libp2p_identity::PublicKey;

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub floodsub: Floodsub,
    pub identify: identify::Behaviour,
} 

impl Behaviour {
    pub fn new(local_public_key: PublicKey, local_peer_id: PeerId) -> Result<Self, Box<dyn std::error::Error>> {
        let floodsub = Floodsub::new(local_peer_id.clone());
        let identify = identify::Behaviour::new(identify::Config::new("1.0".into(), local_public_key));
        Ok(Self {identify, floodsub})
    }
}
// fn main() {
//     // Generate a new identity keypair
//     let identity_keys = identity::Keypair::generate_ed25519();
//     let local_peer_id = PeerId::from(identity_keys.public());

//     // Create the behaviour
//     let behaviour = Behaviour::new(local_peer_id.clone()).expect("Failed to create behaviour");

//     // Create the swarm
//     let swarm = SwarmBuilder::new(behaviour, local_peer_id.clone(), identity_keys)
//         .executor(Box::new(|fut| {
//             tokio::spawn(fut);
//         }))
//         .build();

//     // Start listening on a random TCP port
//     let listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().expect("Failed to parse listen address");
//     Swarm::listen_on(&swarm, listen_addr).expect("Failed to listen on address");

//     // Start the main event loop
//     tokio::run(futures::future::poll_fn(move || {
//         loop {
//             match swarm.poll().expect("Failed to poll swarm") {
//                 Async::Ready(Some(event)) => {
//                     match event {
//                         // Handle floodsub events
//                         BehaviourEvent::Floodsub(FloodsubEvent::Message { message, .. }) => {
//                             println!("Received message: {:?}", String::from_utf8_lossy(&message.data));
//                         }
//                         _ => {}
//                     }
//                 }
//                 Async::Ready(None) | Async::NotReady => break,
//             }
//         }
//         Ok(Async::NotReady)
//     }));
// }