use crate::Args;
use aig::Aig;
use satif::{SatResult, Satif};
use transys::{Transys, TransysUnroll};

pub struct Kind {
    uts: TransysUnroll,
    args: Args,
}

impl Kind {
    pub fn new(args: Args) -> Self {
        let aig = Aig::from_file(&args.model);
        let (ts, _) = Transys::from_aig(&aig);
        let uts = TransysUnroll::new(&ts);
        Self { uts, args }
    }

    pub fn check(&mut self) -> bool {
        println!("{}", self.args.model);
        let mut solver = cadical::Solver::new();
        let mut ind = cadical::Solver::new();
        self.uts.ts.load_init(&mut solver);
        for k in 0.. {
            self.uts.unroll_to(k);
            self.uts.load_trans(&mut solver, k);
            self.uts.load_trans(&mut ind, k);
            let bad = self.uts.lit_next(self.uts.ts.bad, k);
            if k > 0 {
                if self.args.verbose {
                    println!("kind depth: {k}");
                }
                if let SatResult::Unsat(_) = ind.solve(&[bad]) {
                    println!("k-induction proofed in depth {k}");
                    return true;
                }
            }
            if self.args.verbose {
                println!("bmc depth: {k}");
            }
            match solver.solve(&[bad]) {
                satif::SatResult::Sat(_) => {
                    println!("bmc found cex in depth {k}");
                    return false;
                }
                satif::SatResult::Unsat(_) => {
                    ind.add_clause(&[!bad]);
                }
            }
        }
        unreachable!();
    }
}
