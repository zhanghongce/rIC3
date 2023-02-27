use super::{
    activity::Activity,
    basic::BasicShare,
    solver::{BlockResult, PdrSolver},
    statistic::Statistic,
};
use crate::{
    pdr::heap_frame_cube::HeapFrameCube,
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
    collections::{BinaryHeap, HashSet},
    mem::take,
    sync::Arc,
};

pub struct Pdr {
    pub frames: Vec<Vec<Cube>>,
    solvers: Vec<PdrSolver>,
    share: Arc<BasicShare>,
    activity: Activity,

    pub statistic: Statistic,
}

impl Pdr {
    fn depth(&self) -> usize {
        self.frames.len() - 1
    }

    pub fn new_frame(&mut self) {
        self.solvers.push(PdrSolver::new(self.share.clone()));
        self.frames.push(Vec::new());
        self.statistic.num_frames = self.depth();
    }

    fn frame_add_cube(&mut self, frame: usize, cube: Cube) {
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
        for i in begin..=frame {
            self.solvers[i].add_clause(&clause);
        }
    }

    fn trivial_contained(&mut self, frame: usize, cube: &Cube) -> bool {
        self.statistic.num_trivial_contained += 1;
        for i in frame..=self.depth() {
            for c in self.frames[i].iter() {
                if cube_subsume(c, cube) {
                    self.statistic.num_trivial_contained_success += 1;
                    return true;
                }
            }
        }
        false
    }

    fn blocked<'a>(&'a mut self, frame: usize, cube: &Cube) -> BlockResult<'a> {
        assert!(frame > 0);
        self.statistic.num_blocked += 1;
        if frame == 1 {
            self.solvers[frame - 1].pump_act_and_check_restart(&self.frames[0..1]);
        } else {
            self.solvers[frame - 1].pump_act_and_check_restart(&self.frames[frame - 1..]);
        }
        self.solvers[frame - 1].blocked(cube)
    }

    fn down(&mut self, frame: usize, cube: Cube) -> Option<Cube> {
        if cube_subsume_init(&cube) {
            return None;
        }
        self.statistic.num_down_blocked += 1;
        match self.blocked(frame, &cube) {
            BlockResult::Yes(conflict) => Some(conflict.get_conflict()),
            BlockResult::No(_) => None,
        }
    }

    fn ctg_down(&mut self, frame: usize, mut cube: Cube, keep: &HashSet<Lit>) -> Option<Cube> {
        self.statistic.num_ctg_down += 1;
        let mut ctgs = 0;
        loop {
            if cube_subsume_init(&cube) {
                return None;
            }
            self.statistic.num_ctg_down_blocked += 1;
            match self.blocked(frame, &cube) {
                BlockResult::Yes(conflict) => return Some(conflict.get_conflict()),
                BlockResult::No(model) => {
                    let model = model.get_model();
                    if ctgs < 3 && frame > 1 && !cube_subsume_init(&model) {
                        if let BlockResult::Yes(conflict) = self.blocked(frame - 1, &model) {
                            ctgs += 1;
                            let conflict = conflict.get_conflict();
                            let mut i = frame;
                            while i <= self.depth() {
                                if let BlockResult::No(_) = self.blocked(i, &conflict) {
                                    break;
                                }
                                i += 1;
                            }
                            let conflict = self.mic(i - 1, conflict, true);
                            self.frame_add_cube(i - 1, conflict);
                            continue;
                        }
                    }
                    ctgs = 0;
                    let cex_set: HashSet<Lit> = HashSet::from_iter(model.into_iter());
                    let mut cube_new = Cube::new();
                    for lit in cube {
                        if cex_set.contains(&lit) {
                            cube_new.push(lit);
                        } else if keep.contains(&lit) {
                            return None;
                        }
                    }
                    cube = cube_new;
                }
            }
        }
    }

    fn mic(&mut self, frame: usize, mut cube: Cube, simple: bool) -> Cube {
        if simple {
            self.statistic.num_simple_mic += 1;
        } else {
            self.statistic.num_normal_mic += 1;
        }
        self.statistic.average_mic_cube_len += cube.len();
        let origin_len = cube.len();
        let mut i = 0;
        assert!(cube.is_sorted_by_key(|x| *x.var()));
        cube = self.activity.sort_by_activity_ascending(cube);
        let mut keep = HashSet::new();
        while i < cube.len() {
            let mut removed_cube = cube.clone();
            removed_cube.remove(i);
            let res = if simple {
                self.down(frame, removed_cube)
            } else {
                self.ctg_down(frame, removed_cube, &keep)
            };
            match res {
                Some(new_cube) => {
                    cube = new_cube;
                    self.statistic.num_mic_drop_success += 1;
                }
                None => {
                    self.statistic.num_mic_drop_fail += 1;
                    keep.insert(cube[i]);
                    i += 1;
                }
            }
        }
        cube.sort_by_key(|x| *x.var());
        for l in cube.iter() {
            self.activity.pump_activity(l);
        }
        self.statistic.average_mic_droped_var += origin_len - cube.len();
        self.statistic.average_mic_droped_var_percent +=
            (origin_len - cube.len()) as f64 / origin_len as f64;
        cube
    }

    fn generalize(&mut self, frame: usize, cube: Cube) -> (usize, Cube) {
        let cube = self.mic(frame, cube, false);
        for i in frame + 1..=self.depth() {
            self.statistic.num_generalize_blocked += 1;
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
            if frame == 0 {
                return false;
            }
            // println!("{:?}", heap_num);
            // self.statistic();
            heap_num[frame] -= 1;
            if self.trivial_contained(frame, &cube) {
                continue;
            }
            self.statistic.num_rec_block_blocked += 1;
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
        for frame_idx in 1..self.depth() {
            let frame = self.frames[frame_idx].clone();
            for cube in frame {
                self.statistic.num_propagate_blocked += 1;
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
            statistic: Statistic::default(),
            share,
        }
    }

    pub fn check(&mut self) -> bool {
        self.new_frame();
        loop {
            let last_frame_index = self.depth();
            while let SatResult::Sat(model) =
                self.solvers[last_frame_index].solve(&[self.share.aig.bads[0].to_lit()])
            {
                self.statistic.num_get_bad_state += 1;
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

pub fn solve(aig: Aig) -> bool {
    let transition_cnf = aig.get_cnf();
    assert!(aig.latch_init_cube().to_cube().iter().all(|l| l.compl()));
    let state_transform = StateTransform::new(&aig);
    let share = Arc::new(BasicShare {
        aig,
        transition_cnf,
        state_transform,
    });
    let mut pdr = Pdr::new(share);
    pdr.check()
}
