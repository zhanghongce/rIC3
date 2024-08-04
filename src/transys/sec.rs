use super::unroll::TransysUnroll;
use crate::transys::Transys;
use logic_form::{Clause, Lit, Var};
use satif::Satif;
use std::collections::{HashMap, HashSet};

impl Transys {
    fn sec_with_bound(
        uts: &TransysUnroll,
        simulations: &HashMap<Var, u64>,
        avoid: &mut HashSet<Var>,
        eqs: &mut Vec<(Lit, Lit)>,
    ) {
        let mut solver = cadical::Solver::new();
        for k in 0..=uts.num_unroll {
            uts.load_trans(&mut solver, k, true);
            for (x, y) in eqs.iter() {
                solver.add_clause(&uts.lits_next(&Clause::from([*x, !*y]), k));
                solver.add_clause(&uts.lits_next(&Clause::from([!*x, *y]), k));
            }
        }
        for i in 0..uts.ts.latchs.len() {
            let x = uts.ts.latchs[i];
            if avoid.contains(&x) {
                continue;
            }
            for j in i + 1..uts.ts.latchs.len() {
                let y = uts.ts.latchs[j];
                if avoid.contains(&y) {
                    continue;
                }
                if uts.ts.init_map[x].is_some()
                    && uts.ts.init_map[x] == uts.ts.init_map[y]
                    && simulations[&x] == simulations[&y]
                {
                    let act = solver.new_var().lit();
                    let xl = x.lit();
                    let yl = y.lit();
                    for k in 0..=uts.num_unroll {
                        let kxl = uts.lit_next(xl, k);
                        let kyl = uts.lit_next(yl, k);
                        solver.add_clause(&[!act, kxl, !kyl]);
                        solver.add_clause(&[!act, !kxl, kyl]);
                    }
                    let nxl = uts.lit_next(xl, uts.num_unroll + 1);
                    let nyl = uts.lit_next(yl, uts.num_unroll + 1);
                    if !solver.solve(&[act, nxl, !nyl]) && !solver.solve(&[act, !nxl, nyl]) {
                        eqs.push((xl, yl));
                        avoid.insert(x);
                        avoid.insert(y);
                        dbg!(x, y);
                        solver.add_clause(&[act]);
                        break;
                    } else {
                        solver.add_clause(&[!act]);
                    }
                }
            }
        }
    }

    pub fn sec(&self) -> Vec<(Lit, Lit)> {
        let mut eqs = Vec::new();
        let mut avoid = HashSet::new();
        let simulations = self.simulations();
        let Some(simulations) = self.simulation_bv(simulations) else {
            return eqs;
        };
        let mut uts = TransysUnroll::new(self);
        Self::sec_with_bound(&uts, &simulations, &mut avoid, &mut eqs);
        dbg!(eqs.len());
        uts.unroll();
        // Self::sec_with_bound(&uts, &simulations, &mut avoid, &mut eqs);
        // dbg!(eqs.len());
        eqs
    }
}
