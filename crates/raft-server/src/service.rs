struct RaftService {
    to_core: std::sync::mpsc::Sender<raft_core::channels::AppendEntries>,
}

#[tonic::async_trait]
impl crate::server::raft_service_server::RaftService for RaftService {
    async fn append_entries(
        &self,
        request: tonic::Request<crate::rpc_models::AppendEntriesRequest>,
    ) -> Result<tonic::Response<crate::rpc_models::AppendEntriesResponse>, tonic::Status> {

    }

    async fn request_vote(
        &self,
        request: tonic::Request<crate::rpc_models::RequestVoteRequest>,
    ) -> Result<tonic::Response<crate::rpc_models::RequestVoteResponse>, tonic::Status> {

    }
}
