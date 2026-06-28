use crate::{
    state_machine::Raft,
    states::{Candidate, Follower, Leader},
};

/**
 * Follower -> Candidate
 * Candidate -> Leader
 * Candidate -> Follower
 * Leader -> Follower
 */

pub enum RaftNode {
    Follower(Raft<Follower>),
    Candidate(Raft<Candidate>),
    Leader(Raft<Leader>),
}

pub enum Transition {
    To(RaftNode),
    Shutdown(String),
}

impl From<Raft<Follower>> for Raft<Candidate> {
    fn from(raft: Raft<Follower>) -> Self {
        let mut r = Raft {
            shared_state: raft.shared_state,
            node: Candidate::new(),
            to_server: raft.to_server,
            from_server: raft.from_server,
        };
        let term = r.increment_term();
        r.reset_election_timer();
        r.increment_votes();
        r.set_max_term_seen(term);

        r
    }
}

impl From<Raft<Candidate>> for Raft<Follower> {
    fn from(raft: Raft<Candidate>) -> Self {
        Raft {
            shared_state: raft.shared_state,
            node: Follower::new(),
            to_server: raft.to_server,
            from_server: raft.from_server,
        }
    }
}

impl From<Raft<Candidate>> for Raft<Leader> {
    fn from(raft: Raft<Candidate>) -> Self {
        Raft {
            shared_state: raft.shared_state,
            node: Leader::new(),
            to_server: raft.to_server,
            from_server: raft.from_server,
        }
    }
}

impl From<Raft<Leader>> for Raft<Follower> {
    fn from(raft: Raft<Leader>) -> Self {
        Raft {
            shared_state: raft.shared_state,
            node: Follower::new(),
            to_server: raft.to_server,
            from_server: raft.from_server,
        }
    }
}
