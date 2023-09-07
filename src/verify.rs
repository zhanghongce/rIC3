use std::sync::Arc;

use crate::{
    broadcast::create_broadcast,
    solver::{BlockResult, PdrSolver},
    Pdr,
};

impl Pdr {
    pub fn verify(&self) -> bool {
        let frames = self.frames.frames.read().unwrap().clone();
        let invariant = frames.iter().position(|frame| frame.is_empty()).unwrap();
        let (sender, mut receiver) = create_broadcast(1);
        let mut solver = PdrSolver::new(self.share.clone(), invariant, receiver.remove(0));
        for i in invariant..frames.len() {
            for cube in frames[i].iter() {
                sender.send_clause(Arc::new(!cube.clone()));
            }
        }
        for i in invariant..frames.len() {
            for cube in frames[i].iter() {
                solver.block_fetch(&self.frames);
                if let BlockResult::No(_) = solver.blocked(cube) {
                    return false;
                }
            }
        }
        true
    }
}
