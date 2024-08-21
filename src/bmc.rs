use crate::{
    transys::{unroll::TransysUnroll, Transys},
    Engine, Options,
};
use satif::Satif;
use std::time::Duration;

pub struct BMC {
    uts: TransysUnroll,
    options: Options,
}

impl BMC {
    pub fn new(options: Options, ts: Transys) -> Self {
        let uts = TransysUnroll::new(&ts);
        Self { uts, options }
    }

    fn check_with_cadical(&mut self) -> Option<bool> {
        let mut solver = cadical::Solver::new();
        let step = self.options.step as usize;
        for k in (step - 1..).step_by(step) {
            self.uts.unroll_to(k);
            let last_bound = k + 1 - step;
            for s in last_bound..=k {
                self.uts.load_trans(&mut solver, s, true);
            }
            let mut assump = self.uts.ts.init.clone();
            assump.extend_from_slice(&self.uts.lits_next(&self.uts.ts.bad, k));
            if self.options.verbose > 0 {
                println!("bmc depth: {k}");
            }
            if solver.solve(&assump) {
                if self.options.verbose > 0 {
                    println!("bmc found cex in depth {k}");
                }
                return Some(false);
            }
            // for s in last_bound..=k {
            //     solver.add_clause(&[!self.uts.lit_next(self.uts.ts.bad, s)]);
            // }
        }
        unreachable!();
    }

    fn check_with_kissat(&mut self) -> Option<bool> {
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
            for b in self.uts.lits_next(&self.uts.ts.bad, k) {
                solver.add_clause(&[b]);
            }
            let r = if let Some(limit) = self.options.bmc_options.time_limit {
                let Some(r) = solver.solve_with_limit(&[], Duration::from_secs(limit)) else {
                    if self.options.verbose > 0 {
                        println!("bmc solve timeout in depth {k}");
                    }
                    continue;
                };
                r
            } else {
                solver.solve(&[])
            };
            if r {
                if self.options.verbose > 0 {
                    println!("bmc found cex in depth {k}");
                }
                return Some(false);
            }
        }
        unreachable!()
    }
}

impl Engine for BMC {
    fn check(&mut self) -> Option<bool> {
        if self.options.bmc_options.kissat {
            self.check_with_kissat()
        } else {
            self.check_with_cadical()
        }
    }
}
