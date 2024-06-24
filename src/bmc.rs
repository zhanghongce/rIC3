use crate::Args;
use aig::Aig;
use kissat::Solver;
use satif::Satif;
use transys::{Transys, TransysUnroll};

pub struct BMC {
    uts: TransysUnroll,
    args: Args,
}

impl BMC {
    pub fn new(args: Args) -> Self {
        let aig = Aig::from_file(&args.model);
        let ts = Transys::from_aig(&aig);
        let uts = TransysUnroll::new(&ts);
        Self { uts, args }
    }

    pub fn check(&mut self) -> bool {
        println!("{}", self.args.model);
        let mut solver = Solver::new();
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
        let mut solver = Solver::new();
        self.uts.ts.load_init(&mut solver);
        for k in 0..=depth {
            self.uts.unroll_to(k);
            self.uts.load_trans(&mut solver, k);
            if k != depth {
                continue;
            }
            // if self.args.verbose {
            println!("bmc depth: {k}");
            // }
            let bad = self.uts.lit_next(self.uts.ts.bad, k);
            solver.add_clause(&[bad]);
            match solver.solve(&[]) {
                satif::SatResult::Sat(_) => {
                    println!("bmc found cex in depth {k}");
                    return true;
                }
                satif::SatResult::Unsat(_) => (),
            }
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
        for k in 140.. {
            if k % 10 == 0 {
                if self.check_in_depth(k) {
                    return true;
                }
            }
        }
        unreachable!()
    }
}
