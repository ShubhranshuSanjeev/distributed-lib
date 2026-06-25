use crate::cluster::ClusterConfig;

#[derive(Default)]
pub struct ElectionState {
    pub votes_granted: u64,
    pub max_term_seen: u64,
}

pub struct RaftContext {
    pub server_state: raft_models::state::ServerState,
    pub leader_state: raft_models::state::LeaderState,
    pub last_heartbeat: std::time::Instant,
    pub cluster_config: ClusterConfig,
    pub election_state: ElectionState,
}

impl RaftContext {
    pub fn from_persisted_state(path: &str) -> Result<Self, String> {
        let server_state = raft_models::state::ServerState::from_persistent_state(path);
        let cluster_config = ClusterConfig::from_env()?;

        Ok(RaftContext {
            server_state,
            cluster_config,
            leader_state: raft_models::state::LeaderState::default(),
            last_heartbeat: std::time::Instant::now(),
            election_state: ElectionState::default(),
        })
    }

    pub fn init() -> Result<Self, String> {
        Ok(RaftContext {
            server_state: raft_models::state::ServerState::default(),
            leader_state: raft_models::state::LeaderState::default(),
            last_heartbeat: std::time::Instant::now(),
            cluster_config: ClusterConfig::from_env()?,
            election_state: ElectionState::default(),
        })
    }
}
