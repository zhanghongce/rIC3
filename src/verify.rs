use crate::{
    solver::{BlockResult, Ic3Solver},
    worker::Ic3Worker,
};

impl Ic3Worker {
    pub fn verify(&self) -> bool {
        let invariant = self
            .frames
            .iter()
            .position(|frame| frame.is_empty())
            .unwrap();
        let mut solver = Ic3Solver::new(self.share.clone(), invariant);
        let mut num = 0;
        for i in invariant..self.frames.len() {
            for cube in self.frames[i].iter() {
                num += 1;
                solver.add_clause(&!cube);
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
        println!("inductive invariant verified with {num} lemmas!");
        true
    }
}
