use crate::Options;
use aig::Aig;
use logic_form::{
    dimacs::{from_dimacs_str, to_dimacs},
    Clause,
};
use satif::Satif;
use std::{
    io::Write,
    process::{Command, Stdio},
};
use transys::{Transys, TransysUnroll};

pub struct BMC {
    uts: TransysUnroll,
    args: Options,
}

impl BMC {
    pub fn new(args: Options) -> Self {
        let aig = Aig::from_file(&args.model);
        let (ts, _) = Transys::from_aig(&aig);
        let uts = TransysUnroll::new(&ts);
        Self { uts, args }
    }

    pub fn check(&mut self) -> bool {
        let mut solver = cadical::Solver::new();
        self.uts.ts.load_init(&mut solver);
        for k in 0.. {
            self.uts.unroll_to(k);
            self.uts.load_trans(&mut solver, k);
            if self.args.verbose {
                println!("bmc depth: {k}");
            }
            let bad = self.uts.lit_next(self.uts.ts.bad, k);
            match solver.solve(&[bad]) {
                satif::SatResult::Sat(_) => {
                    println!("bmc found cex in depth {k}");
                    return true;
                }
                satif::SatResult::Unsat(_) => (),
            }
        }
        unreachable!();
    }

    pub fn check_in_depth(&mut self, depth: usize) -> bool {
        println!("{}", self.args.model);
        let mut solver = kissat::Solver::new();
        self.uts.ts.load_init(&mut solver);
        self.uts.unroll_to(depth);
        for k in 0..=depth {
            self.uts.load_trans(&mut solver, k);
        }
        println!("bmc depth: {depth}");
        solver.add_clause(&[self.uts.lit_next(self.uts.ts.bad, depth)]);
        if let satif::SatResult::Sat(_) = solver.solve(&[]) {
            println!("bmc found cex in depth {depth}");
            return true;
        }
        false
    }

    pub fn check_no_incremental(&mut self) -> bool {
        if self.check_in_depth(70) {
            return true;
        }
        if self.check_in_depth(130) {
            return true;
        }
        for k in (140..).step_by(50) {
            if self.check_in_depth(k) {
                return true;
            }
        }
        unreachable!()
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
