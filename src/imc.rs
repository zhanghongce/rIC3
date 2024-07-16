use crate::Args;
use aig::Aig;
use cadical::craig::{ClauseLabel, Craig, VarLabel};
use logic_form::Lit;
use satif::{SatResult, Satif};
use transys::{Transys, TransysUnroll};

pub struct IMC {
    uts: TransysUnroll,
    args: Args,
}

impl IMC {
    pub fn new(args: Args) -> Self {
        let aig = Aig::from_file(&args.model);
        let (ts, _) = Transys::from_aig(&aig);
        let uts = TransysUnroll::new(&ts);
        Self { uts, args }
    }

    pub fn check(&mut self) -> bool {
        println!("{}", self.args.model);
        for k in 0.. {
            let mut solver = cadical::Solver::new();
            let mut craig = Craig::new(&mut solver);
            self.uts.unroll_to(k);
            for l in self.uts.ts.latchs.iter() {
                craig.label_var(self.uts.lit_next(l.lit(), k).var(), VarLabel::Global);
            }
            for i in self.uts.ts.init.iter() {
                craig.label_clause(ClauseLabel::A);
                solver.add_clause(&[*i]);
            }
            for u in 0..=k {
                for c in self.uts.ts.trans.iter() {
                    let c: Vec<Lit> = c.iter().map(|l| self.uts.lit_next(*l, u)).collect();
                    if u < k {
                        craig.label_clause(ClauseLabel::A);
                    } else {
                        craig.label_clause(ClauseLabel::B);
                    }
                    solver.add_clause(&c);
                }
                for c in self.uts.ts.constraints.iter() {
                    let c = self.uts.lit_next(*c, u);
                    if u < k {
                        craig.label_clause(ClauseLabel::A);
                    } else {
                        craig.label_clause(ClauseLabel::B);
                    }
                    solver.add_clause(&[c]);
                }
            }
            if self.args.verbose {
                println!("bmc depth: {k}");
            }
            let bad = self.uts.lit_next(self.uts.ts.bad, k);
            craig.label_clause(ClauseLabel::B);
            solver.add_clause(&[bad]);
            match solver.solve(&[]) {
                SatResult::Sat(_) => {
                    println!("bmc found cex in depth {k}");
                    return true;
                }
                SatResult::Unsat(_) => {
                    let itp = craig.interpolant(10000000000);
                    dbg!(itp);
                }
            }
        }
        unreachable!();
    }
}
