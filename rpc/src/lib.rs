use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Request {
    message: String,
}

impl Request {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }

    pub fn get_serialized_message(&self) -> Vec<u8> {
        let mut msg = self.message.clone().into_bytes();
        msg.push(0);
        msg
    }
}

pub struct Response {
    message: String,
}

impl Response {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

#[derive(Clone)]
pub struct Service {
    provider: SocketAddr,
}

impl Service {
    pub fn new(provider: SocketAddr) -> Self {
        Self { provider }
    }

    pub async fn handle_request(&self, request: Request, response: Response) {
        let listener = tokio::net::TcpListener::bind(&self.provider).await;
        match listener {
            Ok(_) => log::info!("Listening on [{}]", self.provider),
            Err(e) => {
                log::error!("Failed to bind to socket: {}", e);
                return;
            }
        }
        let listener = listener.unwrap();

        let (mut stream, addr) = listener.accept().await.unwrap();
        log::debug!("Accepted connection from [{}]", addr);
        let mut buffer = Vec::new();
        match stream.read_to_end(&mut buffer).await {
            Ok(_) => {
                let req = String::from_utf8(buffer).unwrap();
                let req = req.trim();
                log::info!("Received request [{}] from [{}]", req, addr);
                if req == request.message {
                    let res = response.message.as_bytes();
                    if let Err(e) = stream.write_all(res).await {
                        log::error!("Failed to write to socket: {}", e);
                    }
                } else {
                    log::warn!("Received invalid request [{}] from [{}]", req, addr)
                }
            }
            Err(e) => log::error!("Failed to read from socket: {}", e),
        }
    }

    pub async fn send_request(&self, request: Request, target_addr: SocketAddr) {
        let stream = tokio::net::TcpStream::connect(target_addr).await;
        match stream {
            Ok(_) => log::debug!("Connected to [{}]", target_addr),
            Err(e) => {
                log::error!("Failed to connect to [{}]: {}", target_addr, e);
                return;
            }
        }
        let mut stream = stream.unwrap();

        let message = request.get_serialized_message();
        stream.write_all(&message).await.unwrap();
        log::debug!("Sent request [{}] to [{}]", request.message, target_addr);

        let mut buffer = Vec::new();
        match stream.read_to_end(&mut buffer).await {
            Ok(_) => {}
            Err(e) => {
                log::error!("Failed to read from socket: {}", e);
                return;
            }
        }
        let res = String::from_utf8(buffer).unwrap();
        log::info!("Received response [{}] from [{}]", res, target_addr);
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
