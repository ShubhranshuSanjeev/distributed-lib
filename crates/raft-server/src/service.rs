use raft_core::channels::events::Event;

pub struct RaftServer {
    to_core: std::sync::mpsc::Sender<Event>,
    from_core: std::sync::mpsc::Receiver<Event>,
}

impl RaftServer {
    pub fn new(to_core: std::sync::mpsc::Sender<Event>, from_core: std::sync::mpsc::Receiver<Event>) -> Self {
        Self { to_core, from_core }
    }

    pub fn send(&self, event: Event) -> Result<(), std::sync::mpsc::SendError<Event>> {
        self.to_core.send(event)
    }
    pub fn try_receive(&self) -> Result<Event, std::sync::mpsc::TryRecvError> {
        self.from_core.try_recv()
    }
}


// #[tonic::async_trait]
// impl crate::server::raft_service_server::RaftService for RaftServer {
//     async fn append_entries(
//         &self,
//         request: tonic::Request<crate::rpc_models::AppendEntriesRequest>,
//     ) -> Result<tonic::Response<crate::rpc_models::AppendEntriesResponse>, tonic::Status> {
//
//     }
//
//     async fn request_vote(
//         &self,
//         request: tonic::Request<crate::rpc_models::RequestVoteRequest>,
//     ) -> Result<tonic::Response<crate::rpc_models::RequestVoteResponse>, tonic::Status> {
//
//     }
// }
