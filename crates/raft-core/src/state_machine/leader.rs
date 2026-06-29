use raft_models::{rpc, state};

use crate::{
    events::{AppendEntries, ClientRequest, Event},
    state_machine::Raft,
    states::Leader,
};

impl Raft<Leader> {
    // need much more fine-grained
    pub fn get_leader_state<'a>(&'a self) -> &'a state::LeaderState {
        self.node.get_state()
    }
    pub fn get_leader_state_mut<'a>(&'a mut self) -> &'a mut state::LeaderState {
        self.node.get_state_mut()
    }

    pub fn send_heartbeat(&mut self) -> Result<(), String> {
        let request_payload = {
            let (last_log_index, last_log_term) = self
                .logs()
                .last()
                .map(|log| (log.index, log.term))
                .unwrap_or((0, self.current_term()));
            rpc::AppendEntriesRequest {
                term: self.current_term(),
                leader_id: self.my_id(),
                prev_log_index: last_log_index,
                prev_log_term: last_log_term,
                entries: vec![],
                leader_commit_index: self.commit_index(),
            }
        };

        self.send(Event::AppendEntries(AppendEntries {
            req: request_payload,
        }))
        .map_err(|_| {
            format!("failed to send heartbeat event to the server, possible disconnection")
        })
    }

    pub fn process_client_request(&mut self, req: ClientRequest) -> Result<(), String> {
    }
}
