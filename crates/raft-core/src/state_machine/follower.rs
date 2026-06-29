use raft_models::rpc;

use crate::{events::{Event, VoteResult}, state_machine::Raft, states::Follower};

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
    pub fn append_entries(&mut self, req: rpc::AppendEntriesRequest) -> Result<(), String> {
        todo!()
    }
}
