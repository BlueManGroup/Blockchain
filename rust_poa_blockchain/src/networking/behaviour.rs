use libp2p::{
    floodsub::Floodsub,
    swarm::NetworkBehaviour,
    PeerId, identify, 
    mdns, request_response,
    StreamProtocol
};
use libp2p_identity::PublicKey;
use crate::networking::reqres;
use crate::networking::behaviour::request_response::ProtocolSupport;

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub floodsub: Floodsub,
    pub identify: identify::Behaviour,
    pub mdns: mdns::tokio::Behaviour, //Implementation is done, but we're using predefined tables for known nodes
    pub reqres: request_response::json::Behaviour::<reqres::GreetRequest, reqres::GreetResponse>,
} 

impl Behaviour {
    pub fn new(local_public_key: PublicKey, local_peer_id: PeerId) -> Result<Self, Box<dyn std::error::Error>> {
        let floodsub = Floodsub::new(local_peer_id.clone());
        let identify = identify::Behaviour::new(identify::Config::new("1.0".into(), local_public_key));
        let mdns_config = mdns::Config::default();
        let mdns = mdns::tokio::Behaviour::new(mdns_config, local_peer_id)?;
        let reqres = request_response::json::Behaviour::<reqres::GreetRequest, reqres::GreetResponse>::new(
            [(StreamProtocol::new("/JsonDirectMessage"), ProtocolSupport::Full)],
            request_response::Config::default());
        Ok(Self {identify, floodsub,mdns, reqres}) //with mdns
        // Ok(Self {identify, floodsub, reqres}) //without mdns
    }
}