//! # RPC (Remote Procedure Call)
//!
//! A simple RPC implementation.
//!
//! ## Example: Ping
//!
//! Structs [`PingRequest`] and [`PingResponse`] are defined as examples to
//! demonstrate the usage of the RPC library. They implement the [`RpcRequest`]
//! and [`RpcResponse`] traits respectively.
//!
//! This example demonstrates the basic usage of the RPC library
//! by sending a ping request to a set of nodes and waiting for a response.
//!
//! Start some docker containers with the internal docker network attached.
//! And run the following command to start the example:
//!
//! ``` shell
//! cargo run --example ping 172.19.0.2:16
//! ```
//!
//! Note that the IP address and port number are the address of the
//! current node.
//!
//! Or simply run the PowerShell script which handles everything for you:
//!
//! ``` PowerShell
//! .\examples\ping.ps1
//! ```
//!
//! A typical output of a node would be:
//! ``` Log
//! 2024-03-16_15:24:28.099977Z 172.19.0.3:16 TRACE Logger initialized with default level: Trace
//! 2024-03-16_15:24:28.100066Z 172.19.0.3:16 INFO Machine started with socket: 172.19.0.3:16
//! 2024-03-16_15:24:28.100125Z 172.19.0.3:16 INFO Sleeping for 2 seconds to wait for peers to start
//! 2024-03-16_15:24:28.100347Z 172.19.0.3:16 TRACE Listening on 172.19.0.3:16
//! 2024-03-16_15:24:29.706205Z 172.19.0.3:16 TRACE Accepted connection from 172.19.0.2:35496
//! 2024-03-16_15:24:29.706807Z 172.19.0.3:16 DEBUG Received request [Ping] from 172.19.0.2:35496
//! 2024-03-16_15:24:29.707107Z 172.19.0.3:16 INFO Sent response [Pong] to 172.19.0.2:35496
//! 2024-03-16_15:24:30.107307Z 172.19.0.3:16 TRACE Connected to 172.19.0.2:16
//! 2024-03-16_15:24:30.107519Z 172.19.0.3:16 DEBUG Sent request [Ping] to 172.19.0.2:16
//! 2024-03-16_15:24:30.107591Z 172.19.0.3:16 TRACE Connected to 172.19.0.4:16
//! 2024-03-16_15:24:30.107689Z 172.19.0.3:16 DEBUG Sent request [Ping] to 172.19.0.4:16
//! 2024-03-16_15:24:30.107765Z 172.19.0.3:16 INFO Received response [Pong] from 172.19.0.2:16
//! 2024-03-16_15:24:30.107845Z 172.19.0.3:16 INFO Received response [Pong] from 172.19.0.4:16
//! 2024-03-16_15:24:30.518019Z 172.19.0.3:16 TRACE Accepted connection from 172.19.0.4:39230
//! 2024-03-16_15:24:30.518094Z 172.19.0.3:16 DEBUG Received request [Ping] from 172.19.0.4:39230
//! 2024-03-16_15:24:30.518186Z 172.19.0.3:16 INFO Sent response [Pong] to 172.19.0.4:39230
//! 2024-03-16_15:25:00.094457Z 172.19.0.3:16 INFO Task closed due to timeout: Elapsed(())
//! ```
//!
//! See the example `ping` at `./examples/ping` for a complete example.

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
        Service { socket, request, response }
    }

    /// Send a request to the service.
    pub async fn send_request(&self, target: SocketAddr) -> Result<Res> {
        let mut stream = tokio::net::TcpStream::connect(target).await?;
        log::trace!("Connected to {:?}", target);

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
        let listener: tokio::net::TcpListener =
            tokio::net::TcpListener::bind(self.socket).await?;
        log::trace!("Listening on {:?}", self.socket);

        loop {
            let (mut stream, addr) = listener.accept().await?;
            log::trace!("Accepted connection from {:?}", addr);

            let mut buffer: Bytes = vec![0; 1024];
            let n = stream.read(&mut buffer).await?;
            let request = Req::deserialize(buffer[..n].to_vec());
            log::debug!(
                "Received request [{}] from {}",
                request.to_string(),
                addr
            );

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
            Err(_) => PingRequest { data: "Failed to parse data".to_string() },
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
            Err(_) => PingResponse { data: "Failed to parse data".to_string() },
        }
    }

    fn to_string(&self) -> String {
        self.data.clone()
    }
}

pub type PingService = Service<PingRequest, PingResponse>;
