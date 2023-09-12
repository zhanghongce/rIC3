use crate::{
    solver::{BlockResult, PdrSolver},
    worker::PdrWorker,
};

impl PdrWorker {
    pub fn verify(&self) -> bool {
        let invariant = self
            .frames
            .iter()
            .position(|frame| frame.is_empty())
            .unwrap();
        let mut solver = PdrSolver::new(self.share.clone(), invariant);
        for i in invariant..self.frames.len() {
            for cube in self.frames[i].iter() {
                solver.add_clause(&!cube.clone());
            }
        }
        for i in invariant..self.frames.len() {
            for cube in self.frames[i].iter() {
                solver.block_fetch(&self.frames);
                if let BlockResult::No(_) = solver.blocked(cube) {
                    return false;
                }
            }
        }
        println!("inductive invariant verified!");
        true
    }
}
