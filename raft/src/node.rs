//! In Raft, each server is represented as a node [`Node`].

pub struct Node {
    status: Status,
}

/// All possible status (states) of a Raft node.
///
/// As the word **State** have the same wording with *persistent* and
/// *volatile state* in Node's data, we use **Status** instead.
pub enum Status {
    /// Follower status.
    FLR,

    /// Candidate status.
    CDT,

    /// Leader status.
    LDR,
}

impl Status {
    pub fn is_leader(&self) -> bool {
        match self {
            Status::LDR => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_leader() {
        let leader = Status::LDR;
        assert!(leader.is_leader());

        let follower = Status::FLR;
        assert!(!follower.is_leader());

        let candidate = Status::CDT;
        assert!(!candidate.is_leader());
    }
}
