use super::share::PdrShare;
use crate::utils::generalize::generalize_by_ternary_simulation;
use aig::AigCube;
use logic_form::{Clause, Cube, Lit};
use sat_solver::{
    minisat::{Conflict, Model, Solver},
    SatModel, SatResult, SatSolver, UnsatConflict,
};
use std::{mem::take, sync::Arc};

pub struct PdrSolver {
    solver: Solver,
    num_act: usize,

    share: Arc<PdrShare>,
}

impl PdrSolver {
    pub fn new(share: Arc<PdrShare>) -> Self {
        let mut solver = Solver::new();
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
            self.num_act = 0;
            self.solver = Solver::new();
            self.solver.add_cnf(&self.share.transition_cnf);
            for dnf in frames {
                for cube in dnf {
                    self.solver.add_clause(&!cube.clone());
                }
            }
        }
    }

    pub fn blocked<'a>(&'a mut self, cube: &Cube) -> BlockResult<'a> {
        // self.statistic.num_blocked += 1;
        let mut assumption = self.share.state_transform.cube_next(cube);
        let act = self.solver.new_var();
        assumption.push(act);
        let mut tmp_cls = !cube.clone();
        tmp_cls.push(!act);
        self.solver.add_clause(&tmp_cls);
        match self.solver.solve(&assumption) {
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
        }
    }

    pub fn add_clause(&mut self, clause: &Clause) {
        self.solver.add_clause(clause);
        self.solver.simplify();
    }

    pub fn solve<'a>(&'a mut self, assumptions: &[Lit]) -> SatResult<Model<'a>, Conflict<'a>> {
        self.solver.solve(assumptions)
    }
}

pub enum BlockResult<'a> {
    Yes(BlockResultYes<'a>),
    No(BlockResultNo<'a>),
}

pub struct BlockResultYes<'a> {
    solver: &'a mut Solver,
    share: Arc<PdrShare>,
    assumption: Cube,
}

impl BlockResultYes<'_> {
    pub fn get_conflict(mut self) -> Cube {
        let conflict = unsafe { self.solver.get_conflict() };
        let ans = self
            .share
            .as_ref()
            .state_transform
            .previous(
                take(&mut self.assumption)
                    .into_iter()
                    .filter(|l| conflict.has_lit(!*l)),
            )
            .collect();
        ans
    }
}

pub struct BlockResultNo<'a> {
    solver: &'a mut Solver,
    share: Arc<PdrShare>,
    assumption: Cube,
}

impl BlockResultNo<'_> {
    pub fn get_model(mut self) -> Cube {
        let model = unsafe { self.solver.get_model() };
        generalize_by_ternary_simulation(
            &self.share.as_ref().aig,
            model,
            &AigCube::from_cube(take(&mut self.assumption)),
        )
        .to_cube()
    }

    fn lit_value(&mut self, lit: Lit) -> bool {
        let model = unsafe { self.solver.get_model() };
        model.lit_value(lit)
    }
}
