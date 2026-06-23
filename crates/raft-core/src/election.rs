use std::{cell::LazyCell, marker::PhantomData};

use crate::{
    channels::{AppendEntries, RequestVote}, context::Raft, states::{Candidate, Follower, Leader}, utils::state_transition::STResult
};

const JITTER_MIN: u64 = 1;
const JITTER_MAX: u64 = 25;

const ELECTION_TIMEOUT: LazyCell<std::time::Duration> = LazyCell::new(|| {
    let jitter = rand::random_range(JITTER_MIN..=JITTER_MAX);
    std::time::Duration::from_millis(500 + jitter)
});

pub fn check_for_election(
    raft: Raft<Follower>,
) -> STResult<Raft<Candidate>, Raft<Follower>> {
    let timed_out = {
        std::time::Instant::now().duration_since(raft.context.borrow().last_heartbeat)
            > *ELECTION_TIMEOUT
    };
    if timed_out {
        return STResult::Ok(Raft {
            context: raft.context.clone(),
            state: PhantomData::<Candidate>,
            from_service: raft.from_service,
            to_service: raft.to_service,
        });
    }
    STResult::Aborted(raft)
}

pub async fn conduct_election<'a>(
    raft: Raft<Candidate>,
) -> STResult<Raft<Leader>, Raft<Candidate>> {
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

    raft.to_service.send(RequestVote(request_vote_payload));

    // TODO: send request vote event on the channel
    // to raft_server and await for responses
    // let mut jset = tokio::task::JoinSet::new();

    // for member in raft_ctx.cluster_config.members.iter() {
    //     let rpc_client = member.rpc_client.clone();

    //     jset.spawn(async move {
    //         let mut rpc_client = rpc_client.lock().await;
    //         let future = rpc_client.request_vote(tonic::Request::new(request_vote_payload.clone()));
    //         future.await
    //     });
    // }

    // let results = jset.join_all().await;
    // let mut votes_granted: u64 = 0;
    // for result in results {
    //     match result {
    //         Ok(response) => {
    //             if response.get_ref().vote_granted {
    //                 votes_granted += 1;
    //                 // TODO: there was some logic related to term returned in the response
    //                 // if the term in the response is greater than the candidate's term, update the candidate's term
    //                 // check this and implement if required
    //             }
    //         }
    //         Err(status) => {
    //             println!(
    //                 "ErrorCode: {}, Message: {}",
    //                 status.code(),
    //                 status.message()
    //             );
    //         }
    //     }
    // }

    // if votes_granted < raft.context.borrow().cluster_config.replication_factor {
    //     return STResult::Aborted(raft);
    // }

    STResult::Ok(Raft {
        context: raft.context.clone(),
        state: PhantomData::<Leader>,
        from_service: raft.from_service,
        to_service: raft.to_service,
    })
}
