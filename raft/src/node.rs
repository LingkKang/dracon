//! In Raft, each server is represented as a node [`Node`].

use std::collections::HashSet;
use std::net::SocketAddr;

use rand::Rng;

/// A Raft node that represents a server in the cluster.
pub struct Node {
    /// The current status of the node.
    status: Status,

    /// The current term of the node.
    /// Initialized to `0` on first boot, and increases monotonically.
    current_term: u128,

    /// A socket address is the composition of
    ///
    /// 1. IP address (either IPv4 or IPv6)
    /// 2. port number in [`u16`]
    ///
    /// [`SocketAddr`] is an enum of [`SocketAddr::V4`] and [`SocketAddr::V6`]
    socket_addr: SocketAddr,

    /// Election timeout in milliseconds.
    /// Typically 150 - 300 ms.
    election_timeout: u16,

    /// The list of peers in the cluster.
    /// Use [`HashSet`] to avoid duplicates.
    peers: HashSet<SocketAddr>,
}

impl Node {
    /// Create a new node with [`Status::Follower`] status.
    pub fn new(socket_addr: SocketAddr) -> Self {
        log::debug!("Node created");
        Node {
            status: Status::Follower,
            current_term: 0,
            socket_addr,
            election_timeout: crate::next_timeout(),
            peers: HashSet::new(),
        }
    }

    /// Get the current status of the node.
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Get the current term of the node.
    pub fn current_term(&self) -> u128 {
        self.current_term
    }

    /// Get the IP address of the node.
    pub fn socket_addr(&self) -> &SocketAddr {
        &self.socket_addr
    }

    /// Check if the node is a leader.
    pub fn is_leader(&self) -> bool {
        self.status.is_leader()
    }

    pub fn refresh_timeout(&mut self) {
        self.election_timeout = crate::next_timeout();
    }

    /// Append a list of peers to the node.
    pub fn append_peers(&mut self, new_peers: HashSet<SocketAddr>) {
        self.peers.extend(new_peers);
    }

    pub async fn start(&mut self) {
        log::info!("Node started");
        self.initial_election().await;
    }

    /// Initial election.
    async fn initial_election(&mut self) {
        self.status = Status::Candidate;
        self.current_term += 1;

        let (canceller, handle) = crate::timeout(self.election_timeout).await;

        if rand::thread_rng().gen_bool(0.5) {
            canceller.send(()).unwrap();
            log::debug!("Should cancel timeout");
        } else {
            log::debug!("Should not cancel timeout");
        }
        handle.await.unwrap();
        self.refresh_timeout();

        if self.status == Status::Candidate {
            self.status = Status::Follower;
        }
    }
}

/// All possible status (states) of a Raft node.
///
/// As the word **State** have the same wording with *persistent* and
/// *volatile state* in Node's data, we use **Status** instead.
#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    /// Follower status. The default status of a new node.
    Follower,

    /// Candidate status.
    Candidate,

    /// Leader status.
    Leader,
}

impl Status {
    pub fn is_leader(&self) -> bool {
        matches!(self, Status::Leader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static LOCAL_ADDR: &str = "127.0.0.0:2024";

    /// Make sure that a new node is created with:
    /// - Status::Follower
    /// - Term 0
    #[test]
    fn test_new() {
        let node = Node::new(LOCAL_ADDR.parse().unwrap());
        assert!(matches!(node.status(), Status::Follower));
        assert_eq!(node.current_term(), 0);
    }

    #[test]
    fn test_socket() {
        let node = Node::new(LOCAL_ADDR.parse().unwrap());
        assert!(node.socket_addr().ip().is_loopback());
        assert_eq!(node.socket_addr().port(), 2024);
    }

    #[test]
    fn test_is_leader() {
        let leader = Status::Leader;
        assert!(leader.is_leader());

        let follower = Status::Follower;
        assert!(!follower.is_leader());

        let candidate = Status::Candidate;
        assert!(!candidate.is_leader());
    }

    #[test]
    fn test_peers() {
        let node = Node::new(LOCAL_ADDR.parse().unwrap());
        assert!(node.peers.is_empty());
    }

    #[test]
    fn test_append_peers() {
        let mut node = Node::new(LOCAL_ADDR.parse().unwrap());
        let peers: HashSet<SocketAddr> = HashSet::from(
            ["127.0.0.2:16", "127.0.0.3:16", "127.0.0.4:16", "127.0.0.5:16"]
                .map(|socket| socket.parse().unwrap()),
        );
        node.append_peers(peers.clone());
        assert_eq!(node.peers, peers);
    }
}
