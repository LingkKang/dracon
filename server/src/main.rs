//! The binary target of dracon.
//!
//! Local libraries used here are [`logger`] and [`raft`].

use logger::Logger;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use std::net::{SocketAddr, SocketAddrV4};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    Logger::init(Some(log::LevelFilter::Trace));

    #[rustfmt::skip]
    // The initial pool of sockets for all starting nodes.
    let mut sockets: std::collections::HashSet<SocketAddrV4> =
        std::collections::HashSet::from([
            "172.19.0.2:16", 
            "172.19.0.3:16", 
            "172.19.0.4:16"
            ].map(|socket| socket.parse().unwrap()),
        );

    let mut local_socket: Option<SocketAddrV4> = None;

    let args: Vec<String> = std::env::args().collect();

    // Remove local socket from the sockets pool.
    for arg in args {
        let current_socket = arg.parse::<SocketAddrV4>();
        if current_socket.is_ok() {
            let current_socket = current_socket.unwrap();
            if sockets.contains(&current_socket) {
                sockets.remove(&current_socket);
                local_socket = Some(current_socket);
            }
        }
    }

    let local_socket = local_socket.unwrap_or_else(|| {
        log::error!("No socket address provided");
        std::process::exit(0);
    });

    log::info!("Machine started with socket: {}", local_socket);

    // let node = Node::new(SocketAddr::from(local_socket));
    // node.timeout();

    let listener = tokio::net::TcpListener::bind(local_socket).await.unwrap();
    log::debug!("Listening on {}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        loop {
            let (coming_stream, coming_socket) =
                listener.accept().await.unwrap();
            tokio::spawn(handle_ping(coming_stream, coming_socket));
        }
    });

    let wait_time = 10;
    log::info!("Waiting for {wait_time} seconds to send pings to other nodes");
    tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)).await;

    for socket in sockets {
        // Send pings to other nodes.
        tokio::spawn(send_ping(SocketAddr::from(socket)));
    }

    // wait to make sure other spawned tasks are done.
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    std::process::exit(0);
}

async fn handle_ping(mut stream: tokio::net::TcpStream, socket: SocketAddr) {
    log::debug!("New connection from {}", socket);
    let mut buffer = [0; 64];
    loop {
        let n = match stream.read(&mut buffer).await {
            Ok(n) if n == 0 => {
                log::debug!("Connection closed");
                break;
            }
            Ok(n) => n,
            Err(e) => {
                log::error!("Failed to read from socket; err = {:?}", e);
                break;
            }
        };

        let message = String::from_utf8_lossy(&buffer[..n]);
        let message = message.trim(); // Remove the trailing newline.
        log::info!("Received '{}' from {}", message, socket);

        // Respond to the ping.
        if message == "ping" {
            let response = b"pong";
            if let Err(e) = stream.write_all(response).await {
                log::error!("Failed to write to socket; err = {:?}", e);
                break;
            }
        }
    }
}

async fn send_ping(socket: SocketAddr) {
    log::debug!("Sending ping to {}", socket);
    if let Ok(mut stream) = tokio::net::TcpStream::connect(&socket).await {
        if let Err(e) = stream.write_all(b"ping\n").await {
            log::error!("Failed to write to socket; err = {:?}", e);
            // break;
        }
        let mut buffer = [0; 64];
        loop {
            let n = match stream.read(&mut buffer).await {
                Ok(n) if n == 0 => {
                    log::debug!("Connection closed");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    log::error!("Failed to read from socket; err = {:?}", e);
                    break;
                }
            };

            let message = String::from_utf8_lossy(&buffer[..n]);
            let message = message.trim(); // Remove the trailing newline.
            log::info!("Get response '{}' from {}", message, socket);
            break;
        }
        // Close the connection.
        drop(stream);
    }
    log::debug!("Sending ping to {}", socket);
}
