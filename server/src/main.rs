//! The binary target of dracon.
//!
//! Local libraries used here are [`logger`], [`raft`] and [`rpc`].

use logger::Logger;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let input_socket =
        std::env::args().nth(1).expect("No command argument provided");
    Logger::new().set_prefix(input_socket).init();

    #[rustfmt::skip]
    // The initial pool of sockets for all starting nodes.
    let mut sockets = read_config("./tmp/config.txt").unwrap();

    let mut local_socket: Option<SocketAddr> = None;

    let args: Vec<String> = std::env::args().collect();

    log::debug!("Arguments: {:?}", args);
    log::debug!("Config: {:?}", sockets);

    // Remove local socket from the sockets pool.
    for arg in args {
        let current_socket = arg.parse::<SocketAddr>();
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

    let test_ip = Ipv4Addr::LOCALHOST;
    let test_socket = SocketAddr::new(IpAddr::V4(test_ip), local_socket.port());

    let listener = tokio::net::TcpListener::bind(test_socket).await.unwrap();
    log::debug!("Listening on {}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        loop {
            let (coming_stream, coming_socket) =
                listener.accept().await.unwrap();
            tokio::spawn(handle_ping(coming_stream, coming_socket));
        }
    });

    let wait_time = 2;
    log::info!("Waiting for {wait_time} seconds to send pings to other nodes");
    tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)).await;

    for socket in sockets {
        // Send pings to other nodes.
        tokio::spawn(send_ping(socket));
    }

    // wait to make sure other spawned tasks are done.
    tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
    std::process::exit(0);
}

/// Read the configuration file and return a set of sockets.
///
/// The config file should contain a list of socket addresses, one per line.
///
/// For example:
///
/// ``` txt
/// 172.19.0.2:16
/// 172.19.0.3:16
/// 172.19.0.4:16
///
/// ```
fn read_config<P>(
    path: P,
) -> std::io::Result<std::collections::HashSet<SocketAddr>>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut sockets = std::collections::HashSet::new();
    for line in std::io::BufRead::lines(reader) {
        let line = line?;
        if let Ok(socket) = line.parse::<SocketAddr>() {
            sockets.insert(socket);
        }
    }
    Ok(sockets)
}

async fn handle_ping(mut stream: tokio::net::TcpStream, socket: SocketAddr) {
    log::debug!("New connection from {}", socket);
    let mut buffer = [0; 64];
    loop {
        let n = match stream.read(&mut buffer).await {
            Ok(0) => {
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
        }
        let mut buffer = [0; 64];
        match stream.read(&mut buffer).await {
            Ok(0) => {
                log::debug!("Connection closed");
            }
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]);
                let message = message.trim(); // Remove the trailing newline.
                log::info!("Received '{}' from {}", message, socket);
            }
            Err(e) => {
                log::error!("Failed to read from socket; err = {:?}", e);
            }
        }
        // Close the connection.
        drop(stream);
    } else {
        log::error!("Failed to connect to {}", socket);
    }
}
