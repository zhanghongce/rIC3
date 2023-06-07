mod activity;
mod basic;
mod mic;
mod solver;
mod statistic;

use self::{
    activity::Activity,
    basic::BasicShare,
    solver::{BlockResult, PdrSolver},
    statistic::Statistic,
};
use crate::{
    command::Args,
    pdr::basic::HeapFrameCube,
    utils::{
        generalize::generalize_by_ternary_simulation,
        relation::{cube_subsume, cube_subsume_init},
        state_transform::StateTransform,
    },
};
use aig::Aig;
use logic_form::{Clause, Cube, Lit};
use sat_solver::SatResult;
use std::{
    collections::BinaryHeap,
    mem::take,
    sync::{Arc, Mutex},
};

pub struct Pdr {
    pub frames: Vec<Vec<Cube>>,
    solvers: Vec<PdrSolver>,
    share: Arc<BasicShare>,
    activity: Activity,
    min_frame_update: usize,
}

impl Pdr {
    pub fn depth(&self) -> usize {
        self.frames.len() - 1
    }

    pub fn new_frame(&mut self) {
        self.solvers.push(PdrSolver::new(self.share.clone()));
        self.frames.push(Vec::new());
    }

    pub fn frame_add_cube(&mut self, frame: usize, cube: Cube) {
        assert!(frame > 0);
        assert!(cube.is_sorted_by_key(|x| x.var()));
        assert!(!self.trivial_contained(frame, &cube));
        let mut begin = 1;
        for i in 1..=frame {
            let cubes = take(&mut self.frames[i]);
            for c in cubes {
                if cube_subsume(&c, &cube) {
                    begin = i + 1;
                }
                if !cube_subsume(&cube, &c) {
                    self.frames[i].push(c);
                }
            }
        }
        self.frames[frame].push(cube.clone());
        let clause = !cube;
        self.min_frame_update = self.min_frame_update.min(begin);
        for i in begin..=frame {
            self.solvers[i].add_clause(&clause);
        }
    }

    fn trivial_contained(&mut self, frame: usize, cube: &Cube) -> bool {
        for i in frame..=self.depth() {
            for c in self.frames[i].iter() {
                if cube_subsume(c, cube) {
                    return true;
                }
            }
        }
        false
    }

    pub fn blocked<'a>(&'a mut self, frame: usize, cube: &Cube) -> BlockResult<'a> {
        assert!(!cube_subsume_init(cube));
        assert!(frame > 0);
        self.share.statistic.lock().unwrap().num_blocked += 1;
        if frame == 1 {
            self.solvers[frame - 1].pump_act_and_check_restart(&self.frames[0..1]);
        } else {
            self.solvers[frame - 1].pump_act_and_check_restart(&self.frames[frame - 1..]);
        }
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
            if self.trivial_contained(frame, &cube) {
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
                    if !self.trivial_contained(frame - 1, &core) {
                        self.frame_add_cube(frame - 1, core);
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

    fn propagate(&mut self) -> bool {
        for frame_idx in self.min_frame_update..self.depth() {
            let frame = self.frames[frame_idx].clone();
            for cube in frame {
                if self.trivial_contained(frame_idx + 1, &cube) {
                    continue;
                }
                match self.blocked(frame_idx + 1, &cube) {
                    BlockResult::Yes(conflict) => {
                        let conflict = conflict.get_conflict();
                        self.frame_add_cube(frame_idx + 1, conflict);
                    }
                    BlockResult::No(_) => {
                        // 利用cex？x
                    }
                };
            }
            if self.frames[frame_idx].is_empty() {
                return true;
            }
        }
        self.min_frame_update = self.depth();
        false
    }
}

impl Pdr {
    pub fn new(share: Arc<BasicShare>) -> Self {
        let mut solvers = vec![PdrSolver::new(share.clone())];
        let mut init_frame = Vec::new();
        for l in share.aig.latchs.iter() {
            let clause = Clause::from([Lit::new(l.input.into(), !l.init)]);
            init_frame.push(!clause.clone());
            solvers[0].add_clause(&clause);
        }
        let activity = Activity::new(&share.aig);
        Self {
            frames: vec![init_frame],
            solvers,
            activity,
            share,
            min_frame_update: 1,
        }
    }

    pub fn check(&mut self) -> bool {
        self.new_frame();
        loop {
            let last_frame_index = self.depth();
            while let SatResult::Sat(model) =
                self.solvers[last_frame_index].solve(&[self.share.aig.bads[0].to_lit()])
            {
                self.share.statistic.lock().unwrap().num_get_bad_state += 1;
                let cex = generalize_by_ternary_simulation(
                    &self.share.aig,
                    model,
                    &[self.share.aig.bads[0]],
                )
                .to_cube();
                // self.statistic();
                if !self.block(cex) {
                    self.statistic();
                    return false;
                }
            }
            self.statistic();
            self.new_frame();
            if self.propagate() {
                self.statistic();
                return true;
            }
        }
    }
}

pub fn solve(aig: Aig, args: Args) -> bool {
    let transition_cnf = aig.get_cnf();
    assert!(aig.latch_init_cube().to_cube().iter().all(|l| l.compl()));
    let state_transform = StateTransform::new(&aig);
    let share = Arc::new(BasicShare {
        aig,
        transition_cnf,
        state_transform,
        args,
        statistic: Mutex::new(Statistic::default()),
    });
    let mut pdr = Pdr::new(share);
    pdr.check()
}
