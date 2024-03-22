//! The binary target of dracon.
//!
//! Local libraries used here are [`logger`], [`raft`] and [`rpc`].

use logger::Logger;

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

    let service = rpc::Service::new(
        test_socket,
        rpc::PingRequest::new("Ping".to_string()),
        rpc::PingResponse::new("Pong".to_string()),
    );
    let srv = service.clone();

    let task = tokio::spawn(async move { srv.handle_request().await });

    let wait_time = 10;
    log::info!("Waiting for {wait_time} seconds to send pings to other nodes");
    tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)).await;

    for socket in sockets {
        log::trace!("Sending request to {:?}", socket);
        let srv = service.clone();
        tokio::spawn(async move { srv.send_request(socket).await });
    }

    log::trace!("All requests sent and waiting for timeout...");

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
