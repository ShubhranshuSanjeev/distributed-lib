pub use raft_models::rpc as rpc_models;

pub mod client {
    include!("../generated/raft.service.rs");
}
