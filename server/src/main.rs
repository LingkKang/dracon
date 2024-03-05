//! The binary target of dracon
//!
//! Used local library are [`logger`] and [`raft`].

use logger::Logger;

use raft::node::Node;

use std::collections::HashSet;
use std::net::{SocketAddr, SocketAddrV4};

fn main() {
    Logger::init(Some(log::LevelFilter::Trace));

    // The initial pool of IP addresses for all starting nodes.
    let mut ip_pool: HashSet<SocketAddrV4> = HashSet::from([
        "172.19.0.2:16".parse().unwrap(),
        "172.19.0.3:16".parse().unwrap(),
        "172.19.0.4:16".parse().unwrap(),
    ]);

    let mut local_ip: Option<SocketAddrV4> = None;

    let args: Vec<String> = std::env::args().collect();

    for arg in args {
        let current_socket = arg.parse::<SocketAddrV4>();
        if current_socket.is_ok() {
            let current_socket = current_socket.unwrap();
            if ip_pool.contains(&current_socket) {
                ip_pool.remove(&current_socket);
                local_ip = Some(current_socket);
            }
        }
    }

    let local_ip = local_ip.expect("No IP address provided");

    log::info!("Machine started with IP: {}", local_ip);

    let node = Node::new(SocketAddr::from(local_ip));
    node.timeout();
    println!("Hello, world from Node {}", node.socket_addr());
}
