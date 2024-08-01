use crate::{
    transys::{Transys, TransysUnroll},
    Options,
};
use logic_form::{
    dimacs::{from_dimacs_str, to_dimacs},
    Clause,
};
use satif::Satif;
use std::{
    io::Write,
    process::{Command, Stdio},
};

pub struct BMC {
    uts: TransysUnroll,
    options: Options,
}

impl BMC {
    pub fn new(options: Options, ts: Transys) -> Self {
        let uts = TransysUnroll::new(&ts);
        Self { uts, options }
    }

    fn check_with_cadical(&mut self) -> bool {
        let mut solver = cadical::Solver::new();
        let step = self.options.step as usize;
        for k in (step - 1..).step_by(step) {
            self.uts.unroll_to(k);
            let last_bound = k + 1 - step;
            for s in last_bound..=k {
                self.uts.load_trans(&mut solver, s, true);
            }
            let mut assump = self.uts.ts.init.clone();
            assump.push(self.uts.lit_next(self.uts.ts.bad, k));
            if self.options.verbose > 0 {
                println!("bmc depth: {k}");
            }
            if solver.solve(&assump) {
                if self.options.verbose > 0 {
                    println!("bmc found cex in depth {k}");
                }
                return false;
            }
            // for s in last_bound..=k {
            //     solver.add_clause(&[!self.uts.lit_next(self.uts.ts.bad, s)]);
            // }
        }
        unreachable!();
    }

    fn check_with_kissat(&mut self) -> bool {
        let step = self.options.step as usize;
        for k in (step..).step_by(step) {
            let mut solver = kissat::Solver::new();
            self.uts.ts.load_init(&mut solver);
            self.uts.unroll_to(k);
            for k in 0..=k {
                self.uts.load_trans(&mut solver, k, true);
            }
            if self.options.verbose > 0 {
                println!("bmc depth: {k}");
            }
            solver.add_clause(&[self.uts.lit_next(self.uts.ts.bad, k)]);
            if solver.solve(&[]) {
                if self.options.verbose > 0 {
                    println!("bmc found cex in depth {k}");
                }
                return false;
            }
        }
        unreachable!()
    }

    pub fn check(&mut self) -> bool {
        if self.options.bmc_options.kissat {
            self.check_with_kissat()
        } else {
            self.check_with_cadical()
        }
    }
}

pub fn sbva(cnf: &[Clause]) -> Vec<Clause> {
    let mut command = Command::new("../SBVA/sbva");
    let mut sbva = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let stdin = sbva.stdin.as_mut().unwrap();
    let cnf = to_dimacs(cnf);
    stdin.write_all(cnf.as_bytes()).unwrap();
    let out = sbva.wait_with_output().unwrap();
    let simp = String::from_utf8(out.stdout).unwrap();
    from_dimacs_str(&simp)
}
