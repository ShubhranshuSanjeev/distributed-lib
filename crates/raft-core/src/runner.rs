use either::Either;
use std::sync::mpsc::TryRecvError;

use crate::{
    channels::events::Event,
    election::{drive_election, start_election},
    state_machine::{Raft, become_follower},
    states::{Candidate, Follower, Leader},
    utils::state_transition::STResult,
};

pub enum RunResult {
    Follower(Raft<Follower>),
    Candidate(Raft<Candidate>),
    Leader(Raft<Leader>),
    Err(String),
}

pub trait Runnable {
    fn run(self) -> RunResult;
}

impl Runnable for Raft<Follower> {
    fn run(self) -> RunResult {
        let mut raft = self;
        loop {
            match start_election(raft) {
                STResult::Ok(o) => {
                    return RunResult::Candidate(o);
                }
                STResult::Aborted(follower_raft) => {
                    raft = follower_raft;
                    // election failed, continue waiting
                }
            }
            match raft.try_receive() {
                Err(TryRecvError::Disconnected) => {
                    return RunResult::Err("disconnected".to_string());
                }
                Err(TryRecvError::Empty) => { /*noop */ }
                Ok(Event::AppendEntries(r)) => {
                    // handle append entries
                    // both heartbeat and actual entries
                }
                Ok(Event::RequestVote(r)) => {
                    // handle request vote
                }
                Ok(_) => { /*noop */ }
            }
        }
    }
}

impl Runnable for Raft<Candidate> {
    fn run(self) -> RunResult {
        let mut raft = self;
        loop {
            loop {
                // check for election timeout
                // match start_election(raft) {
                //     STResult::Ok(o) => {
                //         return Ok(o);
                //     }
                //     STResult::Aborted(follower_raft) => {
                //         raft = follower_raft;
                //         // election failed, continue waiting
                //     }
                // }
                match raft.try_receive() {
                    Err(TryRecvError::Disconnected) => {
                        return RunResult::Err("disconnected".to_string());
                    }
                    Err(TryRecvError::Empty) => { /*noop */ }
                    Ok(Event::AppendEntries(r)) => {
                        let nraft = become_follower(Either::Right(raft));
                        // handle append entries
                        // both heartbeat and actual entries
                        return RunResult::Follower(nraft);
                    }
                    Ok(Event::VoteResult(r)) => match drive_election(r.response, raft) {
                        STResult::Ok(o) => {
                            return RunResult::Leader(o);
                        }
                        STResult::Aborted(candidate_raft) => {
                            raft = candidate_raft;
                        }
                    },
                    Ok(_) => { /*noop */ }
                }
            }
        }
    }
}

impl Runnable for Raft<Leader> {
    fn run(self) -> RunResult {
        RunResult::Err("unexpected".to_string())
    }
}
