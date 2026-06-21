use std::{cell::LazyCell, marker::PhantomData};

use crate::{
    candidate::Candidate, follower::Follower, leader::Leader,
    raft_context::Raft, state_transition_result::STResult,
};

const JITTER_MIN: u64 = 1;
const JITTER_MAX: u64 = 25;

const ELECTION_TIMEOUT: LazyCell<std::time::Duration> = LazyCell::new(|| {
    let jitter = rand::random_range(JITTER_MIN..=JITTER_MAX);
    std::time::Duration::from_millis(500 + jitter)
});

pub fn check_for_election<'a>(
    raft: &'a Raft<Follower>,
) -> STResult<Raft<Candidate>, &'a Raft<Follower>> {
    let timed_out = {
        std::time::Instant::now().duration_since(raft.context.borrow().last_heartbeat)
            > *ELECTION_TIMEOUT
    };
    if timed_out {
        return STResult::Ok(Raft {
            context: raft.context.clone(),
            state: PhantomData::<Candidate>,
        });
    }
    STResult::Aborted(raft)
}

pub async fn conduct_election<'a>(
    raft: &'a Raft<Candidate>,
) -> STResult<Raft<Leader>, &'a Raft<Candidate>> {
    let raft_ctx = raft.context.borrow();

    let request_vote_payload = {
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

    let mut jset = tokio::task::JoinSet::new();
    for member in raft_ctx.cluster_config.members.iter() {
        let rpc_client = member.rpc_client.clone();

        jset.spawn(async move {
            let mut rpc_client = rpc_client.lock().await;
            let future = rpc_client.request_vote(tonic::Request::new(request_vote_payload.clone()));
            future.await
        });
    }

    let results = jset.join_all().await;
    // check for majority votes

    STResult::Ok(Raft {
        context: raft.context.clone(),
        state: PhantomData::<Leader>,
    })
}
