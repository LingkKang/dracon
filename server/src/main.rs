//! The binary target of dracon.
//!
//! Local libraries used here are [`logger`], [`raft`] and [`rpc`].
//!
//! For the configuration file formatting, see [`read_config()`] function.

use logger::Logger;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // The initial pool of sockets for all starting nodes.
    let sockets = read_config("./tmp/config.txt");
    let mut sockets = match sockets {
        Ok(sockets) => sockets,
        Err(e) => {
            println!("Error reading config file: {:?}", e);
            std::process::exit(1);
        }
    };

    // The first argument is the local socket address.
    let local_socket = sockets.swap_remove(0);

    Logger::new().set_prefix(local_socket.to_string()).init();

    log::info!("Machine started with socket: {}", local_socket);

    // The server should listen on the 0.0.0.0 address,
    // with the same port as the local socket.
    // NOT 127.0.0.1 or LOCALHOST.
    let listen_socket =
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), local_socket.port());

    let service = rpc::Service::new(
        listen_socket,
        rpc::PingRequest::new("Ping".to_string()),
        rpc::PingResponse::new("Pong".to_string()),
    );

    let srv = service.clone();
    let task = tokio::spawn(async move { srv.handle_request().await });

    let wait_time = 10;
    log::info!("Waiting for {wait_time} seconds to send pings to other nodes");
    tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)).await;

    for socket in sockets {
        let srv = service.clone();
        tokio::spawn(async move { srv.send_request(socket).await });
    }

    // wait to make sure other spawned tasks are done.
    match tokio::time::timeout(tokio::time::Duration::from_secs(30), task).await
    {
        Ok(Ok(_)) => log::debug!("Task completed"),
        Ok(Err(e)) => log::error!("Task failed due to error: {:?}", e),
        Err(e) => log::info!("Task closed due to timeout: {:?}", e),
    }
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
///
/// Note that the first line denoted as the local socket address.
fn read_config<P>(path: P) -> std::io::Result<Vec<SocketAddr>>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut sockets = Vec::new();
    for line in std::io::BufRead::lines(reader) {
        let line = line?;
        if let Ok(socket) = line.parse::<SocketAddr>() {
            sockets.push(socket);
        }
    }
    let len = sockets.len();
    assert!(len >= 3, "Need at least 3 nodes to start, found {} nodes", len);
    Ok(sockets)
}
