use crate::Ic3;
use logic_form::Lemma;
use minisat::Solver;
use satif::{SatResult, Satif};
use std::ops::Deref;

impl Ic3 {
    fn verify_invariant(&mut self, invariants: &[Lemma]) -> bool {
        let mut solver = Solver::new();
        self.model.load_trans(&mut solver);
        for lemma in invariants {
            solver.add_clause(&!lemma.deref());
        }
        if let SatResult::Sat(_) = solver.solve(&self.model.bad) {
            return false;
        }
        for lemma in invariants {
            if let SatResult::Sat(_) = solver.solve(&self.model.cube_next(lemma)) {
                return false;
            }
        }
        true
    }

    pub fn verify(&mut self) -> bool {
        let invariant = self
            .gipsat
            .frame
            .iter()
            .position(|frame| frame.is_empty())
            .unwrap();
        let mut invariants = Vec::new();
        for i in invariant..self.gipsat.frame.len() {
            for cube in self.gipsat.frame[i].iter() {
                invariants.push(cube.lemma.clone());
            }
        }
        if !self.verify_invariant(&invariants) {
            println!("invariant varify failed");
            return false;
        }
        println!(
            "inductive invariant verified with {} lemmas!",
            invariants.len()
        );
        true
    }
}
