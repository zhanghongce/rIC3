use super::{proofoblig::ProofObligation, IC3};
use crate::transys::{unroll::TransysUnroll, Transys};
use logic_form::{Clause, Cube, Lemma, Lit};
use satif::Satif;
use satif_minisat::Solver;
use std::{io::Write, fs::File, ops::Deref};

pub fn verify_invariant(ts: &Transys, invariants: &[Lemma]) -> bool {
    let mut solver = Solver::new();
    solver.new_var_to(ts.max_var);
    for cls in ts.trans.iter() {
        solver.add_clause(cls)
    }
    for lemma in invariants {
        solver.add_clause(&!lemma.deref());
    }
    for c in ts.constraints.iter() {
        solver.add_clause(&Clause::from([*c]));
    }
    if solver.solve(&ts.bad.cube()) {
        return false;
    }
    for lemma in invariants {
        let mut assump = ts.constraints.clone();
        assump.push(ts.bad);
        if solver.solve(&ts.cube_next(lemma)) {
            return false;
        }
    }
    true
}

impl IC3 {
    pub fn verify(&mut self) {
        if !self.options.certify {
            return;
        }
        let invariants = self.frame.invariant();
        if !verify_invariant(&self.ts, &invariants) {
            panic!("invariant varify failed");
        }
        if self.options.verbose > 0 {
            println!(
                "inductive invariant verified with {} lemmas!",
                invariants.len()
            );
        }
        let mut file = File::create("inv.cnf").expect("Unable to create inv.cnf");
        writeln!(&mut file, "{}", invariants.len()).expect("Failed to write to file");
        for clause in invariants.iter() {
            for lit in clause.cube().iter() {
                write!(&mut file, "{} ", lit).expect("Failed to write to file");
            }
            writeln!(&mut file,"").expect("Failed to write to file");
        }
    }

    pub fn check_witness(&mut self) -> Option<Lit> {
        let mut b = self.obligations.peak();
        while let Some(bad) = b {
            let imply = if let Some(next) = bad.next.clone() {
                self.ts.cube_next(&next.lemma)
            } else {
                self.ts.bad.cube()
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
                .chain(self.ts.constraints.iter())
                .all(|l| self.lift.sat_value(*l).is_some_and(|v| v)));
            b = bad.next.clone();
        }
        if self.options.verbose > 0 {
            println!("witness checking passed");
        }
        None
    }

    fn check_witness_with_constrain<S: Satif + ?Sized>(
        &mut self,
        solver: &mut S,
        uts: &TransysUnroll,
        constraint: &Cube,
    ) -> bool {
        let mut assumps = Cube::new();
        for k in 0..=uts.num_unroll {
            assumps.extend_from_slice(&uts.lits_next(constraint, k));
        }
        assumps.push(uts.lit_next(uts.ts.bad, uts.num_unroll));
        solver.solve(&assumps)
    }

    pub fn check_witness_by_bmc(&mut self, b: ProofObligation) -> Option<Cube> {
        let mut uts = TransysUnroll::new(&self.ts);
        uts.unroll_to(b.depth);
        let mut solver: Box<dyn satif::Satif> = Box::new(satif_cadical::Solver::new());
        for k in 0..=b.depth {
            uts.load_trans(solver.as_mut(), k, false);
        }
        uts.ts.load_init(solver.as_mut());
        let mut cst = uts.ts.constraints.clone();
        if self.check_witness_with_constrain(solver.as_mut(), &uts, &cst) {
            if self.options.verbose > 0 {
                println!("witness checking passed");
            }
            self.bmc_solver = Some((solver, uts));
            None
        } else {
            let mut i = 0;
            while i < cst.len() {
                if self.abs_cst.contains(&cst[i]) {
                    i += 1;
                    continue;
                }
                let mut drop = cst.clone();
                drop.remove(i);
                if self.check_witness_with_constrain(solver.as_mut(), &uts, &drop) {
                    i += 1;
                } else {
                    cst = drop;
                }
            }
            cst.retain(|l| !self.abs_cst.contains(l));
            assert!(!cst.is_empty());
            Some(cst)
        }
    }
}
