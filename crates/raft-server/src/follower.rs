use crate::{raft_context::Raft};

pub struct Follower;

// impl Follower {
//     fn prepare_for_election(raft: &Raft<Follower>) {}
//     fn append_entries(raft: &Raft<Follower>, request: raft_models::rpc::AppendEntriesRequest) {}
//     fn receive_heartbeat(raft: &Raft<Follower>, t: std::time::Instant) {}
//     fn receive_vote_request(raft: &Raft<Follower>, request: raft_models::rpc::RequestVoteRequest) {}
// }
