use crate::Args;
use aig::Aig;
use cadical::Solver;
use satif::Satif;
use transys::{Transys, TransysUnroll};

pub struct BMC {
    uts: TransysUnroll,
    solver: Solver,
    args: Args,
}

impl BMC {
    pub fn new(args: Args) -> Self {
        let aig = Aig::from_file(&args.model);
        let ts = Transys::from_aig(&aig);
        let uts = TransysUnroll::new(&ts);
        let solver = Solver::new();
        Self { uts, solver, args }
    }

    pub fn check(&mut self) -> bool {
        println!("{}", self.args.model);
        self.uts.ts.load_init(&mut self.solver);
        for k in 0.. {
            self.uts.unroll_to(k);
            self.uts.load_trans(&mut self.solver, k);
            if !(k == 70 || k == 130 || (k >= 140 && k % 10 == 0)) {
                continue;
            }
            if self.args.verbose {
                println!("bmc depth: {k}");
            }
            let bad = self.uts.lit_next(self.uts.ts.bad, k);
            match self.solver.solve(&[bad]) {
                satif::SatResult::Sat(_) => {
                    println!("bmc found cex in depth {k}");
                    return false;
                }
                satif::SatResult::Unsat(_) => (),
            }
        }
        unreachable!();
    }
}
