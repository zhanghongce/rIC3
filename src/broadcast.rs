use logic_form::Clause;
use std::sync::{
    mpsc::{channel, Receiver, Sender, TryRecvError},
    Arc,
};

pub struct PdrSolverBroadcastSender {
    pub senders: Vec<Sender<Arc<Clause>>>,
}

impl PdrSolverBroadcastSender {
    pub fn send_clause(&self, clause: Arc<Clause>) {
        for sender in self.senders.iter() {
            sender.send(clause.clone()).unwrap();
        }
    }
}

unsafe impl Sync for PdrSolverBroadcastSender {}

unsafe impl Send for PdrSolverBroadcastSender {}

pub struct PdrSolverBroadcastReceiver {
    receiver: Receiver<Arc<Clause>>,
}

impl PdrSolverBroadcastReceiver {
    pub fn receive_clause(&mut self) -> Option<Arc<Clause>> {
        match self.receiver.try_recv() {
            Ok(clause) => Some(clause),
            Err(err) => match err {
                TryRecvError::Empty => None,
                TryRecvError::Disconnected => todo!(),
            },
        }
    }
}

pub fn create_broadcast(
    num_receiver: usize,
) -> (PdrSolverBroadcastSender, Vec<PdrSolverBroadcastReceiver>) {
    let mut receivers = Vec::new();
    let mut senders = Vec::new();
    for _ in 0..num_receiver {
        let (s, r) = channel();
        receivers.push(PdrSolverBroadcastReceiver { receiver: r });
        senders.push(s);
    }
    (PdrSolverBroadcastSender { senders }, receivers)
}
