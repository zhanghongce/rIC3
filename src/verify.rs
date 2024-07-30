use crate::IC3;
use aig::AigEdge;
use logic_form::{Clause, Cube, Lemma};
use minisat::Solver;
use satif::{SatResult, Satif};
use std::{
    io::{self, Write},
    ops::Deref,
    process::Command,
};

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

        if !self.verify_invariant(&invariants) {
            println!("invariant varify failed");
            return false;
        }
        if self.options.verbose > 0 {
            println!(
                "inductive invariant verified with {} lemmas!",
                invariants.len()
            );
        }
        if self.options.certifaiger {
            self.certifaiger()
        } else {
            true
        }
    }

    pub fn certifaiger(&self) -> bool {
        let invariants = self.invariant();
        let invariants = invariants
            .iter()
            .map(|l| Cube::from_iter(l.iter().map(|l| self.ts_restore.restore(*l))));
        let mut certifaiger = self.aig.clone();
        let mut certifaiger_dnf = vec![];
        for cube in invariants {
            certifaiger_dnf
                .push(certifaiger.new_ands_node(cube.into_iter().map(AigEdge::from_lit)));
        }
        let invariants = certifaiger.new_ors_node(certifaiger_dnf.into_iter());
        let constrains: Vec<AigEdge> = certifaiger.constraints.iter().map(|e| !*e).collect();
        let constrains = certifaiger.new_ors_node(constrains.into_iter());
        let invariants = certifaiger.new_or_node(invariants, constrains);
        certifaiger.bads.clear();
        certifaiger.outputs.clear();
        certifaiger.outputs.push(invariants);
        let certifaiger_file = tempfile::NamedTempFile::new().unwrap();
        let certifaiger_path = certifaiger_file.path().as_os_str().to_str().unwrap();
        certifaiger.to_file(certifaiger_path);
        let output = Command::new("/root/certifaiger/build/check")
            .arg(&self.options.model)
            .arg(certifaiger_path)
            .output()
            .expect("certifaiger not found");

        if output.status.success() {
            io::stdout().write_all(&output.stdout).unwrap();
            println!("certifaiger check passed");
            true
        } else {
            println!("certifaiger check failed");
            false
        }
    }
}
