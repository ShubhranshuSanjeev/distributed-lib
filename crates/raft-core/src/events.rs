use std::str::Bytes;

use crate::cluster::member::ClusterMemberId;

// Core -> Server, or Server -> Core
pub struct AppendEntries {
    pub req: raft_models::rpc::AppendEntriesRequest,
}

pub struct RequestVote {
    pub req: raft_models::rpc::RequestVoteRequest,
}

// From Server to Core
pub struct AppendResult {
    pub from: ClusterMemberId,
    pub response: raft_models::rpc::AppendEntriesResponse,
}
pub struct VoteResult {
    pub from: ClusterMemberId,
    pub response: raft_models::rpc::RequestVoteResponse,
}

pub struct ClientRequest {
    pub command: Vec<u8>
}

pub enum Event {
    AppendEntries(AppendEntries),
    RequestVote(RequestVote),
    AppendResult(AppendResult),
    VoteResult(VoteResult),
    ClientRequest(ClientRequest),
}
