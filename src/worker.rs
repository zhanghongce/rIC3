use super::{
    activity::Activity,
    basic::{BasicShare, HeapFrameCube},
    broadcast::PdrSolverBroadcastReceiver,
    frames::Frames,
    solver::{BlockResult, PdrSolver},
};
use crate::{cex::Cex, utils::relation::cube_subsume_init};
use logic_form::Cube;
use std::{
    collections::{BinaryHeap, VecDeque},
    sync::{Arc, Mutex},
};

pub struct PdrWorker {
    solvers: Vec<PdrSolver>,
    pub frames: Arc<Frames>,
    pub share: Arc<BasicShare>,
    pub activity: Activity,
    pub cex: Arc<Mutex<Cex>>,
}

impl PdrWorker {
    pub fn new(share: Arc<BasicShare>, frames: Arc<Frames>, cex: Arc<Mutex<Cex>>) -> Self {
        Self {
            solvers: Vec::new(),
            frames,
            activity: Activity::new(&share.aig),
            share,
            cex,
        }
    }

    pub fn depth(&self) -> usize {
        self.solvers.len() - 1
    }

    pub fn new_frame(&mut self, receiver: PdrSolverBroadcastReceiver) {
        self.solvers.push(PdrSolver::new(
            self.share.clone(),
            self.solvers.len(),
            receiver,
        ));
    }

    pub fn blocked<'a>(&'a mut self, frame: usize, cube: &Cube) -> BlockResult<'a> {
        assert!(!cube_subsume_init(cube));
        assert!(frame > 0);
        self.solvers[frame - 1].block_fetch(&self.frames);
        self.solvers[frame - 1].blocked(cube)
    }

    fn generalize(&mut self, frame: usize, cube: Cube) -> (usize, Cube) {
        let cube = self.mic(frame, cube, false);
        for i in frame + 1..=self.depth() {
            if let BlockResult::No(_) = self.blocked(i, &cube) {
                return (i, cube);
            }
        }
        (self.depth() + 1, cube)
    }

    pub fn block(&mut self, frame: usize, cube: Cube) -> bool {
        let mut heap = BinaryHeap::new();
        let mut heap_num = vec![0; self.depth() + 1];
        heap.push(HeapFrameCube::new(frame, cube));
        heap_num[frame] += 1;
        while let Some(HeapFrameCube { frame, cube }) = heap.pop() {
            assert!(cube.is_sorted_by_key(|x| x.var()));
            if frame == 0 {
                return false;
            }
            assert!(!cube_subsume_init(&cube));
            if self.share.args.verbose {
                println!("{:?}", heap_num);
                self.statistic();
            }
            heap_num[frame] -= 1;
            if self.frames.trivial_contained(frame, &cube) {
                continue;
            }
            match self.blocked(frame, &cube) {
                BlockResult::Yes(conflict) => {
                    let conflict = conflict.get_conflict();
                    if self.share.args.ctp {
                        if let Some(similar_cube) = self.frames.similar(&conflict, frame) {
                            self.share.statistic.lock().unwrap().test_a += 1;
                            if let BlockResult::Yes(similar_conflict) =
                                self.blocked(frame, &similar_cube)
                            {
                                let similar_conflict = similar_conflict.get_conflict();
                                self.share.statistic.lock().unwrap().test_c += 1;
                                self.frames.add_cube(frame, similar_conflict);
                                continue;
                            }
                        } else {
                            self.share.statistic.lock().unwrap().test_b += 1;
                        }
                    }
                    let (frame, core) = self.generalize(frame, conflict);
                    if frame < self.depth() {
                        heap.push(HeapFrameCube::new(frame + 1, cube));
                        heap_num[frame + 1] += 1;
                    }
                    self.frames.add_cube(frame - 1, core);
                }
                BlockResult::No(model) => {
                    heap.push(HeapFrameCube::new(frame - 1, model.get_model()));
                    heap.push(HeapFrameCube::new(frame, cube));
                    heap_num[frame - 1] += 1;
                    heap_num[frame] += 1;
                }
            }
        }
        true
    }

    // pub fn try_block(&mut self, frame: usize, cube: &Cube, mut max_try: usize) -> bool {
    //     loop {
    //         if max_try > 5 {
    //             return false;
    //         }
    //         max_try -= 1;
    //         match self.blocked(frame, &cube) {
    //             BlockResult::Yes(conflict) => {
    //                 let conflict = conflict.get_conflict();
    //                 self.frames.add_cube(frame + 1, conflict);
    //                 return true;
    //             }
    //             BlockResult::No(cex) => {
    //                 let cex = cex.get_model();
    //                 if let BlockResult::Yes(conflict) = self.blocked(frame, &cex) {
    //                     let conflict = conflict.get_conflict();
    //                     let cex = self.mic(frame_idx, conflict, true);
    //                     frame.push_back(cex.clone());
    //                     self.frames.add_cube(frame_idx, cex);
    //                 } else {
    //                     break;
    //                 }
    //             }
    //         }
    //     }
    // }

    pub fn start(&mut self) -> bool {
        loop {
            let cex = self.cex.lock().unwrap().get();
            if let Some(cex) = cex {
                assert!(cex.is_sorted_by_key(|x| x.var()));
                if !self.block(self.depth(), cex) {
                    return false;
                }
            } else {
                return true;
            }
        }
    }

    pub fn propagate(&mut self) -> bool {
        for frame_idx in 1..self.depth() {
            let mut frame = VecDeque::from_iter(
                self.frames.frames.read().unwrap()[frame_idx]
                    .iter()
                    .cloned(),
            );
            while let Some(cube) = frame.pop_front() {
                if self.frames.trivial_contained(frame_idx + 1, &cube) {
                    continue;
                }
                if self.share.args.ctp {
                    let mut ctp = 0;
                    loop {
                        if ctp > 5 {
                            break;
                        }
                        match self.blocked(frame_idx + 1, &cube) {
                            BlockResult::Yes(conflict) => {
                                dbg!(ctp);
                                let conflict = conflict.get_conflict();
                                self.frames.add_cube(frame_idx + 1, conflict);
                                break;
                            }
                            BlockResult::No(cex) => {
                                let cex = cex.get_model();
                                ctp += 1;
                                if let BlockResult::Yes(conflict) = self.blocked(frame_idx, &cex) {
                                    let conflict = conflict.get_conflict();
                                    let cex = self.mic(frame_idx, conflict, true);
                                    frame.push_back(cex.clone());
                                    self.frames.add_cube(frame_idx, cex);
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    if let BlockResult::Yes(conflict) = self.blocked(frame_idx + 1, &cube) {
                        let conflict = conflict.get_conflict();
                        self.frames.add_cube(frame_idx + 1, conflict);
                    }
                }
            }
            if self.frames.frames.read().unwrap()[frame_idx].is_empty() {
                return true;
            }
        }
        self.frames.reset_early_update();
        false
    }
}
