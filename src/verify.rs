use crate::{
    proofoblig::ProofObligation,
    transys::{unroll::TransysUnroll, Transys},
    Engine, Options, IC3,
};
use aig::Aig;
// use aig::AigEdge;
use logic_form::{Clause, Cube, Lemma, Lit, Var};
use minisat::Solver;
use satif::Satif;
use std::{
    // io::{self, Write},
    collections::HashMap,
    fs::File,
    io::{self, Write},
    ops::Deref,
    process::Command, // process::Command,
};

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
    pub fn verify(&mut self) {
        if !self.options.verify {
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
                if self.abs_cst.contains(&cst[i]) {
                    i += 1;
                    continue;
                }
                let mut drop = cst.clone();
                drop.remove(i);
                if self.check_witness_with_constrain(&mut solver, &uts, &drop) {
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

pub fn check_certifaiger(engine: &mut Box<dyn Engine>, aig: &Aig, option: &Options) {
    if option.verify_path.is_none() && !option.verify {
        return;
    }
    let certifaiger = engine.certifaiger(&aig);
    if let Some(witness) = &option.verify_path {
        certifaiger.to_file(witness, true);
    }
    if !option.verify {
        return;
    }
    let certifaiger_file = tempfile::NamedTempFile::new().unwrap();
    let certifaiger_path = certifaiger_file.path().as_os_str().to_str().unwrap();
    certifaiger.to_file(certifaiger_path, true);
    let output = Command::new("/root/certifaiger/build/check")
        .arg(&option.model)
        .arg(certifaiger_path)
        .output()
        .expect("certifaiger not found");
    io::stdout().write_all(&output.stdout).unwrap();
    if output.status.success() {
        println!("certifaiger check passed");
    } else {
        panic!("certifaiger check failed");
    }
}

pub fn check_witness(engine: &mut Box<dyn Engine>, aig: &Aig, option: &Options) {
    if option.verify_path.is_none() && !option.verify {
        return;
    }
    let witness = engine.witness();
    if let Some(witness_file) = &option.verify_path {
        let mut file = File::create(witness_file).unwrap();
        file.write_all(b"1\n").unwrap();
        file.write_all(b"b0\n").unwrap();
        let map: HashMap<Var, bool> =
            HashMap::from_iter(witness[0].iter().map(|l| (l.var(), l.polarity())));
        let mut line = String::new();
        for l in aig.latchs.iter() {
            line.push(if let Some(r) = map.get(&Var::new(l.input)) {
                if *r {
                    '1'
                } else {
                    '0'
                }
            } else {
                'x'
            })
        }
        line.push('\n');
        file.write_all(line.as_bytes()).unwrap();
        for c in witness[1..].iter() {
            let map: HashMap<Var, bool> =
                HashMap::from_iter(c.iter().map(|l| (l.var(), l.polarity())));
            let mut line = String::new();
            for l in aig.inputs.iter() {
                line.push(if let Some(r) = map.get(&Var::new(*l)) {
                    if *r {
                        '1'
                    } else {
                        '0'
                    }
                } else {
                    'x'
                })
            }
            line.push('\n');
            file.write_all(line.as_bytes()).unwrap();
        }
        file.write_all(b".\n").unwrap();
    }
}
