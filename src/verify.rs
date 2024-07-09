use crate::IC3;
use aig::AigEdge;
use logic_form::{Clause, Cube, Lemma};
use minisat::Solver;
use satif::{SatResult, Satif};
use std::{ops::Deref, process::Command};

impl IC3 {
    fn invariant(&self) -> Vec<Lemma> {
        let invariant = self
            .frame
            .iter()
            .position(|frame| frame.is_empty())
            .unwrap();
        let mut invariants = Vec::new();
        for i in invariant..self.frame.len() {
            for cube in self.frame[i].iter() {
                invariants.push(cube.deref().clone());
            }
        }
        invariants.sort();
        invariants
    }

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
        let invariants = self.invariant();
        // for c in invariants.iter() {
        //     println!("{:?}", **c);
        // }

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

    pub fn certifaiger(&self) {
        let invariants = self.invariant();
        let invariants = invariants
            .iter()
            .map(|l| Cube::from_iter(l.iter().map(|l| self.ts_restore.restore(*l))));
        let mut certifaiger = self.aig.clone();
        let mut certifaiger_dnf = vec![];
        for cube in invariants {
            certifaiger_dnf
                .push(certifaiger.new_ands_node(cube.into_iter().map(|l| AigEdge::from_lit(l))));
        }
        let invariants = certifaiger.new_ors_node(certifaiger_dnf.into_iter());
        certifaiger.bads.clear();
        certifaiger.outputs.clear();
        certifaiger.outputs.push(invariants);
        certifaiger.to_file("./certifaiger.aig");
        let output = Command::new("/root/certifaiger/build/check")
            .arg(&self.args.model)
            .arg("./certifaiger.aig")
            .output()
            .expect("certifaiger not found");

        if output.status.success() {
            println!("certifaiger check passed");
        } else {
            println!("certifaiger check failed");
        }
    }
}
