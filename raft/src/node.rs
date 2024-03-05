//! In Raft, each server is represented as a node [`Node`].

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
}

impl Node {
    /// Create a new node with [`Status::Follower`] status.
    pub fn new(socket_addr: SocketAddr) -> Self {
        log::debug!("Creating Node with IP: {}", socket_addr);
        let mut rng = rand::thread_rng();
        Node {
            status: Status::Follower,
            current_term: 0,
            socket_addr,
            election_timeout: rng.gen_range(150..=300),
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

    pub fn timeout(&self) {
        log::debug!(
            "Initial election timeout for Node: {} in {} ms.",
            self.socket_addr,
            self.election_timeout
        );
        std::thread::sleep(std::time::Duration::from_millis(
            self.election_timeout as u64,
        ));
        log::trace!("Node: {} timed out.", self.socket_addr);
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
}
