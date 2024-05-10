use libp2p;
use async_std;



#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GreetRequest {
    pub message: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GreetResponse {
    pub message: String,
}