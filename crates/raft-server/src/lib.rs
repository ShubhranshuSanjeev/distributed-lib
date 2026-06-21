pub use raft_models::rpc as rpc_models;

pub mod service {
    include!("../generated/raft.service.rs");
}
pub mod candidate;
pub mod election;
pub mod follower;
pub mod leader;
pub mod raft_context;
pub mod state_transition_result;
pub mod cluster_config;
pub mod utils;
