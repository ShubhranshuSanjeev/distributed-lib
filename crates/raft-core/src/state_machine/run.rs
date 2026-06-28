use std::sync::mpsc::TryRecvError;

use raft_models::rpc::RequestVoteResponse;

use crate::{
    events::{AppendResult, Event, RequestVote, VoteResult},
    state_machine::{
        Raft,
        transitions::{RaftNode, Transition},
    },
    states::{Candidate, Follower, Leader},
};

impl Raft<Follower> {
    fn run(self) -> Transition {
        let mut raft = self;
        loop {
            if raft.election_timed_out() {
                let candidate = Raft::<Candidate>::from(raft);
                candidate.request_votes();
                return Transition::To(RaftNode::Candidate(candidate));
            }

            match raft.try_receive() {
                Err(TryRecvError::Disconnected) => {
                    return Transition::Shutdown("disconnected".to_string());
                }
                Err(TryRecvError::Empty) => { /*noop */ }
                Ok(Event::AppendEntries(r)) => {
                    raft.stale_term(&r.req);
                    raft.receive_heartbeat(&r.req);
                    // handle append entries
                    // both heartbeat and actual entries
                }
                Ok(Event::RequestVote(RequestVote { req })) => {
                    raft.stale_term(&req);
                    match raft.grant_vote(req) {
                        Ok(_) => { /*noop */ }
                        Err(e) => {
                            // TODO: add a connection error type
                            // re-establish channels between server and core
                            // or restart the server completely
                            return Transition::Shutdown(e);
                        }
                    }
                }
                Ok(_) => { /*noop */ }
            }
        }
    }
}

impl Raft<Candidate> {
    fn run(self) -> Transition {
        let mut raft = self;
        loop {
            if raft.election_timed_out() {
                let term = raft.increment_term();
                raft.reset_election_timer();
                raft.increment_votes();
                raft.set_max_term_seen(term);

                raft.request_votes();
            }
            match raft.try_receive() {
                Err(TryRecvError::Disconnected) => {
                    return Transition::Shutdown("disconnected".to_string());
                }
                Err(TryRecvError::Empty) => { /*noop */ }
                Ok(Event::AppendEntries(r)) => {
                    let modified = raft.stale_term(&r.req);
                    if modified {
                        let follower = Raft::<Follower>::from(raft);
                        // TODO: handle append entries
                        return Transition::To(RaftNode::Follower(follower));
                    }
                    // else no-op
                }
                Ok(Event::VoteResult(VoteResult { from, response })) => {
                    let modified = raft.stale_term(&response);
                    if modified {
                        let follower = Raft::<Follower>::from(raft);
                        return Transition::To(RaftNode::Follower(follower));
                    }

                    let RequestVoteResponse { term, vote_granted } = response;
                    if term > raft.current_term() {
                        panic!(
                            "HOW THE HELL DID WE GET A VOTE FROM A TERM THAT'S HIGHER THAN OURS?"
                        )
                    }

                    if vote_granted && term == raft.current_term() {
                        raft.increment_votes();
                        if raft.has_quorum() {
                            return Transition::To(RaftNode::Leader(Raft::<Leader>::from(raft)));
                        }
                    }
                }
                Ok(_) => { /*noop */ }
            }
        }
    }
}

impl Raft<Leader> {
    fn run(self) -> Transition {
        let mut raft = self;
        loop {
            match raft.try_receive() {
                Err(TryRecvError::Disconnected) => {
                    return Transition::Shutdown("disconnected".to_string());
                }
                Err(TryRecvError::Empty) => { /*noop */ }
                Ok(Event::ClientRequest) => {
                    todo!()
                }
                Ok(Event::AppendEntries(r)) => {
                    let modified = raft.stale_term(&r.req);
                    if modified {
                        let follower = Raft::<Follower>::from(raft);
                        // TODO: handle append entries
                        return Transition::To(RaftNode::Follower(follower));
                    }
                    // else no-op
                }
                Ok(Event::AppendResult(AppendResult { from, response })) => {
                    let modified = raft.stale_term(&response);
                    if modified {
                        // PONDER: why the hell this happened????
                        let follower = Raft::<Follower>::from(raft);
                        return Transition::To(RaftNode::Follower(follower));
                    }
                }
                Ok(Event::RequestVote(RequestVote { req })) => {
                    let modified = raft.stale_term(&req);
                    if modified {
                        let mut follower = Raft::<Follower>::from(raft);
                        match follower.grant_vote(req) {
                            Ok(_) => { /*noop */ }
                            Err(e) => {
                                // TODO: add a connection error type
                                // re-establish channels between server and core
                                // or restart the server completely
                                return Transition::Shutdown(e);
                            }
                        }
                        return Transition::To(RaftNode::Follower(follower));
                    }
                }
                Ok(_) => { /*noop */ }
            }
        }
    }
}

impl RaftNode {
    pub fn run(self) -> Transition {
        match self {
            RaftNode::Follower(raft) => raft.run(),
            RaftNode::Candidate(raft) => raft.run(),
            RaftNode::Leader(raft) => raft.run(),
        }
    }
}
