pub use raft_models::rpc as rpc_models;

pub mod service {
    include!("../generated/raft.service.rs");
}
