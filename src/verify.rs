use crate::IC3;
use logic_form::{Clause, Lemma};
use minisat::Solver;
use satif::{SatResult, Satif};
use std::ops::Deref;

impl IC3 {
    fn verify_invariant(&mut self, invariants: &[Lemma]) -> bool {
        let mut solver = Solver::new();
        while solver.num_var() < self.ts.num_var {
            solver.new_var();
        }
        for cls in self.ts.trans.iter() {
            solver.add_clause(cls)
        }
        for lemma in invariants {
            solver.add_clause(&!lemma.deref());
        }
        for c in self.ts.constraints.iter() {
            solver.add_clause(&Clause::from([*c]));
        }
        if let SatResult::Sat(_) = solver.solve(&[self.ts.bad]) {
            return false;
        }
        for lemma in invariants {
            let mut assump = self.ts.constraints.clone();
            assump.push(self.ts.bad);
            if let SatResult::Sat(_) = solver.solve(&self.ts.cube_next(lemma)) {
                return false;
            }
        }
        true
    }

    pub fn verify(&mut self) -> bool {
        let invariant = self
            .frame
            .iter()
            .position(|frame| frame.is_empty())
            .unwrap();
        let mut invariants = Vec::new();
        for i in invariant..self.frame.len() {
            for cube in self.frame[i].iter() {
                invariants.push(cube.clone());
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
