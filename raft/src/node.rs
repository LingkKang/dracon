//! In Raft, each server is represented as a node [`Node`].

pub struct Node {
    status: Status,
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
        }
    }

    /// Get the current status of the node.
    pub fn status(&self) -> &Status {
        &self.status
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

    #[test]
    // Make sure that a new node is created with Follower status.
    fn test_new() {
        let node = Node::new();
        assert!(matches!(node.status(), Status::Follower));
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
