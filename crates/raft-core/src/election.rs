use std::cell::LazyCell;

use either::Either;

use crate::{
    channels::{RequestVote, events::Event},
    state_machine::{Raft, become_candidate, become_follower, become_leader},
    states::{Candidate, Follower, Leader},
    utils::state_transition::STResult,
};

const JITTER_MIN: u64 = 1;
const JITTER_MAX: u64 = 25;

const ELECTION_TIMEOUT: LazyCell<std::time::Duration> = LazyCell::new(|| {
    let jitter = rand::random_range(JITTER_MIN..=JITTER_MAX);
    std::time::Duration::from_millis(500 + jitter)
});

pub fn start_election(raft: Raft<Follower>) -> STResult<Raft<Candidate>, Raft<Follower>> {
    let timed_out = {
        let raft_ctx = raft.context.borrow();
        std::time::Instant::now().duration_since(raft_ctx.last_heartbeat) > *ELECTION_TIMEOUT
    };
    if !timed_out {
        return STResult::Aborted(raft);
    }

    let raft = become_candidate(raft);
    let request_vote_payload = {
        let raft_ctx = raft.context.borrow();
        let (last_log_index, last_log_term) = raft_ctx
            .server_state
            .log
            .last()
            .map(|l| (l.index, l.term))
            .unwrap_or((0, raft_ctx.server_state.current_term));

        raft_models::rpc::RequestVoteRequest {
            term: raft_ctx.server_state.current_term,
            candidate_id: raft_ctx.cluster_config.my_id,
            last_log_index,
            last_log_term,
        }
    };

    let send_result = raft.send(Event::RequestVote(RequestVote {
        req: request_vote_payload,
    }));

    match send_result {
        Ok(_) => STResult::Ok(raft),
        Err(_) => STResult::Aborted(become_follower(Either::Right(raft))),
    }
}

pub fn drive_election(
    vote_resp: raft_models::rpc::RequestVoteResponse,
    raft: Raft<Candidate>,
) -> STResult<Raft<Leader>, Raft<Candidate>> {
    {
        // state update
        let mut raft_ctx = raft.context.borrow_mut();
        raft_ctx.election_state.max_term_seen =
            std::cmp::max(raft_ctx.election_state.max_term_seen, vote_resp.term);
        if vote_resp.vote_granted {
            raft_ctx.election_state.votes_granted += 1;
        }
    }

    let transition_to_leader = {
        let raft_ctx = raft.context.borrow();
        raft_ctx.election_state.votes_granted >= raft.replication_factor()
    };

    if transition_to_leader {
        return STResult::Ok(become_leader(raft));
    }

    STResult::Aborted(raft)
}
