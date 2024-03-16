use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub type Byte = u8;
pub type Bytes = Vec<Byte>;
pub type Result<T> = std::result::Result<T, std::io::Error>;

pub trait RpcRequest {
    fn serialize(&self) -> Bytes;
    fn deserialize(data: Bytes) -> Self;
    fn to_string(&self) -> String;
}

pub trait RpcResponse {
    fn serialize(&self) -> Bytes;
    fn deserialize(data: Bytes) -> Self;
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
