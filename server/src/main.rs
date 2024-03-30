//! The binary target of dracon.
//!
//! Local libraries used here are [`logger`], [`raft`] and [`rpc`].
//!
//! For the configuration file formatting, see [`read_config()`] function.

use logger::Logger;
use raft::node::Node;

use std::net::SocketAddr;

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

    // Initialize the logger with the local socket address as the prefix.
    Logger::new().set_prefix(local_socket.to_string()).init();
    log::info!("Machine started with socket: {}", local_socket);

    // Convert the sockets into a hash set.
    // 1. Duplicates will be removed if any.
    // 2. The order of the sockets is not important.
    let sockets: std::collections::HashSet<SocketAddr> =
        std::collections::HashSet::from_iter(sockets);

    let mut node = Node::new(local_socket);
    node.append_peers(sockets);

    node.start().await;

    log::info!("Exiting");
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
