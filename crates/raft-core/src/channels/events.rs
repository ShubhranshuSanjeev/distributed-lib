use crate::cluster::member::ClusterMemberId;

pub struct AppendEntries((raft_models::rpc::AppendEntriesRequest, ));
pub struct RequestVote(raft_models::rpc::RequestVoteRequest);
pub struct AppendResult {
    from: ClusterMemberId,
    response: raft_models::rpc::AppendEntriesResponse,
}
pub struct VoteResult {
    from: ClusterMemberId,
    response: raft_models::rpc::RequestVoteResponse,
}

enum Event {

}
