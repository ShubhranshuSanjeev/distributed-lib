use std::sync::mpsc::channel;

use raft_core::{
    events::Event,
    state_machine::{
        raft_init,
        transitions::{RaftNode, Transition},
    },
};
use raft_server::service::RaftServer;

#[tokio::main]
async fn main() {
    let (to_core, from_server) = channel::<Event>();
    let (to_server, from_core) = channel::<Event>();

    let core_t = tokio::spawn(async move {
        let mut raft = RaftNode::Follower(raft_init("state.bin", to_server, from_server).unwrap());
        loop {
            match raft.run() {
                Transition::To(rnode) => raft = rnode,
                Transition::Shutdown(err) => {
                    // TODO: handle the error
                    return;
                }
            };
        }
    });
    let server_t = tokio::spawn(async move {
        let server = RaftServer::new(to_core, from_core);
    });

    tokio::pin!(core_t);
    tokio::select! {
        _ = core_t => {
        },
        _ = server_t => {
        },
    }
}
