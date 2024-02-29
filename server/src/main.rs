use logger::Logger;
use raft::node::Node;
use std::collections::HashSet;

fn main() {
    Logger::init(Some(log::LevelFilter::Trace));

    #[rustfmt::skip]
    // The initial pool of IP addresses for all starting nodes.
    let mut ip_pool: HashSet<String> = HashSet::from([
        "172.19.0.2/16",
        "172.19.0.3/16",
        "172.19.0.4/16"
    ].map(String::from));

    let mut local_ip: Option<String> = None;

    let args: Vec<String> = std::env::args().collect();
    for arg in args {
        if ip_pool.contains(&arg) {
            ip_pool.remove(&arg);
            local_ip = Some(arg);
        }
    }

    let local_ip = local_ip.expect("No IP address provided");

    log::info!("Machine started with IP: {}", local_ip);

    let node = Node::new(local_ip);
    node.timeout();
    println!("Hello, world from Node {}", node.local_ip());
}
