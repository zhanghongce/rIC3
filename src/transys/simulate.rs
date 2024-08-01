use std::collections::HashMap;

use crate::{transys::unroll::TransysUnroll, Transys};
use cadical::Solver;
use logic_form::{Cube, Var};
use satif::Satif;

impl Transys {
    pub fn simulations(&self) -> Vec<Cube> {
        let mut uts = TransysUnroll::new(self);
        let depth = 5;
        uts.unroll_to(depth);
        let mut solver = Solver::new();
        self.load_init(&mut solver);
        for k in 0..=depth {
            uts.load_trans(&mut solver, k, true);
        }
        let mut res = vec![];
        let ninit: Cube = uts.lits_next(&self.init, depth + 1);
        solver.add_clause(&!&ninit);
        while res.len() < 64 {
            if !solver.solve(&[]) {
                break;
            };
            let mut cube = Cube::new();
            for l in self.latchs.iter() {
                let l = l.lit();
                let nl = uts.lit_next(l, depth + 1);
                if let Some(v) = solver.sat_value(nl) {
                    cube.push(l.not_if(!v));
                    solver.set_polarity(nl.var(), Some(!v))
                }
            }
            for r in res.iter().skip(1) {
                let its = cube.intersection(r);
                let nits: Cube = uts.lits_next(&its, depth + 1);
                solver.add_clause(&!&nits);
            }
            let ncube: Cube = uts.lits_next(&cube, depth + 1);
            solver.add_clause(&!&ncube);
            res.push(cube);
        }
        println!("{:?}", res.len());
        res
    }

    pub fn simulation_bv(&self, simulation: Vec<Cube>) -> Option<HashMap<Var, u64>> {
        let mut bv = HashMap::new();
        for v in self.latchs.iter() {
            bv.insert(*v, 0);
        }
        for (i, s) in simulation.into_iter().enumerate() {
            if s.len() != self.latchs.len() {
                return None;
            }
            for l in s.iter() {
                let bv = bv.get_mut(&l.var()).unwrap();
                if l.polarity() {
                    *bv = *bv | (1 << i);
                }
            }
        }
        Some(bv)
    }
}
