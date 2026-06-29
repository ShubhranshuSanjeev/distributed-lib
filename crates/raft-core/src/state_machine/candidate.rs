use raft_models::rpc;

use crate::{
    events::{Event, RequestVote},
    state_machine::Raft,
    states::Candidate,
};

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
            let (last_log_index, last_log_term) = self
                .logs()
                .last()
                .map(|l| (l.index, l.term))
                .unwrap_or((0, self.current_term()));

            rpc::RequestVoteRequest {
                term: self.current_term(),
                candidate_id: self.my_id(),
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
