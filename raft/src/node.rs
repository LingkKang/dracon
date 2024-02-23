//! In Raft, each server is represented as a node [`Node`].

/// A Raft node that represents a server in the cluster.
pub struct Node {
    /// The current status of the node.
    status: Status,

    /// The current term of the node.
    /// Initialized to `0` on first boot, and increases monotonically.
    current_term: u128,
}

/// By default, a new node is created with [`Status::Follower`] status.
impl Default for Node {
    fn default() -> Self {
        Node::new()
    }
}

impl Node {
    /// Create a new node with [`Status::Follower`] status.
    pub fn new() -> Self {
        Node {
            status: Status::Follower,
            current_term: 0,
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

    /// Check if the node is a leader.
    pub fn is_leader(&self) -> bool {
        self.status.is_leader()
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

    /// Make sure that a new node is created with:
    /// - Status::Follower
    /// - Term 0
    #[test]
    fn test_new() {
        let node = Node::new();
        assert!(matches!(node.status(), Status::Follower));
        assert_eq!(node.current_term(), 0);
    }

    #[test]
    fn test_default() {
        let node = Node::default();
        assert!(matches!(node.status(), Status::Follower));
        assert_eq!(node.current_term(), 0);
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
