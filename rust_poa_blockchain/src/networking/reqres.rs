use libp2p;
use async_std;



#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GreetRequest {
    name: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GreetResponse {
    message: String,
}