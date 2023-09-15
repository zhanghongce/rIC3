use super::{basic::BasicShare, frames::Frames};
use crate::utils::{generalize::generalize_by_ternary_simulation, relation::cube_subsume_init};
use aig::AigCube;
use logic_form::{Clause, Cube, Lit};
use sat_solver::{
    minisat::{Conflict, Model, Solver},
    SatResult, SatSolver, UnsatConflict,
};
use std::{mem::take, sync::Arc, time::Instant};

pub struct Ic3Solver {
    solver: Solver,
    num_act: usize,
    share: Arc<BasicShare>,
    frame: usize,
}

impl Ic3Solver {
    pub fn new(share: Arc<BasicShare>, frame: usize) -> Self {
        let mut solver = Solver::new();
        solver.set_random_seed(share.args.random as f64);
        solver.add_cnf(&share.as_ref().transition_cnf);
        Self {
            solver,
            frame,
            num_act: 0,
            share,
        }
    }

    pub fn reset(&mut self, frames: &Frames) {
        self.num_act = 0;
        self.solver = Solver::new();
        self.solver.add_cnf(&self.share.transition_cnf);
        let frames_slice = if self.frame == 0 {
            &frames[0..1]
        } else {
            &frames[self.frame..]
        };
        for dnf in frames_slice.iter() {
            for cube in dnf {
                self.add_clause(&!cube);
            }
        }
    }

    pub fn block_fetch(&mut self, frames: &Frames) {
        self.num_act += 1;
        if self.num_act > 300 {
            self.reset(frames)
        }
    }

    pub fn blocked<'a>(&'a mut self, cube: &Cube) -> BlockResult<'a> {
        let start = Instant::now();
        assert!(!cube_subsume_init(&self.share.init, cube));
        let mut assumption = self.share.state_transform.cube_next(cube);
        let act = self.solver.new_var().into();
        assumption.push(act);
        let mut tmp_cls = !cube;
        tmp_cls.push(!act);
        self.add_clause(&tmp_cls);
        let res = match self.solver.solve(&assumption) {
            SatResult::Sat(_) => {
                let act = !assumption.pop().unwrap();
                self.solver.release_var(act);
                BlockResult::No(BlockResultNo {
                    solver: &mut self.solver,
                    share: self.share.clone(),
                    assumption,
                })
            }
            SatResult::Unsat(_) => {
                let act = !assumption.pop().unwrap();
                self.solver.release_var(act);
                BlockResult::Yes(BlockResultYes {
                    solver: &mut self.solver,
                    cube: cube.clone(),
                    assumption,
                    share: self.share.clone(),
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

    pub fn get_bad(&mut self) -> Option<Cube> {
        let bad = if self.share.aig.bads.is_empty() {
            self.share.aig.outputs[0]
        } else {
            self.share.aig.bads[0]
        };
        if let SatResult::Sat(model) = self.solver.solve(&[bad.to_lit()]) {
            self.share.statistic.lock().unwrap().num_get_bad_state += 1;
            let cex = generalize_by_ternary_simulation(&self.share.aig, model, &[bad]).to_cube();
            return Some(cex);
        }
        None
    }

    #[allow(unused)]
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
    cube: Cube,
    assumption: Cube,
    share: Arc<BasicShare>,
}

impl BlockResultYes<'_> {
    pub fn get_conflict(self) -> Cube {
        let conflict = unsafe { self.solver.get_conflict() };
        assert!(self.cube.len() == self.assumption.len());
        let mut ans = Cube::new();
        for i in 0..self.cube.len() {
            if conflict.has(!self.assumption[i]) {
                ans.push(self.cube[i]);
            }
        }
        if cube_subsume_init(&self.share.init, &ans) {
            ans.push(
                *self
                    .cube
                    .iter()
                    .find(|l| {
                        self.share
                            .init
                            .get(&l.var())
                            .is_some_and(|i| *i != l.polarity())
                    })
                    .unwrap(),
            );
        }
        assert!(!cube_subsume_init(&self.share.init, &ans));
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
}
