use crate::{
    transys::{Transys, TransysUnroll},
    Options,
};
use aig::Aig;
use satif::Satif;

pub struct Kind {
    uts: TransysUnroll,
    options: Options,
}

impl Kind {
    pub fn new(args: Options) -> Self {
        let aig = Aig::from_file(&args.model);
        let (ts, _) = Transys::from_aig(&aig, true);
        let uts = TransysUnroll::new(&ts);
        Self { uts, options: args }
    }

    pub fn check(&mut self) -> bool {
        let step = self.options.step as usize;
        let mut solver = cadical::Solver::new();
        for k in (step - 1..).step_by(step) {
            self.uts.unroll_to(k);
            let kind_bound = k + 1 - step;
            self.uts.load_trans(&mut solver, kind_bound, true);
            if kind_bound > 0 {
                if self.options.verbose > 0 {
                    println!("kind depth: {kind_bound}");
                }
                if !solver.solve(&[self.uts.lit_next(self.uts.ts.bad, kind_bound)]) {
                    println!("k-induction proofed in depth {kind_bound}");
                    return true;
                }
            }
            for s in kind_bound + 1..=k {
                self.uts.load_trans(&mut solver, s, true);
            }
            if !self.options.kind_options.kind_no_bmc {
                let mut assump = self.uts.ts.init.clone();
                assump.push(self.uts.lit_next(self.uts.ts.bad, k));
                if self.options.verbose > 0 {
                    println!("kind bmc depth: {k}");
                }
                if solver.solve(&assump) {
                    if self.options.verbose > 0 {
                        println!("bmc found cex in depth {k}");
                    }
                    return false;
                }
            }
            for s in k + 1 - step..=k {
                solver.add_clause(&[!self.uts.lit_next(self.uts.ts.bad, s)]);
            }
        }
        unreachable!();
    }

    pub fn check_in_depth(&mut self, depth: usize) -> bool {
        println!("{}", self.options.model);
        assert!(depth > 0);
        let mut kind = kissat::Solver::new();
        self.uts.unroll_to(depth);
        for k in 0..=depth {
            self.uts.load_trans(&mut kind, k, true);
        }
        for k in 0..depth {
            kind.add_clause(&[!self.uts.lit_next(self.uts.ts.bad, k)]);
        }
        kind.add_clause(&[self.uts.lit_next(self.uts.ts.bad, depth)]);
        println!("kind depth: {depth}");
        if !kind.solve(&[]) {
            println!("kind proofed in depth {depth}");
            return true;
        }
        false
    }
}
