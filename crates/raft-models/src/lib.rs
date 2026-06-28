use prost::Message;

use crate::state::{PersistentState, ServerState};

pub mod common {
    include!("../generated/raft.common.rs");
}
pub mod state {
    include!("../generated/raft.state.rs");
}
pub mod rpc {
    include!("../generated/raft.rpc_models.rs");
}

impl PersistentState {
    pub fn load_from_file(path: &str) -> Result<Self, prost::DecodeError> {
        let data: Vec<u8> = std::fs::read(path).unwrap();
        PersistentState::decode(data.as_slice())
    }
}

impl ServerState {
    pub fn from_persistent_state(path: &str) -> Self {
        match PersistentState::load_from_file(path) {
            Err(e) => {
                println!("failed to load persistent state {}", e);
                Self::default()
            }
            Ok(ps) => Self {
                current_term: ps.current_term,
                voted_for: ps.voted_for,
                log: ps.log,
                // TODO: there should be some relation between the logs
                // and then these commit_index and last_applied fields
                commit_index: 0,
                last_applied: 0,
            },
        }
    }
}

pub trait RaftMessage {
    fn term(&self) -> u64;
    fn is_heartbeat(&self) -> bool;
}

impl RaftMessage for rpc::AppendEntriesRequest {
    fn term(&self) -> u64 {
        self.term
    }

    fn is_heartbeat(&self) -> bool {
        self.entries.is_empty()
    }
}
impl RaftMessage for rpc::RequestVoteRequest {
    fn term(&self) -> u64 {
        self.term
    }

    fn is_heartbeat(&self) -> bool {
        false
    }
}
impl RaftMessage for rpc::RequestVoteResponse {
    fn term(&self) -> u64 {
        self.term
    }

    fn is_heartbeat(&self) -> bool {
        false
    }
}
impl RaftMessage for rpc::AppendEntriesResponse {
    fn term(&self) -> u64 {
        self.term
    }

    fn is_heartbeat(&self) -> bool {
        false
    }
}
