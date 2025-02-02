use super::IC3;
use crate::gipsat::ClauseKind;
use logic_form::{Clause, Cube, Lemma, Lit, Var};
use rand::seq::SliceRandom;
use std::{collections::HashSet, time::Instant};

impl IC3 {
    pub fn get_bad(&mut self) -> Option<(Cube, Cube)> {
        self.statistic.num_get_bad += 1;
        let start = Instant::now();
        let solver = self.solvers.last_mut().unwrap();
        let res = solver.solve_without_bucket(&self.ts.bad.cube(), vec![]);
        self.statistic.block_get_bad_time += start.elapsed();
        res.then(|| self.get_pred(self.solvers.len(), true))
    }
}

impl IC3 {
    #[inline]
    pub fn sat_contained(&mut self, frame: usize, lemma: &Lemma) -> bool {
        !self.solvers[frame].solve(lemma, vec![])
    }

    pub fn blocked_with_ordered(
        &mut self,
        frame: usize,
        cube: &Cube,
        ascending: bool,
        strengthen: bool,
    ) -> bool {
        let mut ordered_cube = cube.clone();
        self.activity.sort_by_activity(&mut ordered_cube, ascending);
        self.solvers[frame - 1].inductive(&ordered_cube, strengthen)
    }

    pub fn blocked_with_ordered_with_constrain(
        &mut self,
        frame: usize,
        cube: &Cube,
        ascending: bool,
        strengthen: bool,
        constraint: Vec<Clause>,
    ) -> bool {
        let mut ordered_cube = cube.clone();
        self.activity.sort_by_activity(&mut ordered_cube, ascending);
        self.solvers[frame - 1].inductive_with_constrain(&ordered_cube, strengthen, constraint)
    }

    pub fn get_pred(&mut self, frame: usize, strengthen: bool) -> (Cube, Cube) {
        let start = Instant::now();
        let solver = &mut self.solvers[frame - 1];
        let mut cls: Cube = solver.get_last_assump().clone();
        cls.extend_from_slice(&self.abs_cst);
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
        self.lift.set_domain(cls.iter().cloned());
        let mut latchs = Cube::new();
        for latch in self.ts.latchs.iter() {
            let lit = latch.lit();
            if self.lift.domain.has(lit.var()) {
                if let Some(v) = solver.sat_value(lit) {
                    if in_cls.contains(latch) || !solver.flip_to_none(*latch) {
                        latchs.push(lit.not_if(!v));
                    }
                }
            }
        }
        let inn: Box<dyn FnMut(&mut Cube)> = Box::new(|cube: &mut Cube| {
            cube.sort();
            cube.reverse();
        });
        let act: Box<dyn FnMut(&mut Cube)> = Box::new(|cube: &mut Cube| {
            self.activity.sort_by_activity(cube, false);
        });
        let rev: Box<dyn FnMut(&mut Cube)> = Box::new(|cube: &mut Cube| {
            cube.reverse();
        });
        let mut order = if self.options.ic3.inn || !self.auxiliary_var.is_empty() {
            vec![inn, act, rev]
        } else {
            vec![act, rev]
        };
        for i in 0.. {
            if latchs.is_empty() {
                break;
            }
            if let Some(f) = order.get_mut(i) {
                f(&mut latchs);
            } else {
                latchs.shuffle(&mut self.rng);
            }
            let olen = latchs.len();
            latchs = self.lift.minimal_pred(&inputs, &latchs, &cls).unwrap();
            if latchs.len() == olen || !strengthen {
                break;
            }
        }
        self.lift.unset_domain();
        self.statistic.block_get_predecessor_time += start.elapsed();
        (latchs, inputs)
    }

    pub fn new_var(&mut self) -> Var {
        let var = self.ts.new_var();
        for s in self.solvers.iter_mut() {
            assert!(var == s.new_var());
        }
        assert!(var == self.lift.new_var());
        var
    }

    pub fn add_latch(
        &mut self,
        state: Var,
        next: Lit,
        init: Option<bool>,
        mut trans: Vec<Clause>,
        dep: Vec<Var>,
    ) {
        for i in 0..trans.len() {
            let mut nt = Clause::new();
            for l in trans[i].iter() {
                nt.push(if l.var() == state {
                    next.not_if(!l.polarity())
                } else {
                    self.ts.lit_next(*l)
                });
            }
            trans.push(nt);
        }
        self.ts
            .add_latch(state, next, init, trans.clone(), dep.clone());
        let tmp_lit_set = &mut self.frame.tmp_lit_set;
        tmp_lit_set.reserve(self.ts.max_latch);
        for s in self.solvers.iter_mut().chain(Some(&mut self.lift)) {
            s.reset();
            for cls in trans.iter() {
                s.add_clause_inner(cls, ClauseKind::Trans);
            }
            s.add_domain(state, true);
        }
        if self.solvers[0].sat_value(state.lit()).is_some() {
            if self.solvers[0].sat_value(state.lit()).unwrap() {
                self.ts.init.push(state.lit());
                self.ts.init_map[state] = Some(true);
            } else {
                self.ts.init.push(!state.lit());
                self.ts.init_map[state] = Some(false);
            }
        } else if !self.solvers[0].solve(&[state.lit()], vec![]) {
            self.ts.init.push(!state.lit());
            self.ts.init_map[state] = Some(false);
        } else if !self.solvers[0].solve(&[!state.lit()], vec![]) {
            self.ts.init.push(state.lit());
            self.ts.init_map[state] = Some(true);
        }
        self.activity.reserve(state);
        self.auxiliary_var.push(state);
    }
}
