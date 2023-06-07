use super::basic::BasicShare;
use crate::utils::{generalize::generalize_by_ternary_simulation, relation::cube_subsume_init};
use aig::AigCube;
use logic_form::{Clause, Cube, Lit};
use sat_solver::{
    minisat::{Conflict, Model, Solver},
    SatModel, SatResult, SatSolver, UnsatConflict,
};
use std::{mem::take, sync::Arc, time::Instant};

pub struct PdrSolver {
    pub solver: Solver,
    num_act: usize,

    share: Arc<BasicShare>,
}

impl PdrSolver {
    pub fn new(share: Arc<BasicShare>) -> Self {
        let mut solver = Solver::new();
        solver.set_random_seed(91648253_f64);
        solver.add_cnf(&share.as_ref().transition_cnf);
        Self {
            solver,
            num_act: 0,
            share,
        }
    }

    pub fn pump_act_and_check_restart(&mut self, frames: &[Vec<Cube>]) {
        self.num_act += 1;
        if self.num_act > 300 {
            *self = Self::new(self.share.clone());
            for dnf in frames.iter() {
                for cube in dnf {
                    self.solver.add_clause(&!cube.clone());
                }
            }
        }
    }

    pub fn blocked<'a>(&'a mut self, cube: &Cube) -> BlockResult<'a> {
        let start = Instant::now();
        let mut assumption = self.share.state_transform.cube_next(cube);
        let act = self.solver.new_var();
        assumption.push(act);
        let mut tmp_cls = !cube.clone();
        tmp_cls.push(!act);
        self.solver.add_clause(&tmp_cls);
        let res = match self.solver.solve(&assumption) {
            SatResult::Sat(_) => {
                let last = assumption.len() - 1;
                let act = !assumption.remove(last);
                self.solver.release_var(act);
                BlockResult::No(BlockResultNo {
                    solver: &mut self.solver,
                    share: self.share.clone(),
                    assumption,
                })
            }
            SatResult::Unsat(_) => {
                let last = assumption.len() - 1;
                let act = !assumption.remove(last);
                self.solver.release_var(act);
                BlockResult::Yes(BlockResultYes {
                    solver: &mut self.solver,
                    share: self.share.clone(),
                    assumption,
                })
            }
        };
        self.share.statistic.lock().unwrap().blocked_check_time += start.elapsed();
        res
    }

    pub fn add_clause(&mut self, clause: &Clause) {
        self.solver.add_clause(clause);
        self.solver.simplify();
    }

    pub fn solve<'a>(&'a mut self, assumptions: &[Lit]) -> SatResult<Model<'a>, Conflict<'a>> {
        self.solver.solve(assumptions)
    }
}

unsafe impl Sync for PdrSolver {}

unsafe impl Send for PdrSolver {}

pub enum BlockResult<'a> {
    Yes(BlockResultYes<'a>),
    No(BlockResultNo<'a>),
}

pub struct BlockResultYes<'a> {
    solver: &'a mut Solver,
    share: Arc<BasicShare>,
    assumption: Cube,
}

impl BlockResultYes<'_> {
    pub fn get_conflict(self) -> Cube {
        assert!(!cube_subsume_init(&self.assumption));
        let conflict = unsafe { self.solver.get_conflict() };
        let conflict: Cube = self
            .assumption
            .iter()
            .filter_map(|l| conflict.has_lit(!*l).then_some(*l))
            .collect();
        let mut ans = self
            .share
            .as_ref()
            .state_transform
            .previous(conflict.into_iter())
            .collect();
        if cube_subsume_init(&ans) {
            let pos_lit = self.assumption.iter().find(|l| !l.compl()).unwrap();
            ans.push(*pos_lit);
        }
        ans
    }
}

pub struct BlockResultNo<'a> {
    solver: &'a mut Solver,
    share: Arc<BasicShare>,
    assumption: Cube,
}

impl BlockResultNo<'_> {
    pub fn get_model(mut self) -> Cube {
        let model = unsafe { self.solver.get_model() };
        let res = generalize_by_ternary_simulation(
            &self.share.as_ref().aig,
            model,
            &AigCube::from_cube(take(&mut self.assumption)),
        )
        .to_cube();
        res
    }

    fn lit_value(&mut self, lit: Lit) -> bool {
        let model = unsafe { self.solver.get_model() };
        model.lit_value(lit)
    }
}
