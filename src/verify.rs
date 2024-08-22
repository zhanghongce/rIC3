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
        if self.options.not_certify {
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

    pub fn check_witness(&mut self) -> Option<Lit> {
        let mut b = self.obligations.peak();
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
                .chain(self.ts.constraints.iter())
                .all(|l| self.lift.sat_value(*l).is_some_and(|v| v)));
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
    if option.certify_path.is_none() && option.not_certify {
        return;
    }
    let certifaiger = engine.certifaiger(&aig);
    if let Some(witness) = &option.certify_path {
        certifaiger.to_file(witness, true);
    }
    if option.not_certify {
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

pub fn witness_encode(aig: &Aig, witness: &[Cube]) -> String {
    let mut wit = Vec::new();
    wit.push("1".to_string());
    wit.push("b0".to_string());
    let map: HashMap<Var, bool> =
        HashMap::from_iter(witness[0].iter().map(|l| (l.var(), l.polarity())));
    let mut line = String::new();
    for l in aig.latchs.iter() {
        let r = if let Some(r) = l.init {
            r
        } else if let Some(r) = map.get(&Var::new(l.input)) {
            *r
        } else {
            true
        };
        line.push(if r { '1' } else { '0' })
    }
    wit.push(line);
    for c in witness[1..].iter() {
        let map: HashMap<Var, bool> = HashMap::from_iter(c.iter().map(|l| (l.var(), l.polarity())));
        let mut line = String::new();
        for l in aig.inputs.iter() {
            let r = if let Some(r) = map.get(&Var::new(*l)) {
                *r
            } else {
                true
            };
            line.push(if r { '1' } else { '0' })
        }
        wit.push(line);
    }
    wit.push(".\n".to_string());
    wit.join("\n")
}

pub fn check_witness(engine: &mut Box<dyn Engine>, aig: &Aig, option: &Options) {
    if option.certify_path.is_none() && option.not_certify {
        return;
    }
    let witness = engine.witness(aig);
    if let Some(witness_file) = &option.certify_path {
        let mut file: File = File::create(witness_file).unwrap();
        file.write_all(witness.as_bytes()).unwrap();
    }
    if option.not_certify {
        return;
    }
    let mut wit_file = tempfile::NamedTempFile::new().unwrap();
    wit_file.write_all(witness.as_bytes()).unwrap();
    let wit_path = wit_file.path().as_os_str().to_str().unwrap();
    let output = Command::new("/root/certifaiger/build/simulate")
        .arg(&option.model)
        .arg(wit_path)
        .output()
        .expect("certifaiger not found");
    io::stdout().write_all(&output.stdout).unwrap();
    if output.status.success() {
        println!("certifaiger check passed");
    } else {
        panic!("certifaiger check failed");
    }
}
