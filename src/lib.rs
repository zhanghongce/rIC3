#![feature(assert_matches, is_sorted, get_mut_unchecked)]

mod activity;
mod basic;
mod command;
mod frames;
mod mic;
mod model;
mod solver;
mod statistic;
mod utils;
mod verify;

use crate::basic::ProofObligation;
use crate::{basic::BasicShare, statistic::Statistic};
use crate::{
    basic::{Ic3Error, ProofObligationQueue},
    solver::Lift,
};
use activity::Activity;
use aig::Aig;
pub use command::Args;
use frames::Frames;
use logic_form::{Cube, Lit};
use model::Model;
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
    pub cav23_activity: Activity,
    pub obligations: ProofObligationQueue,
    pub lift: Lift,
    pub blocked: HashMap<(usize, Cube), Cube>,
}

impl Ic3 {
    pub fn new(args: Args) -> Self {
        let aig = Aig::from_file(args.model.as_ref().unwrap()).unwrap();
        let model = Model::from_aig(&aig);
        let bad = Cube::from([if aig.bads.is_empty() {
            aig.outputs[0]
        } else {
            aig.bads[0]
        }
        .to_lit()]);
        let share = Arc::new(BasicShare {
            aig,
            args,
            model,
            statistic: Mutex::new(Statistic::default()),
            bad,
        });
        let mut res = Self {
            solvers: Vec::new(),
            frames: Frames::new(),
            activity: Activity::new(),
            cav23_activity: Activity::new(),
            lift: Lift::new(share.clone()),
            share,
            obligations: ProofObligationQueue::new(),
            blocked: HashMap::new(),
        };
        res.new_frame();
        for i in 0..res.share.aig.latchs.len() {
            let l = &res.share.aig.latchs[i];
            if let Some(init) = l.init {
                let cube = Cube::from([Lit::new(l.input.into(), !init)]);
                res.add_cube(0, cube)
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
    }

    fn generalize(
        &mut self,
        frame: usize,
        cube: Cube,
        depth: usize,
        successor: Option<&Cube>,
    ) -> Result<(usize, Cube), Ic3Error> {
        // let cube = self.new_mic(frame, cube, !self.share.args.ctg, depth, successor)?;
        let cube = self.mic(frame, cube, !self.share.args.ctg)?;
        for i in frame + 1..=self.depth() {
            if let BlockResult::No(_) = self.blocked(i, &cube) {
                return Ok((i, cube));
            }
        }
        Ok((self.depth() + 1, cube))
    }

    pub fn handle_blocked(&mut self, po: ProofObligation, conflict: Cube) {
        let (frame, core) = self
            .generalize(po.frame, conflict, po.depth, po.successor.as_ref())
            .unwrap();
        if frame <= self.depth() {
            self.obligations
                .add(ProofObligation::new(frame, po.cube, po.depth, po.successor));
        }
        self.add_cube(frame - 1, core);
    }

    pub fn block(&mut self, frame: usize, cube: Cube) -> Result<bool, Ic3Error> {
        assert!(self.obligations.is_empty());
        self.obligations
            .add(ProofObligation::new(frame, cube, 0, None));
        while let Some(po) = self.obligations.pop() {
            if po.frame == 0 {
                return Ok(false);
            }
            assert!(!self.share.model.cube_subsume_init(&po.cube));
            if self.share.args.verbose {
                self.obligations.statistic();
                self.statistic();
            }
            if self.frames.trivial_contained(po.frame, &po.cube) {
                continue;
            }
            if let Some(conflict) = self.blocked.get(&(po.frame, po.cube.clone())) {
                self.handle_blocked(po, conflict.clone());
                self.share.statistic.lock().unwrap().test_d += 1;
                continue;
            }
            // if self.sat_contained(po.frame, &po.cube) {
            //     continue;
            // }
            match self.blocked(po.frame, &po.cube) {
                BlockResult::Yes(blocked) => {
                    let conflict = self.blocked_get_conflict(&blocked);
                    self.handle_blocked(po, conflict);
                }
                BlockResult::No(unblocked) => {
                    let model = self.unblocked_get_model(&unblocked);
                    self.obligations.add(ProofObligation::new(
                        po.frame - 1,
                        model,
                        po.depth + 1,
                        Some(po.cube.clone()),
                    ));
                    self.obligations.add(po);
                }
            }
        }
        Ok(true)
    }

    pub fn propagate(&mut self, trivial: bool) -> bool {
        let start = if trivial { self.depth() - 1 } else { 1 };
        for frame_idx in start..self.depth() {
            let mut frame = self.frames[frame_idx].clone();
            frame.sort_by_key(|x| x.len());
            for cube in frame {
                if !self.frames[frame_idx].contains(&cube) {
                    continue;
                }
                if let BlockResult::Yes(blocked) = self.blocked(frame_idx + 1, &cube) {
                    let conflict = self.blocked_get_conflict(&blocked);
                    self.add_cube(frame_idx + 1, conflict);
                    if self.share.args.cav23 {
                        self.cav23_activity.pump_cube_activity(&cube);
                    }
                }
            }
            self.solvers[frame_idx + 1].simplify();
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
                if let Some(cex) = self.get_bad() {
                    trivial = false;
                    match self.block(self.depth(), cex) {
                        Ok(false) => {
                            self.statistic();
                            return false;
                        }
                        Ok(true) => (),
                        Err(Ic3Error::StopBlock) => {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
            let blocked_time = start.elapsed();
            // println!(
            //     "[{}:{}] frame: {}, time: {:?}",
            //     file!(),
            //     line!(),
            //     self.depth(),
            //     blocked_time,
            // );
            self.share.statistic.lock().unwrap().overall_block_time += blocked_time;
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
