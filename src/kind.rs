use crate::{
    transys::{unroll::TransysUnroll, Transys},
    verify::witness_encode,
    Engine, Options,
};
use aig::{Aig, AigEdge};
use logic_form::{Clause, Cube};
use satif::Satif;

pub struct Kind {
    uts: TransysUnroll,
    options: Options,
    solver: cadical::Solver,
    pre_lemmas: Vec<Clause>,
}

impl Kind {
    pub fn new(options: Options, ts: Transys, pre_lemmas: Vec<Clause>) -> Self {
        let uts = TransysUnroll::new(&ts);
        let solver = cadical::Solver::new();
        Self {
            uts,
            options,
            pre_lemmas,
            solver,
        }
    }

    fn load_pre_lemmas(&mut self, k: usize) {
        for cls in self.pre_lemmas.iter() {
            let cls: Clause = self.uts.lits_next(cls, k);
            self.solver.add_clause(&cls);
        }
    }

    // pub fn check_in_depth(&mut self, depth: usize) -> bool {
    //     println!("{}", self.options.model);
    //     assert!(depth > 0);
    //     let mut solver = kissat::Solver::new();
    //     self.uts.unroll_to(depth);
    //     for k in 0..=depth {
    //         self.uts.load_trans(&mut solver, k, true);
    //     }
    //     for k in 0..depth {
    //         solver.add_clause(&!self.uts.lits_next(&self.uts.ts.bad, k));
    //         self.load_pre_lemmas(&mut solver, k);
    //     }
    //     for b in self.uts.lits_next(&self.uts.ts.bad, depth).iter() {
    //         solver.add_clause(&[*b]);
    //     }
    //     println!("kind depth: {depth}");
    //     if !solver.solve(&[]) {
    //         println!("kind proofed in depth {depth}");
    //         return true;
    //     }
    //     false
    // }
}

impl Engine for Kind {
    fn check(&mut self) -> Option<bool> {
        let step = self.options.step as usize;
        for k in (step - 1..).step_by(step) {
            self.uts.unroll_to(k);
            let kind_bound = k + 1 - step;
            self.uts.load_trans(&mut self.solver, kind_bound, true);
            self.load_pre_lemmas(kind_bound);
            if kind_bound > 0 {
                if self.options.verbose > 0 {
                    println!("kind depth: {kind_bound}");
                }
                if !self
                    .solver
                    .solve(&self.uts.lits_next(&self.uts.ts.bad, kind_bound))
                {
                    println!("k-induction proofed in depth {kind_bound}");
                    return Some(true);
                }
            }
            for s in kind_bound + 1..=k {
                self.uts.load_trans(&mut self.solver, s, true);
                self.load_pre_lemmas(s);
            }
            if !self.options.kind_options.no_bmc {
                let mut assump = self.uts.ts.init.clone();
                assump.extend_from_slice(&self.uts.lits_next(&self.uts.ts.bad, k));
                if self.options.verbose > 0 {
                    println!("kind bmc depth: {k}");
                }
                if self.solver.solve(&assump) {
                    if self.options.verbose > 0 {
                        println!("bmc found cex in depth {k}");
                    }
                    return Some(false);
                }
            }
            for s in k + 1 - step..=k {
                self.solver
                    .add_clause(&!self.uts.lits_next(&self.uts.ts.bad, s));
            }
        }
        unreachable!();
    }

    fn witness(&mut self, aig: &Aig) -> String {
        let mut wit = vec![Cube::new()];
        for l in self.uts.ts.latchs.iter() {
            let l = l.lit();
            if let Some(v) = self.solver.sat_value(l) {
                wit[0].push(self.uts.ts.restore(l.not_if(!v)));
            }
        }
        for k in 0..=self.uts.num_unroll {
            let mut w = Cube::new();
            for l in self.uts.ts.inputs.iter() {
                let l = l.lit();
                let kl = self.uts.lit_next(l, k);
                if let Some(v) = self.solver.sat_value(kl) {
                    w.push(self.uts.ts.restore(l.not_if(!v)));
                }
            }
            wit.push(w);
        }
        witness_encode(aig, &wit)
    }
}
