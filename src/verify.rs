use crate::{
    proofoblig::ProofObligation,
    transys::{unroll::TransysUnroll, Transys},
    IC3,
};
// use aig::AigEdge;
use logic_form::{Clause, Cube, Lemma, Lit};
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

    pub fn check_witness(&mut self, b: ProofObligation) -> Option<Lit> {
        let mut b = Some(b);
        while let Some(bad) = b {
            let imply = if let Some(next) = bad.next.clone() {
                self.ts.cube_next(&next.lemma)
            } else {
                self.ts.bad.clone()
            };
            let mut assump = bad.lemma.deref().clone();
            assump.extend_from_slice(&bad.input);
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
                .find(|l| self.lift.sat_value(**l).is_some_and(|v| !v))
            {
                return Some(*v);
            }
            if let Some(v) = self
                .ts
                .constraints
                .iter()
                .find(|l| self.lift.sat_value(**l).is_none())
            {
                return Some(*v);
            }
            b = bad.next.clone();
        }
        if self.options.verbose > 0 {
            println!("witness checking passed");
        }
        None
    }

    fn check_witness_with_constrain(
        &mut self,
        solver: &mut cadical::Solver,
        uts: &TransysUnroll,
        constrain: &Cube,
    ) -> bool {
        let mut assumps = Cube::new();
        for k in 0..=uts.num_unroll {
            assumps.extend_from_slice(&uts.lits_next(constrain, k));
        }
        assumps.extend_from_slice(&uts.lits_next(&uts.ts.bad, uts.num_unroll));
        solver.solve(&assumps)
    }

    pub fn check_witness_by_bmc(&mut self, b: ProofObligation) -> Option<Cube> {
        dbg!(b.depth);
        let mut uts = TransysUnroll::new(&self.ts);
        uts.unroll_to(b.depth);
        let mut solver = cadical::Solver::new();
        for k in 0..=b.depth {
            uts.load_trans(&mut solver, k, false);
        }
        uts.ts.load_init(&mut solver);
        let mut cst = uts.ts.constraints.clone();
        if self.check_witness_with_constrain(&mut solver, &uts, &cst) {
            if self.options.verbose > 0 {
                println!("witness checking passed");
            }
            None
        } else {
            let mut i = 0;
            while i < cst.len() {
                let mut drop = cst.clone();
                drop.remove(i);
                if self.check_witness_with_constrain(&mut solver, &uts, &drop) {
                    i += 1;
                } else {
                    cst = drop;
                }
            }
            Some(cst)
        }
    }
}
