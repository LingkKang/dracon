use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub type Byte = u8;
pub type Bytes = Vec<Byte>;
pub type Result<T> = std::result::Result<T, std::io::Error>;

/// Trait for RPC request.
pub trait RpcRequest {
    /// Serialize the request into [`Bytes`].
    fn serialize(&self) -> Bytes;

    /// Deserialize the request from [`Bytes`].
    fn deserialize(data: Bytes) -> Self;

    /// Convert the request into a [`String`].
    fn to_string(&self) -> String;
}

/// Trait for RPC response.
pub trait RpcResponse {
    /// Serialize the response into [`Bytes`].
    fn serialize(&self) -> Bytes;

    /// Deserialize the response from [`Bytes`].
    fn deserialize(data: Bytes) -> Self;

    /// Convert the response into a [`String`].
    fn to_string(&self) -> String;
}

/// RPC service.
/// 
/// Mainly used for sending requests and handling requests.
#[derive(Clone)]
pub struct Service<Req, Res>
where
    Req: RpcRequest,
    Res: RpcResponse,
{
    /// Socket address of the service.
    socket: SocketAddr,

    /// Request type.
    request: Req,

    /// Response type.
    response: Res,
}

impl<Req, Res> Service<Req, Res>
where
    Req: RpcRequest,
    Res: RpcResponse,
{
    /// Create a new service.
    pub fn new(socket: SocketAddr, request: Req, response: Res) -> Self {
        Service {
            socket,
            request,
            response,
        }
    }

    /// Send a request to the service.
    pub async fn send_request(&self, target: SocketAddr) -> Result<Res> {
        let mut stream = tokio::net::TcpStream::connect(target).await?;
        log::trace!("Connected to {:?}", target);

        stream.write_all(&self.request.serialize()).await?;
        log::debug!("Sent request [{}] to {}", self.request.to_string(), target);

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
        log::trace!("Listening on {:?}", self.socket);

        loop {
            let (mut stream, addr) = listener.accept().await?;
            log::trace!("Accepted connection from {:?}", addr);

            let mut buffer: Bytes = vec![0; 1024];
            let n = stream.read(&mut buffer).await?;
            let request = Req::deserialize(buffer[..n].to_vec());
            log::debug!("Received request [{}] from {}", request.to_string(), addr);

            let response_msg = self.response.to_string();
            stream.write_all(&self.response.serialize()).await?;
            log::info!("Sent response [{}] to {}", response_msg, addr);
        }
    }
}

#[derive(Clone)]
pub struct PingRequest {
    data: String,
}

impl PingRequest {
    pub fn new(data: String) -> Self {
        PingRequest { data }
    }
}

impl RpcRequest for PingRequest {
    fn serialize(&self) -> Bytes {
        self.data.as_bytes().to_vec()
    }

    #[allow(unused_variables)]
    fn deserialize(data: Bytes) -> Self {
        let data = data.to_vec();
        match String::from_utf8(data) {
            Ok(data) => {
                let data = data.trim().to_string();
                PingRequest { data }
            }
            Err(_) => PingRequest {
                data: "Failed to parse data".to_string(),
            },
        }
    }

    fn to_string(&self) -> String {
        self.data.clone()
    }
}

#[derive(Clone)]
pub struct PingResponse {
    data: String,
}

impl PingResponse {
    pub fn new(data: String) -> Self {
        PingResponse { data }
    }
}

impl RpcResponse for PingResponse {
    fn serialize(&self) -> Bytes {
        self.data.as_bytes().to_vec()
    }

    #[allow(unused_variables)]
    fn deserialize(data: Bytes) -> Self {
        let data = data.to_vec();
        match String::from_utf8(data) {
            Ok(data) => {
                let data = data.trim().to_string();
                PingResponse { data }
            }
            Err(_) => PingResponse {
                data: "Failed to parse data".to_string(),
            },
        }
    }

    fn to_string(&self) -> String {
        self.data.clone()
    }
}

pub type PingService = Service<PingRequest, PingResponse>;
