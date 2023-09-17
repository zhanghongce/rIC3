use super::{
    activity::Activity,
    basic::BasicShare,
    frames::Frames,
    solver::{BlockResult, Ic3Solver},
};
use crate::{basic::ProofObligationQueue, utils::relation::cube_subsume_init};
use logic_form::Cube;
use pic3::Synchronizer;
use rand::{seq::SliceRandom, thread_rng};
use std::{sync::Arc, time::Instant};

pub struct Ic3Worker {
    pub solvers: Vec<Ic3Solver>,
    pub frames: Frames,
    pub share: Arc<BasicShare>,
    pub activity: Activity,
    pub pic3_synchronizer: Option<Synchronizer>,
    pub cav23_activity: Activity,
}

impl Ic3Worker {
    pub fn new(share: Arc<BasicShare>, pic3_synchronizer: Option<Synchronizer>) -> Self {
        Self {
            solvers: Vec::new(),
            frames: Frames::new(),
            activity: Activity::new(&share.aig),
            cav23_activity: Activity::new(&share.aig),
            pic3_synchronizer,
            share,
        }
    }

    pub fn depth(&self) -> usize {
        self.solvers.len() - 1
    }

    pub fn new_frame(&mut self) {
        self.frames.new_frame();
        self.solvers
            .push(Ic3Solver::new(self.share.clone(), self.solvers.len()));
    }

    pub fn blocked<'a>(&'a mut self, frame: usize, cube: &Cube) -> BlockResult<'a> {
        self.pic3_sync();
        assert!(!cube_subsume_init(&self.share.init, cube));
        assert!(frame > 0);
        self.solvers[frame - 1].block_fetch(&self.frames);
        self.solvers[frame - 1].blocked(cube)
    }

    fn generalize(&mut self, frame: usize, cube: Cube, simple: bool) -> (usize, Cube) {
        let cube = self.mic(frame, cube, simple);
        for i in frame + 1..=self.depth() {
            if let BlockResult::No(_) = self.blocked(i, &cube) {
                return (i, cube);
            }
        }
        (self.depth() + 1, cube)
    }

    pub fn block(&mut self, frame: usize, cube: Cube) -> bool {
        let mut obligations = ProofObligationQueue::new();
        let mut heap_num = vec![0; self.depth() + 1];
        obligations.add(frame, cube);
        heap_num[frame] += 1;
        while let Some((frame, cube)) = obligations.get() {
            if frame == 0 {
                return false;
            }
            assert!(!cube_subsume_init(&self.share.init, &cube));
            if self.share.args.verbose {
                println!("{:?}", heap_num);
                self.statistic();
            }
            heap_num[frame] -= 1;
            if self.frames.trivial_contained(frame, &cube) {
                continue;
            }
            let start = Instant::now();
            if self.sat_contained(frame, &cube) {
                continue;
            }
            self.share.statistic.lock().unwrap().test_duration += start.elapsed();
            match self.blocked(frame, &cube) {
                BlockResult::Yes(conflict) => {
                    let conflict = conflict.get_conflict();
                    let (frame, core) = self.generalize(frame, conflict, !self.share.args.ctg);
                    if frame <= self.depth() {
                        obligations.add(frame, cube);
                        heap_num[frame] += 1;
                    }
                    self.add_cube(frame - 1, core);
                }
                BlockResult::No(model) => {
                    obligations.add(frame - 1, model.get_model());
                    obligations.add(frame, cube);
                    heap_num[frame - 1] += 1;
                    heap_num[frame] += 1;
                }
            }
        }
        true
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
                    let (frame, core) = self.generalize(frame, conflict, simple);
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

    pub fn start(&mut self) -> bool {
        loop {
            self.pic3_sync();
            if let Some(cex) = self.solvers.last_mut().unwrap().get_bad() {
                if !self.block(self.depth(), cex) {
                    return false;
                }
            } else {
                return true;
            }
        }
    }

    // pub fn eliminate_check(&mut self) {
    //     let mut remove = HashSet::new();
    //     for frame_idx in 1..self.depth() - 1 {
    //         for c in self.frames[frame_idx].iter() {
    //             let mut solver = Ic3Solver::new(self.share.clone(), frame_idx);
    //             for frame in &self.frames[frame_idx..] {
    //                 for cc in frame {
    //                     if c != cc {
    //                         solver.add_clause(&!cc);
    //                     }
    //                 }
    //             }
    //             let mut ans = true;
    //             for cc in self.frames[frame_idx + 1].iter() {
    //                 if let SatResult::Sat(_) =
    //                     solver.solve(&self.share.state_transform.cube_next(cc))
    //                 {
    //                     self.share.statistic.lock().unwrap().test_a += 1;
    //                     ans = false;
    //                     break;
    //                 }
    //             }
    //             if ans {
    //                 if let SatResult::Unsat(_) = self.solvers[frame_idx - 1].solve(&c) {
    //                     remove.insert(c.clone());
    //                     self.share.statistic.lock().unwrap().test_b += 1;
    //                 }
    //             }
    //         }
    //     }
    //     for i in 1..self.depth() - 1 {
    //         self.frames[i] = self.frames[i]
    //             .iter()
    //             .filter(|c| !remove.contains(c))
    //             .cloned()
    //             .collect();
    //     }
    //     for i in 1..self.depth() - 1 {
    //         self.solvers[i].reset(&self.frames);
    //         while let Some(bad) = self.solvers[i].get_bad() {
    //             dbg!(i);
    //             self.block(i, bad);
    //         }
    //     }
    // }

    pub fn propagate(&mut self) -> bool {
        for frame_idx in 1..self.depth() {
            let mut frame = self.frames[frame_idx].clone();
            frame.shuffle(&mut thread_rng());
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
