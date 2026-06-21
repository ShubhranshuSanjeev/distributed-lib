use prost::Message;

use crate::state::PersistentState;

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
