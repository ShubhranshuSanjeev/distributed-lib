use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

use crate::{
    cluster_config::{ClusterConfig, ClusterMemberId},
    follower::Follower,
};

pub struct RaftContext {
    pub server_state: raft_models::state::ServerState,
    pub leader_state: raft_models::state::LeaderState,
    pub last_heartbeat: std::time::Instant,
    pub cluster_config: ClusterConfig,
    pub election_state: HashMap<ClusterMemberId, raft_models::rpc::RequestVoteResponse>,
}

impl RaftContext {
    pub async fn from_persisted_state() -> Result<Self, String> {
        match raft_models::state::PersistentState::load_from_file("") {
            Err(e) => {
                println!("failed to load persistent state {}", e);
                Self::init().await
            }
            Ok(persisted_state) => {
                let cluster_config = ClusterConfig::from_env().await?;
                Ok(RaftContext {
                    server_state: raft_models::state::ServerState {
                        current_term: persisted_state.current_term,
                        voted_for: persisted_state.voted_for,
                        log: persisted_state.log,
                        commit_index: 0,
                        last_applied: 0,
                    },
                    leader_state: raft_models::state::LeaderState::default(),
                    last_heartbeat: std::time::Instant::now(),
                    cluster_config,
                    election_state: HashMap::new(),
                })
            }
        }
    }

    async fn init() -> Result<Self, String> {
        let cluster_config = ClusterConfig::from_env().await?;
        Ok(RaftContext {
            server_state: raft_models::state::ServerState::default(),
            leader_state: raft_models::state::LeaderState::default(),
            last_heartbeat: std::time::Instant::now(),
            cluster_config,
            election_state: HashMap::new(),
        })
    }
}

pub struct Raft<T> {
    pub context: Rc<RefCell<RaftContext>>,
    pub state: PhantomData<T>,
}

impl<T> Raft<T> {
    pub async fn init() -> Result<Raft<Follower>, String> {
        let ctx = Rc::new(RefCell::new(RaftContext::from_persisted_state().await?));
        Ok(Raft {
            context: ctx,
            state: PhantomData::<Follower>,
        })
    }
}
