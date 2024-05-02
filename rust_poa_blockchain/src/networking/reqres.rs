use libp2p;
use async_std;

struct DMProtocol();

impl libp2p::request_response::ProtocolName for DMProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/DMProtocol/1.0.0".as_bytes()
    }
}




struct DMCodec();

#[async_trait]
impl RequestResponseCodec for DMCodec {
    type Protocol = DMProtocol;
    type Request = String;
    type Response = String;

    async fn read_request(&mut self, io: &mut impl async_std::io::ReadExt) -> std::io::Result<Self::Request> {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(String::from_utf8(buf).unwrap())
    }

    async fn read_response(&mut self, io: &mut impl async_std::io::ReadExt) -> std::io::Result<Self::Response> {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(String::from_utf8(buf).unwrap())
    }

    async fn write_request(&mut self, io: &mut impl async_std::io::WriteExt, request: Self::Request) -> std::io::Result<()> {
        io.write_all(request.as_bytes()).await
    }

    async fn write_response(&mut self, io: &mut impl async_std::io::WriteExt, response: Self::Response) -> std::io::Result<()> {
        io.write_all(response.as_bytes()).await
    }
}