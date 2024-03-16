use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub type Byte = u8;
pub type Bytes = Vec<Byte>;
pub type Result<T> = std::result::Result<T, std::io::Error>;

pub trait RpcRequest {
    fn serialize(&self) -> Bytes;
    fn deserialize(data: Bytes) -> Self;
    fn clone(&self) -> Self;
    fn to_string(&self) -> String;
}

pub trait RpcResponse {
    fn serialize(&self) -> Bytes;
    fn deserialize(data: Bytes) -> Self;
    fn clone(&self) -> Self;
    fn to_string(&self) -> String;
}

#[derive(Clone)]
pub struct Service<Req, Res>
where
    Req: RpcRequest,
    Res: RpcResponse,
{
    socket: SocketAddr,
    request: Req,
    response: Res,
}

impl<Req, Res> Service<Req, Res>
where
    Req: RpcRequest,
    Res: RpcResponse,
{
    pub fn new(socket: SocketAddr, request: Req, response: Res) -> Self {
        Service {
            socket,
            request,
            response,
        }
    }

    pub async fn send_request(&self, target: SocketAddr) -> Result<Res> {
        let mut stream = tokio::net::TcpStream::connect(target).await?;
        log::trace!("Connected to server: {:?}", target);

        stream.write_all(&self.request.serialize()).await?;
        log::debug!(
            "Sent request [{}] to {}",
            self.request.to_string(),
            target
        );

        let mut buffer: Bytes = vec![0; 1024];
        let n = stream.read(&mut buffer).await?;
        let response = Res::deserialize(buffer[..n].to_vec());
        log::info!(
            "Received response [{}] from {}",
            response.to_string(),
            target
        );

        Ok(response)
    }

    pub async fn handle_request(&self) -> Result<()> {
        let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind(self.socket).await?;
        log::trace!("Listening on: {:?}", self.socket);
        loop {
            let (mut stream, addr) = listener.accept().await?;
            log::trace!("Accepted connection from: {:?}", addr);

            let mut buffer: Bytes = vec![0; 1024];
            let n = stream.read(&mut buffer).await?;
            let request = Req::deserialize(buffer[..n].to_vec());
            log::debug!("Received request [{}] from {}", request.to_string(), addr);

            let response = self.response.clone();
            stream.write_all(&response.serialize()).await?;
            log::info!("Sent response: {} to {}", response.to_string(), addr);
        }
    }
}

#[derive(Clone)]
pub struct PingRequest;

impl RpcRequest for PingRequest {
    fn serialize(&self) -> Bytes {
        vec![0]
    }

    #[allow(unused_variables)]
    fn deserialize(data: Bytes) -> Self {
        PingRequest
    }

    fn clone(&self) -> Self {
        PingRequest
    }

    fn to_string(&self) -> String {
        "PingRequest".to_string()
    }
}

#[derive(Clone)]
pub struct PingResponse;

impl RpcResponse for PingResponse {
    fn serialize(&self) -> Bytes {
        vec![0]
    }

    #[allow(unused_variables)]
    fn deserialize(data: Bytes) -> Self {
        PingResponse
    }

    fn clone(&self) -> Self {
        PingResponse
    }

    fn to_string(&self) -> String {
        "PingResponse".to_string()
    }
}

pub type PingService = Service<PingRequest, PingResponse>;
