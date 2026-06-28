use crate::{cluster::ClusterConfig, constants::ELECTION_TIMEOUT};

pub struct RaftSharedState {
    pub server_state: raft_models::state::ServerState,
    pub cluster_config: ClusterConfig,
}

pub struct Follower {
    last_heartbeat: std::time::Instant,
}

pub struct Candidate {
    last_heartbeat: std::time::Instant,
    election_state: ElectionState,
}

pub struct Leader {
    state: raft_models::state::LeaderState,
}

pub struct ElectionState {
    votes_granted: u64,
    max_term_seen: u64,
}

impl RaftSharedState {
    pub fn from_persisted_state(path: &str) -> Result<Self, String> {
        let server_state = raft_models::state::ServerState::from_persistent_state(path);
        let cluster_config = ClusterConfig::from_env()?;

        Ok(RaftSharedState {
            server_state,
            cluster_config,
        })
    }

    pub fn init() -> Self {
        RaftSharedState {
            server_state: raft_models::state::ServerState::default(),
            cluster_config: ClusterConfig::from_env().unwrap(),
        }
    }
}

impl Follower {
    pub fn new() -> Self {
        Follower {
            last_heartbeat: std::time::Instant::now(),
        }
    }

    pub fn election_timed_out(&self) -> bool {
        std::time::Instant::now().duration_since(self.last_heartbeat) > *ELECTION_TIMEOUT
    }

    pub fn reset_election_timer(&mut self) {
        self.last_heartbeat = std::time::Instant::now();
    }
}

impl Candidate {
    pub fn new() -> Self {
        Candidate {
            last_heartbeat: std::time::Instant::now(),
            election_state: ElectionState {
                votes_granted: 0,
                max_term_seen: 0,
            }
        }
    }

    pub fn increment_votes(&mut self) {
        self.election_state.increment_votes();
    }
    pub fn set_max_term_seen(&mut self, term: u64) {
        self.election_state.set_max_term_seen(term);
    }
    pub fn reset_election_timer(&mut self) {
        self.last_heartbeat = std::time::Instant::now();
    }
    pub fn election_timed_out(&self) -> bool {
        std::time::Instant::now().duration_since(self.last_heartbeat) > *ELECTION_TIMEOUT
    }
    pub fn votes_granted(&self) -> u64 {
        self.election_state.votes_granted()
    }
    pub fn max_term_seen(&self) -> u64 {
        self.election_state.max_term_seen()
    }
}

impl Leader {
    pub fn new() -> Self {
        Leader {
            state: raft_models::state::LeaderState::default(),
        }
    }

    pub fn get_state(&self) -> &raft_models::state::LeaderState {
        &self.state
    }
    pub fn get_state_mut(&mut self) -> &mut raft_models::state::LeaderState {
        &mut self.state
    }
}

impl ElectionState {
    fn new(current_term: u64) -> Self {
        ElectionState {
            votes_granted: 1,
            max_term_seen: current_term,
        }
    }

    fn increment_votes(&mut self) {
        self.votes_granted += 1;
    }
    fn set_max_term_seen(&mut self, term: u64) {
        self.max_term_seen = term;
    }
    fn votes_granted(&self) -> u64 {
        self.votes_granted
    }
    fn max_term_seen(&self) -> u64 {
        self.max_term_seen
    }
}
