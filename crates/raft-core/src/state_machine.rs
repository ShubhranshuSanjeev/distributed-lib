pub mod candidate;
pub mod follower;
pub mod leader;
pub mod run;
pub mod transitions;

use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use raft_models::RaftMessage;

use crate::{
    cluster::member::{self, ClusterMemberId},
    events::Event,
    states::{Follower, RaftSharedState},
};

pub struct Raft<T> {
    pub shared_state: Rc<RefCell<RaftSharedState>>,
    pub node: T,

    to_server: std::sync::mpsc::Sender<Event>,
    from_server: std::sync::mpsc::Receiver<Event>,
}

impl<T> Raft<T> {
    pub fn current_term(&self) -> u64 {
        self.shared_state.borrow().server_state.current_term
    }
    pub fn voted_for(&self) -> u64 {
        self.shared_state.borrow().server_state.voted_for
    }
    pub fn commit_index(&self) -> u64 {
        self.shared_state.borrow().server_state.commit_index
    }
    pub fn last_applied(&self) -> u64 {
        self.shared_state.borrow().server_state.last_applied
    }
    pub fn my_id(&self) -> ClusterMemberId {
        self.shared_state.borrow().cluster_config.my_id
    }
    pub fn replication_factor(&self) -> u64 {
        self.shared_state.borrow().cluster_config.quorum_size
    }
    pub fn cluster_members(&self) -> Ref<'_, [member::ClusterMember]> {
        Ref::map(self.shared_state.borrow(), |s| {
            s.cluster_config.members.as_slice()
        })
    }
    pub fn logs(&self) -> Ref<'_, [raft_models::common::LogEntry]> {
        Ref::map(self.shared_state.borrow(), |s| {
            s.server_state.log.as_slice()
        })
    }

    pub fn set_current_term(&mut self, term: u64) {
        self.shared_state.borrow_mut().server_state.current_term = term;
    }
    pub fn set_voted_for(&mut self, voted_for: u64) {
        self.shared_state.borrow_mut().server_state.voted_for = voted_for;
    }
    pub fn increment_term(&mut self) -> u64 {
        self.shared_state.borrow_mut().server_state.current_term += 1;
        self.shared_state.borrow().server_state.current_term
    }

    pub fn send(&self, event: Event) -> Result<(), std::sync::mpsc::SendError<Event>> {
        self.to_server.send(event)
    }
    pub fn receive(&self) -> Result<Event, std::sync::mpsc::RecvError> {
        self.from_server.recv()
    }
    pub fn try_receive(&self) -> Result<Event, std::sync::mpsc::TryRecvError> {
        self.from_server.try_recv()
    }
    pub fn stale_term(&mut self, req: &impl RaftMessage) -> bool {
        let term = req.term();
        if term > self.current_term() {
            self.set_current_term(term);
            return true;
        }
        false
    }
}

pub fn raft_init(
    path: &str,
    to_server: std::sync::mpsc::Sender<Event>,
    from_server: std::sync::mpsc::Receiver<Event>,
) -> Result<Raft<Follower>, String> {
    let raft_ss = Rc::new(RefCell::new(RaftSharedState::from_persisted_state(path)?));
    Ok(Raft {
        shared_state: raft_ss,
        node: Follower::new(),
        to_server,
        from_server,
    })
}
