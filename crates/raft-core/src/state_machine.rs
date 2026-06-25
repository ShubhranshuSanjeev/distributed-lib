use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use either::Either;

use crate::{
    channels::events::Event,
    context::{ElectionState, RaftContext},
    states::{Candidate, Follower, Leader},
};

pub struct Raft<T> {
    pub context: Rc<RefCell<RaftContext>>,

    state: PhantomData<T>,
    to_server: std::sync::mpsc::Sender<Event>,
    from_server: std::sync::mpsc::Receiver<Event>,
}

impl<T> Raft<T> {
    pub fn current_term(&self) -> u64 {
        self.context.borrow().server_state.current_term
    }
    pub fn replication_factor(&self) -> u64 {
        self.context.borrow().cluster_config.replication_factor
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
}


pub fn raft_init(
    path: &str,
    to_server: std::sync::mpsc::Sender<Event>,
    from_server: std::sync::mpsc::Receiver<Event>,
) -> Result<Raft<Follower>, String> {
    let ctx = Rc::new(RefCell::new(RaftContext::from_persisted_state(path)?));
    Ok(Raft {
        context: ctx,
        state: PhantomData::<Follower>,
        to_server,
        from_server,
    })
}


pub fn become_candidate(raft: Raft<Follower>) -> Raft<Candidate> {
    {
        let mut ctx = raft.context.borrow_mut();
        ctx.last_heartbeat = std::time::Instant::now();
        ctx.server_state.current_term += 1;
        ctx.election_state = ElectionState {
            votes_granted: 1,
            max_term_seen: ctx.server_state.current_term,
        };
    }
    Raft {
        context: raft.context,
        state: PhantomData::<Candidate>,
        to_server: raft.to_server,
        from_server: raft.from_server,
    }
}
pub fn become_follower(raft: Either<Raft<Leader>, Raft<Candidate>>) -> Raft<Follower> {
    match raft {
        Either::Left(raft) => Raft {
            context: raft.context,
            state: PhantomData::<Follower>,
            to_server: raft.to_server,
            from_server: raft.from_server,
        },
        Either::Right(raft) => Raft {
            context: raft.context,
            state: PhantomData::<Follower>,
            to_server: raft.to_server,
            from_server: raft.from_server,
        },
    }
}
pub fn become_leader(raft: Raft<Candidate>) -> Raft<Leader> {
    Raft {
        context: raft.context,
        state: PhantomData::<Leader>,
        to_server: raft.to_server,
        from_server: raft.from_server,
    }
}
