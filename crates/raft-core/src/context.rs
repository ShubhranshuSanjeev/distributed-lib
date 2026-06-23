use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

use crate::{
    cluster::{ClusterConfig, member::ClusterMemberId},
    states::Follower,
};

pub struct RaftContext {
    pub server_state: raft_models::state::ServerState,
    pub leader_state: raft_models::state::LeaderState,
    pub last_heartbeat: std::time::Instant,
    pub cluster_config: ClusterConfig,
    pub election_state: HashMap<ClusterMemberId, raft_models::rpc::RequestVoteResponse>,
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
            election_state: HashMap::new(),
        })
    }

    pub fn init() -> Result<Self, String> {
        Ok(RaftContext {
            server_state: raft_models::state::ServerState::default(),
            leader_state: raft_models::state::LeaderState::default(),
            last_heartbeat: std::time::Instant::now(),
            cluster_config: ClusterConfig::from_env()?,
            election_state: HashMap::new(),
        })
    }
}

pub struct Raft<T> {
    pub context: Rc<RefCell<RaftContext>>,
    pub state: PhantomData<T>,

    pub from_service: std::sync::mpsc::Receiver<crate::channels::AppendEntries>,
    pub to_service: std::sync::mpsc::Sender<crate::channels::AppendEntries>,
}

impl<T> Raft<T> {
    pub fn init(
        path: &str,
        from_service: std::sync::mpsc::Receiver<crate::channels::AppendEntries>,
        to_service: std::sync::mpsc::Sender<crate::channels::AppendEntries>,
    ) -> Result<Raft<Follower>, String> {
        let ctx = Rc::new(RefCell::new(RaftContext::from_persisted_state(path)?));
        Ok(Raft {
            context: ctx,
            state: PhantomData::<Follower>,
            from_service,
            to_service,
        })
    }
}
