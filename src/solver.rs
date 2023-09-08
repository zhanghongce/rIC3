use super::{basic::BasicShare, broadcast::PdrSolverBroadcastReceiver, frames::Frames};
use crate::utils::{generalize::generalize_by_ternary_simulation, relation::cube_subsume_init};
use aig::AigCube;
use logic_form::{Clause, Cube};
use sat_solver::{minisat::Solver, SatResult, SatSolver, UnsatConflict};
use std::{mem::take, sync::Arc, time::Instant};

pub struct PdrSolver {
    solver: Solver,
    receiver: PdrSolverBroadcastReceiver,
    num_act: usize,
    share: Arc<BasicShare>,
    frame: usize,
}

impl PdrSolver {
    pub fn new(share: Arc<BasicShare>, frame: usize, receiver: PdrSolverBroadcastReceiver) -> Self {
        let mut solver = Solver::new();
        solver.set_random_seed(91648253_f64);
        solver.add_cnf(&share.as_ref().transition_cnf);
        Self {
            solver,
            receiver,
            frame,
            num_act: 0,
            share,
        }
    }

    pub fn block_fetch(&mut self, frames: &Frames) {
        self.num_act += 1;
        if self.num_act > 300 {
            self.num_act = 0;
            self.solver = Solver::new();
            self.solver.add_cnf(&self.share.transition_cnf);
            let frames = frames.frames.read().unwrap();
            let frames_slice = if self.frame == 0 {
                &frames[0..1]
            } else {
                &frames[self.frame..]
            };
            for dnf in frames_slice.iter() {
                for cube in dnf {
                    self.add_clause(&!cube.clone());
                }
            }
            while self.receiver.receive_clause().is_some() {}
            drop(frames);
        } else {
            while let Some(clause) = self.receiver.receive_clause() {
                self.add_clause(&clause);
            }
        }
    }

    pub fn blocked<'a>(&'a mut self, cube: &Cube) -> BlockResult<'a> {
        let start = Instant::now();
        assert!(!cube_subsume_init(&cube));
        let mut assumption = self.share.state_transform.cube_next(cube);
        let act = self.solver.new_var().into();
        assumption.push(act);
        let mut tmp_cls = !cube.clone();
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
}

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
        let conflict = unsafe { self.solver.get_conflict() };
        let conflict: Cube = self
            .assumption
            .iter()
            .filter_map(|l| conflict.has(!*l).then_some(*l))
            .collect();
        let mut ans = self
            .share
            .as_ref()
            .state_transform
            .previous(conflict.into_iter())
            .collect();
        if cube_subsume_init(&ans) {
            let pos_lit = self.assumption.iter().find(|l| l.polarity()).unwrap();
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
}
