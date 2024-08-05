use crate::{proofoblig::ProofObligation, transys::Transys, IC3};
// use aig::AigEdge;
use logic_form::{Clause, Lemma, Lit};
use minisat::Solver;
use satif::Satif;
use std::{
    // io::{self, Write},
    ops::Deref,
    // process::Command,
};

pub fn verify_invariant(ts: &Transys, invariants: &[Lemma]) -> bool {
    let mut solver = Solver::new();
    while solver.num_var() < ts.num_var {
        solver.new_var();
    }
    for cls in ts.trans.iter() {
        solver.add_clause(cls)
    }
    for lemma in invariants {
        solver.add_clause(&!lemma.deref());
    }
    for c in ts.constraints.iter() {
        solver.add_clause(&Clause::from([*c]));
    }
    if solver.solve(&ts.bad) {
        return false;
    }
    for lemma in invariants {
        let mut assump = ts.constraints.clone();
        assump.extend_from_slice(&ts.bad);
        if solver.solve(&ts.cube_next(lemma)) {
            return false;
        }
    }
    true
}

impl IC3 {
    pub fn verify(&mut self) -> bool {
        let invariants = self.frame.invariant();

        if !verify_invariant(&self.ts, &invariants) {
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
        // let invariants = self.invariant();
        // let invariants = invariants
        //     .iter()
        //     .map(|l| Cube::from_iter(l.iter().map(|l| self.ts_restore.restore(*l))));
        // let mut certifaiger = self.aig.clone();
        // let mut certifaiger_dnf = vec![];
        // for cube in invariants {
        //     certifaiger_dnf
        //         .push(certifaiger.new_ands_node(cube.into_iter().map(AigEdge::from_lit)));
        // }
        // let invariants = certifaiger.new_ors_node(certifaiger_dnf.into_iter());
        // let constrains: Vec<AigEdge> = certifaiger.constraints.iter().map(|e| !*e).collect();
        // let constrains = certifaiger.new_ors_node(constrains.into_iter());
        // let invariants = certifaiger.new_or_node(invariants, constrains);
        // certifaiger.bads.clear();
        // certifaiger.outputs.clear();
        // certifaiger.outputs.push(invariants);
        // let certifaiger_file = tempfile::NamedTempFile::new().unwrap();
        // let certifaiger_path = certifaiger_file.path().as_os_str().to_str().unwrap();
        // certifaiger.to_file(certifaiger_path);
        // let output = Command::new("/root/certifaiger/build/check")
        //     .arg(&self.options.model)
        //     .arg(certifaiger_path)
        //     .output()
        //     .expect("certifaiger not found");

        // if output.status.success() {
        //     io::stdout().write_all(&output.stdout).unwrap();
        //     println!("certifaiger check passed");
        //     true
        // } else {
        //     println!("certifaiger check failed");
        //     false
        // }
        todo!()
    }

    pub fn check_witness(&mut self, mut bad: ProofObligation) -> Option<Lit> {
        while bad.next.is_some() {
            let next = bad.next.clone().unwrap();
            let mut assump = bad.lemma.deref().clone();
            assump.extend_from_slice(&bad.input);
            let imply = self.ts.cube_next(&next.lemma);
            self.lift.imply(
                imply
                    .iter()
                    .chain(self.ts.constraints.iter())
                    .map(|l| l.var()),
                assump.iter(),
            );
            assert!(imply
                .iter()
                .all(|l| self.lift.sat_value(*l).is_some_and(|v| v)));
            if let Some(v) = self
                .ts
                .constraints
                .iter()
                .find(|l| !self.lift.sat_value(**l).is_some_and(|v| v))
            {
                return Some(*v);
            }
            bad = next;
        }
        if self.options.verbose > 0 {
            println!("witness checking passed");
        }
        None
    }
}
