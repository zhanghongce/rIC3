#![feature(assert_matches, is_sorted, get_mut_unchecked)]

mod activity;
mod basic;
mod command;
mod frames;
mod mic;
mod solver;
mod statistic;
mod utils;
mod verify;

use activity::Activity;
pub use command::Args;

use crate::basic::ProofObligation;
use crate::utils::state_transform::StateTransform;
use crate::{basic::BasicShare, statistic::Statistic};
use crate::{
    basic::{Ic3Error, ProofObligationQueue},
    solver::Lift,
    utils::relation::cube_subsume_init,
};
use aig::Aig;
use frames::Frames;
use logic_form::{Cube, Lit};
use pic3::Synchronizer;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use solver::{BlockResult, Ic3Solver};
use std::collections::HashMap;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

pub struct Ic3 {
    pub solvers: Vec<Ic3Solver>,
    pub frames: Frames,
    pub share: Arc<BasicShare>,
    pub activity: Activity,
    pub pic3_synchronizer: Option<Synchronizer>,
    pub cav23_activity: Activity,
    pub stop_block: bool,
    pub lift: Lift,
    rng: StdRng,
}

impl Ic3 {
    pub fn new(args: Args, pic3_synchronizer: Option<Synchronizer>) -> Self {
        let aig = Aig::from_file(args.model.as_ref().unwrap()).unwrap();
        let transition_cnf = aig.get_cnf();
        let mut init = HashMap::new();
        for l in aig.latch_init_cube().to_cube() {
            init.insert(l.var(), l.polarity());
        }
        let state_transform = StateTransform::new(&aig);
        let share = Arc::new(BasicShare {
            aig,
            transition_cnf,
            state_transform,
            args,
            init,
            statistic: Mutex::new(Statistic::default()),
        });
        let mut res = Self {
            solvers: Vec::new(),
            frames: Frames::new(),
            activity: Activity::new(),
            cav23_activity: Activity::new(),
            pic3_synchronizer,
            rng: SeedableRng::seed_from_u64(share.args.random as _),
            lift: Lift::new(share.clone()),
            share,
            stop_block: false,
        };
        res.new_frame();
        for i in 0..res.share.aig.latchs.len() {
            let l = &res.share.aig.latchs[i];
            if let Some(init) = l.init {
                let cube = Cube::from([Lit::new(l.input.into(), !init)]);
                res.add_cube(0, cube.clone())
            }
        }
        res
    }

    pub fn depth(&self) -> usize {
        self.solvers.len() - 1
    }

    pub fn new_frame(&mut self) {
        self.frames.new_frame();
        self.solvers
            .push(Ic3Solver::new(self.share.clone(), self.solvers.len()));
        self.stop_block = false;
    }

    pub fn blocked<'a>(&'a mut self, frame: usize, cube: &Cube) -> BlockResult<'a> {
        self.pic3_sync();
        assert!(!cube_subsume_init(&self.share.init, cube));
        assert!(frame > 0);
        self.solvers[frame - 1].block_fetch(&self.frames);
        self.solvers[frame - 1].blocked(cube, &mut self.lift, &self.activity)
    }

    pub fn blocked_with_polarity<'a>(
        &'a mut self,
        frame: usize,
        cube: &Cube,
        polarity: &[Lit],
    ) -> BlockResult<'a> {
        self.pic3_sync();
        assert!(!cube_subsume_init(&self.share.init, cube));
        assert!(frame > 0);
        self.solvers[frame - 1].block_fetch(&self.frames);
        for l in polarity {
            self.solvers[frame - 1].set_polarity(*l)
        }
        self.solvers[frame - 1].blocked(cube, &mut self.lift, &self.activity)
    }

    fn generalize(
        &mut self,
        frame: usize,
        cube: Cube,
        simple: bool,
    ) -> Result<(usize, Cube), Ic3Error> {
        let cube = self.mic(frame, cube, simple)?;
        for i in frame + 1..=self.depth() {
            if let BlockResult::No(_) = self.blocked(i, &cube) {
                return Ok((i, cube));
            }
        }
        Ok((self.depth() + 1, cube))
    }

    pub fn block(&mut self, frame: usize, cube: Cube) -> Result<bool, Ic3Error> {
        let mut obligations = ProofObligationQueue::new();
        obligations.add(ProofObligation {
            frame,
            cube,
            depth: 0,
        });
        while let Some(po) = obligations.get() {
            if po.frame == 0 {
                return Ok(false);
            }
            self.check_stop_block()?;
            assert!(!cube_subsume_init(&self.share.init, &po.cube));
            if self.share.args.verbose {
                obligations.statistic();
                self.statistic();
            }
            if self.frames.trivial_contained(po.frame, &po.cube) {
                continue;
            }
            // if self.sat_contained(po.frame, &po.cube) {
            //     continue;
            // }
            match self.blocked(po.frame, &po.cube) {
                BlockResult::Yes(conflict) => {
                    let conflict = conflict.get_conflict();
                    let (frame, core) =
                        self.generalize(po.frame, conflict, !self.share.args.ctg)?;
                    if frame <= self.depth() {
                        obligations.add(ProofObligation {
                            frame,
                            cube: po.cube,
                            depth: po.depth,
                        });
                    }
                    self.add_cube(frame - 1, core);
                }
                BlockResult::No(model) => {
                    obligations.add(ProofObligation {
                        frame: po.frame - 1,
                        cube: model.get_model(),
                        depth: po.depth + 1,
                    });
                    obligations.add(po);
                }
            }
        }
        Ok(true)
    }

    #[allow(dead_code)]
    pub fn try_block(
        &mut self,
        frame: usize,
        cube: &Cube,
        mut max_try: usize,
        simple: bool,
    ) -> bool {
        loop {
            if max_try == 0 || frame == 0 || cube_subsume_init(&self.share.init, cube) {
                return false;
            }
            assert!(!cube_subsume_init(&self.share.init, cube));
            max_try -= 1;
            match self.blocked(frame, cube) {
                BlockResult::Yes(conflict) => {
                    let conflict = conflict.get_conflict();
                    let (frame, core) = self.generalize(frame, conflict, simple).unwrap();
                    self.add_cube(frame - 1, core);
                    return true;
                }
                BlockResult::No(cex) => {
                    let cex = cex.get_model();
                    if !self.try_block(frame - 1, &cex, max_try, true) {
                        return false;
                    }
                }
            }
        }
    }

    pub fn propagate(&mut self, trivial: bool) -> bool {
        let start = if trivial { self.depth() - 1 } else { 1 };
        for frame_idx in start..self.depth() {
            let mut frame = self.frames[frame_idx].clone();
            frame.shuffle(&mut self.rng);
            for cube in frame {
                if self.frames.trivial_contained(frame_idx + 1, &cube) {
                    continue;
                }
                if let BlockResult::Yes(conflict) = self.blocked(frame_idx + 1, &cube) {
                    let conflict = conflict.get_conflict();
                    self.add_cube(frame_idx + 1, conflict);
                    if self.share.args.cav23 {
                        self.activity.pump_cube_activity(&cube);
                    }
                }
            }
            if self.frames[frame_idx].is_empty() {
                return true;
            }
        }
        false
    }
}

impl Ic3 {
    pub fn check(&mut self) -> bool {
        loop {
            let start = Instant::now();
            let mut trivial = true;
            loop {
                self.pic3_sync();
                if let Some(cex) = self.solvers.last_mut().unwrap().get_bad() {
                    trivial = false;
                    match self.block(self.depth(), cex) {
                        Ok(false) => {
                            self.statistic();
                            return false;
                        }
                        Ok(true) => (),
                        Err(Ic3Error::StopBlock) => {
                            self.stop_block = false;
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
            let blocked_time = start.elapsed();
            let depth = self.depth();
            if let Some(pic3_synchronizer) = self.pic3_synchronizer.as_mut() {
                pic3_synchronizer.frame_blocked(depth);
            }
            println!(
                "[{}:{}] frame: {}, time: {:?}",
                file!(),
                line!(),
                self.depth(),
                blocked_time,
            );
            if let Some(pic3_synchronizer) = self.pic3_synchronizer.as_mut() {
                pic3_synchronizer.sync();
            }
            self.share.statistic.lock().unwrap().overall_block_time += blocked_time;
            // self.statistic();
            self.new_frame();
            let start = Instant::now();
            let propagate = self.propagate(trivial);
            self.share.statistic.lock().unwrap().overall_propagate_time += start.elapsed();
            if propagate {
                self.statistic();
                if self.share.args.verify {
                    assert!(self.verify());
                }
                return true;
            }
        }
    }
}
