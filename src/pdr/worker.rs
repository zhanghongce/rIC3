use super::{
    activity::Activity,
    basic::{BasicShare, HeapFrameCube},
    broadcast::PdrSolverBroadcastReceiver,
    frames::Frames,
    solver::{BlockResult, PdrSolver},
};
use crate::utils::{generalize::generalize_by_ternary_simulation, relation::cube_subsume_init};
use logic_form::Cube;
use sat_solver::SatResult;
use std::{
    collections::BinaryHeap,
    sync::{Arc, RwLock},
};

pub struct PdrWorker {
    solvers: Vec<PdrSolver>,
    pub frames: Arc<RwLock<Frames>>,
    pub share: Arc<BasicShare>,
    pub activity: Activity,
}

impl PdrWorker {
    pub fn new(share: Arc<BasicShare>, frames: Arc<RwLock<Frames>>) -> Self {
        Self {
            solvers: Vec::new(),
            frames,
            activity: Activity::new(&share.aig),
            share,
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
        self.solvers[frame - 1].fetch(&self.frames);
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

    pub fn block(&mut self, cube: Cube) -> bool {
        let mut heap = BinaryHeap::new();
        let frame = self.depth();
        let mut heap_num = vec![0; frame + 1];
        heap.push(HeapFrameCube::new(frame, cube));
        heap_num[frame] += 1;
        while let Some(HeapFrameCube { frame, cube }) = heap.pop() {
            assert!(cube.is_sorted_by_key(|x| x.var()));
            assert!(!cube_subsume_init(&cube));
            if frame == 0 {
                return false;
            }
            if self.share.args.verbose {
                println!("{:?}", heap_num);
                self.statistic();
            }
            heap_num[frame] -= 1;
            if self.frames.read().unwrap().trivial_contained(frame, &cube) {
                continue;
            }
            match self.blocked(frame, &cube) {
                BlockResult::Yes(conflict) => {
                    let conflict = conflict.get_conflict();
                    let (frame, core) = self.generalize(frame, conflict);
                    if frame < self.depth() {
                        heap.push(HeapFrameCube::new(frame + 1, cube));
                        heap_num[frame + 1] += 1;
                    }
                    if !self
                        .frames
                        .read()
                        .unwrap()
                        .trivial_contained(frame - 1, &core)
                    {
                        self.frames.write().unwrap().add_cube(frame - 1, core);
                    }
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

    pub fn propagate(&mut self) -> bool {
        for frame_idx in 1..self.depth() {
            let frame = self.frames.read().unwrap().frames[frame_idx].clone();
            for cube in frame {
                if self
                    .frames
                    .read()
                    .unwrap()
                    .trivial_contained(frame_idx + 1, &cube)
                {
                    continue;
                }
                match self.blocked(frame_idx + 1, &cube) {
                    BlockResult::Yes(conflict) => {
                        let conflict = conflict.get_conflict();
                        self.frames
                            .write()
                            .unwrap()
                            .add_cube(frame_idx + 1, conflict);
                    }
                    BlockResult::No(_) => {
                        // 利用cex？x
                    }
                };
            }
            if self.frames.read().unwrap().frames[frame_idx].is_empty() {
                return true;
            }
        }
        false
    }

    pub fn get_cex(&mut self) -> Option<Cube> {
        let last_frame_index = self.depth();
        self.solvers[last_frame_index].fetch(&self.frames);
        if let SatResult::Sat(model) =
            self.solvers[last_frame_index].solve(&[self.share.aig.bads[0].to_lit()])
        {
            self.share.statistic.lock().unwrap().num_get_bad_state += 1;
            let cex =
                generalize_by_ternary_simulation(&self.share.aig, model, &[self.share.aig.bads[0]])
                    .to_cube();
            return Some(cex);
        }
        None
    }
}
