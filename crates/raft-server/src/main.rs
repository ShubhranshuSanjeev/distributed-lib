use std::sync::mpsc::channel;

use raft_core::{
    channels::events::Event,
    runner::{RunResult, Runnable},
    state_machine::raft_init,
};
use raft_server::service::RaftServer;

#[tokio::main]
async fn main() {
    let (to_core, from_server) = channel::<Event>();
    let (to_server, from_core) = channel::<Event>();

    let core_t = tokio::spawn(async move {
        let raft = raft_init("state.bin", to_server, from_server).unwrap();
        let mut runner: Box<dyn Runnable> = Box::new(raft);
        loop {
            match runner.run() {
                RunResult::Follower(raft) => {
                    runner = Box::new(raft);
                }
                RunResult::Candidate(raft) => {
                    runner = Box::new(raft);
                }
                RunResult::Leader(raft) => {
                    runner = Box::new(raft);
                }
                RunResult::Err(e) => {
                    return;
                }
            };
        }
    });
    let server_t = tokio::spawn(async move {
        let server = RaftServer::new(to_core, from_core);
    });

    tokio::pin!(core_t);

    loop {
        tokio::select! {
            _ = core_t => {

                // if core completes tear down the process
                break;
            },
            _ = server_t => {
                // restart the server if panics
            },
        }
    }
}
