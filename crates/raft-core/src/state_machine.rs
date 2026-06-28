pub mod run;
pub mod transitions;

use std::{cell::RefCell, rc::Rc};

use raft_models::{RaftMessage, rpc, state};

use crate::{
    cluster::member::ClusterMemberId,
    events::{Event, RequestVote, VoteResult},
    states::{Candidate, Follower, Leader, RaftSharedState},
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

impl Raft<Follower> {
    pub fn reset_election_timer(&mut self) {
        self.node.reset_election_timer();
    }

    pub fn election_timed_out(&self) -> bool {
        self.node.election_timed_out()
    }

    pub fn grant_vote(&mut self, req: rpc::RequestVoteRequest) -> Result<(), String> {
        let voter_id = self.my_id();
        let voted_for = self.voted_for();
        let voter_term = self.current_term();
        let voter_index = self.last_applied();

        let vote_granted = if voter_term > req.term {
            false
        } else if voter_term == req.term {
            voted_for == 0 && voter_index >= req.last_log_index
        } else {
            true
        };

        self.send(Event::VoteResult(VoteResult {
            from: voter_id,
            response: rpc::RequestVoteResponse {
                term: voter_term,
                vote_granted,
            },
        }))
        .map(|_| {
            if vote_granted {
                self.set_voted_for(voter_id);
            }
        })
        .map_err(|_| format!("Failed to send RequestVote event to server"))
    }

    pub fn receive_heartbeat(&mut self, _req: &rpc::AppendEntriesRequest) {
        self.reset_election_timer();
    }
}

impl Raft<Candidate> {
    pub fn votes_granted(&self) -> u64 {
        self.node.votes_granted()
    }
    pub fn max_term_seen(&self) -> u64 {
        self.node.max_term_seen()
    }

    pub fn increment_votes(&mut self) {
        self.node.increment_votes();
    }
    pub fn set_max_term_seen(&mut self, term: u64) {
        self.node.set_max_term_seen(term);
    }
    pub fn reset_election_timer(&mut self) {
        self.node.reset_election_timer();
    }

    pub fn election_timed_out(&self) -> bool {
        self.node.election_timed_out()
    }
    pub fn has_quorum(&self) -> bool {
        let votes_received = self.votes_granted();
        let quorum_size = self.shared_state.borrow().cluster_config.quorum_size;
        votes_received >= quorum_size
    }
    pub fn request_votes(&self) -> Result<(), String> {
        let request_vote_payload = {
            let ctx = self.shared_state.borrow();
            let (last_log_index, last_log_term) = ctx
                .server_state
                .log
                .last()
                .map(|l| (l.index, l.term))
                .unwrap_or((0, ctx.server_state.current_term));

            rpc::RequestVoteRequest {
                term: ctx.server_state.current_term,
                candidate_id: ctx.cluster_config.my_id,
                last_log_index,
                last_log_term,
            }
        };
        self.send(Event::RequestVote(RequestVote {
            req: request_vote_payload,
        }))
        .map_err(|_| format!("Failed to send RequestVote event to server"))
    }

    pub fn receive_heartbeat(&mut self, _req: rpc::AppendEntriesRequest) {
        self.reset_election_timer();
    }
}

impl Raft<Leader> {
    // need much more fine-grained
    pub fn get_leader_state<'a>(&'a self) -> &'a state::LeaderState {
        self.node.get_state()
    }
    pub fn get_leader_state_mut<'a>(&'a mut self) -> &'a mut state::LeaderState {
        self.node.get_state_mut()
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
