pub mod common {
    include!("../generated/raft.common.rs");
}
pub mod state {
    include!("../generated/raft.state.rs");
}
pub mod rpc {
    include!("../generated/raft.rpc_models.rs");
}
