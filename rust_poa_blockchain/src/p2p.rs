use blocks;

pub static KEYS: Lazy = Lazy::new(identity::Keypair::generate_ed25519);
pub static PEER_ID: Lazy = Lazy::new(|| PeerId::from(KEYS.public()));
pub static CHAIN_TOPIC: Lazy = Lazy::new(|| Topic::new("chains"));
pub static BLOCK_TOPIC: Lazy = Lazy::new(|| Topic::new("blocks"));

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainResponse {
        pub blocks: Vec,
        pub receiver: String,
}

#[derive (Debug, Serialize, Deserialize)]
pub struct LocalChainRequest {
    pub from_peer_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    LocalChainResponse(ChainResponse),
    Input(String),
    Init,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppBehaviour {
    pub floodsub: Floodsub,
    pub mdns: Mdns,
    #[Behaviour(ignore)]
    pub response_sender: mpsc::UnboundedSender,
    #[Behaviour(ignore)]
    pub init_sender: mpsc::UnboundedSender,
    #[Behaviour(ignore)]
    pub blockchain: blocks::Blockchain,

}


impl AppBehaviour {
    pub async fn new(
        blockchain: blocks::Blockchain,
        response_sender: mpsc::UnboundedSender,
        init_sender: mpsc::UnboundedSender,
    ) -> self {
        let mut behaviour = Self {
            blockchain,
            floodsub: Floodsub::new(*PEER_ID),
            mdns: Mdns::new(Default::default())
                .await
                .expect("can create mdns"),
            response_sender,
            init_sender,
        };
        behaviour.floodsub.subscribe(CHAIN_TOPIC.clone());
        behaviour.floodsub.subscribe(BLOCK_TOPIC.clone());

        behaviour
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for AppBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(discovered_list) => {
                for (peer, _addr) in discovered_list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(expired_list) => {
                for (peer, _addr) in expired_list {
                    self.floodsub.remove_node_from_partial_view(&peer)
                }
            }
        }
    }
}


impl NetworkBehaviourEventProcess<FloodsubEvent> for AppBehaviour {
    fn inject_event(&mut self, event: FloodsubEvent) {
        match event {
            FloodsubEvent::Message(msg) => {
                if let Ok(resp) = serde_json::from_slice::<ChainResponse>(&msg.data) {
                    if resp.receiver == PEER_ID.to_string() {
                        info!("Response from {}:", msg.source);
                        resp.blocks.iter().for_each(|r| info!("{:?}", r));

                        self.blockchain.chain = self.blockchain.choose_chain(self.blockchain.chain.clone(), resp.blocks);
                    }
                } else if let Ok(resp) = serde_json::from_slice::<LocalChainRequest>(&msg.data) {
                    info!("sending local chain to {}", msg.source.to_string());
                    let peer_id = resp.from_peer_id;
                    if PEER_ID.to_string() == peer_id {
                        if let Err(e) = self.response_sender.send(ChainResponse {
                            blocks: self.blockchain.chain.clone(),
                            receiver: msg.source.to_string(),
                        }) {
                            error!("error sending response via channel, {}", e);
                        }
                    }
                } else if let Ok(block) = serde_json::from_slice::<Block>(&msg.data) {
                    info!("received new block from {}", msg.source.to_string());
                    self.blockchain.try_add_block(block);
                }
            }
            _ => {}
        }
    }
}