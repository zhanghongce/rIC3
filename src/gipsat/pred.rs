use super::Solver;
use logic_form::{Clause, Cube, Lit, Var};
use rand::seq::SliceRandom;
use std::collections::HashSet;

impl Solver {
    #[inline]
    pub fn minimal_pred(
        &mut self,
        inputs: &[Lit],
        latchs: &[Lit],
        target_constrain: &Clause,
    ) -> Option<Cube> {
        let assump = Cube::from_iter(inputs.iter().chain(latchs.iter()).copied());
        if self
            .solve_with_domain(&assump, vec![target_constrain.clone()], true, false)
            .unwrap()
        {
            return None;
        }
        Some(
            latchs
                .iter()
                .filter(|l| self.unsat_has(**l))
                .copied()
                .collect(),
        )
    }

    pub fn get_pred(&mut self, solver: &mut Solver, strengthen: bool) -> (Cube, Cube) {
        let mut cls: Cube = solver.assump.clone();
        cls.extend_from_slice(&self.ts.constraints);
        if cls.is_empty() {
            return (Cube::new(), Cube::new());
        }
        let in_cls: HashSet<Var> = HashSet::from_iter(cls.iter().map(|l| l.var()));
        let cls = !cls;
        let mut inputs = Cube::new();
        for input in self.ts.inputs.iter() {
            let lit = input.lit();
            if let Some(v) = solver.sat_value(lit) {
                inputs.push(lit.not_if(!v));
            }
        }
        self.set_domain(cls.iter().cloned());
        let mut latchs = Cube::new();
        for latch in self.ts.latchs.iter() {
            let lit = latch.lit();
            if self.domain.has(lit.var()) {
                if let Some(v) = solver.sat_value(lit) {
                    if in_cls.contains(latch) || !solver.flip_to_none(*latch) {
                        latchs.push(lit.not_if(!v));
                    }
                }
            }
        }
        for _ in 0.. {
            if latchs.is_empty() {
                break;
            }
            latchs.shuffle(&mut self.rng);
            let olen = latchs.len();
            latchs = self.minimal_pred(&inputs, &latchs, &cls).unwrap();
            if latchs.len() == olen || !strengthen {
                break;
            }
        }
        self.unset_domain();
        (latchs, inputs)
    }
}
