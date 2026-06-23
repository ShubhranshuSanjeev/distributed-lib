pub use raft_models::rpc as rpc_models;

pub mod server {
    include!("../generated/raft.service.rs");
}
pub mod service;
