//! The binary target of dracon.
//!
//! Local libraries used here are [`logger`] and [`raft`].

use logger::Logger;

use raft::node::Node;

use std::collections::HashSet;
use std::net::{SocketAddr, SocketAddrV4};
use std::process::exit;

fn main() {
    Logger::init(Some(log::LevelFilter::Trace));

    #[rustfmt::skip]
    // The initial pool of sockets for all starting nodes.
    let mut sockets: HashSet<SocketAddrV4> = HashSet::from([
        "172.19.0.2:16",
        "172.19.0.3:16",
        "172.19.0.4:16",
    ].map(|socket| socket.parse().unwrap()));

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
        exit(0);
    });

    log::info!("Machine started with socket: {}", local_socket);

    let node = Node::new(SocketAddr::from(local_socket));
    node.timeout();
    println!("Hello, world from Node {}", node.socket_addr());
}
