use super::{activity::Activity, statistic::Statistic};
use crate::utils::{generalize::generalize_by_ternary_simulation, state_transform::StateTransform};
use aig::{Aig, AigCube};
use logic_form::{Clause, Cnf, Cube, Lit};
use sat_solver::{minisat::Solver, SatResult, SatSolver, UnsatConflict};
use std::{collections::BinaryHeap, mem::take};

pub struct Pdr {
    aig: Aig,
    transition_cnf: Cnf,
    init_cube: Cube,
    state_transform: StateTransform,
    delta_frames: Vec<Vec<Cube>>,
    solvers: Vec<Solver>,
    activity: Activity,

    statistic: Statistic,
}

impl Pdr {
    fn depth(&self) -> usize {
        self.delta_frames.len() - 1
    }

    fn new_frame(&mut self) {
        let mut solver = Solver::new();
        solver.add_cnf(&self.transition_cnf);
        self.solvers.push(solver);
        self.delta_frames.push(Vec::new());
        self.statistic.num_frames = self.depth();
    }

    fn blocked<'a>(&'a mut self, frame: usize, cube: &Cube) -> BlockResult<'a> {
        self.statistic.num_blocked += 1;
        let mut assumption = self.state_transform.cube_next(cube);
        let act = self.solvers[frame - 1].new_var();
        assumption.push(act);
        let mut tmp_cls = !cube.clone();
        tmp_cls.push(!act);
        self.solvers[frame - 1].add_clause(&tmp_cls);
        match self.solvers[frame - 1].solve(&assumption) {
            SatResult::Sat(_) => {
                let last = assumption.len() - 1;
                let act = !assumption.remove(last);
                self.solvers[frame - 1].release_var(act);
                BlockResult::No(BlockResultNo {
                    solver: &mut self.solvers[frame - 1],
                    aig: &self.aig,
                    assumption,
                })
            }
            SatResult::Unsat(_) => {
                let last = assumption.len() - 1;
                let act = !assumption.remove(last);
                self.solvers[frame - 1].release_var(act);
                BlockResult::Yes(BlockResultYes {
                    solver: &mut self.solvers[frame - 1],
                    state_transform: &self.state_transform,
                    assumption,
                })
            }
        }
    }

    fn frame_add_cube(&mut self, frame: usize, cube: Cube, to_all: bool) {
        assert!(cube.is_sorted_by_key(|x| x.var()));
        let begin = if to_all { 1 } else { frame };
        self.delta_frames[frame].push(cube.clone());
        let clause = !cube;
        for i in begin..=frame {
            self.solvers[i].add_clause(&clause);
        }
    }

    fn down(&mut self, frame: usize, cube: Cube) -> Option<Cube> {
        if cube.subsume(&self.init_cube) {
            return None;
        }
        self.statistic.num_down_blocked += 1;
        match self.blocked(frame, &cube) {
            BlockResult::Yes(conflict) => Some(conflict.get_conflict()),
            BlockResult::No(_) => None,
        }
    }

    // fn ctg_down(&mut self, frame: usize, cube: Cube, rec_depth: usize) -> Option<Cube> {
    //     let ctgs = 0;
    //     loop {
    //         if cube.subsume(&self.init_cube) {
    //             return None;
    //         }
    //         // match self.blocked(frame, &cube, true, true) {
    //         //     (true, conflict) => Some(conflict),
    //         //     (false, cex) => {
    //         //         // if rec_depth > 1 {
    //         //         //     return None;
    //         //         // }
    //         //         // let cex_set: HashSet<Lit> = HashSet::from_iter(cex.into_iter());
    //         //         // cube = cube.into_iter().filter(|l| cex_set.contains(l)).collect();
    //         //         todo!();
    //         //     }
    //         // }
    //     }
    // }

    fn mic(&mut self, frame: usize, mut cube: Cube) -> Cube {
        self.statistic.average_mic_cube_len += cube.len();
        let origin_len = cube.len();
        let mut i = 0;
        assert!(cube.is_sorted_by_key(|x| *x.var()));
        // let mut single_removable = 0;
        // for i in 0..origin_len {
        //     let mut removed_cube = cube.clone();
        //     removed_cube.remove(i);
        //     if let Some(_) = self.down(frame, removed_cube) {
        //         single_removable += 1;
        //     }
        // }
        // self.statistic.average_mic_single_removable_percent +=
        // single_removable as f64 / origin_len as f64;
        cube = self.activity.sort_by_activity_ascending(cube);
        while i < cube.len() {
            let mut removed_cube = cube.clone();
            removed_cube.remove(i);
            match self.down(frame, removed_cube) {
                Some(new_cube) => {
                    cube = new_cube;
                    self.statistic.num_mic_drop_success += 1;
                }
                None => {
                    self.statistic.num_mic_drop_fail += 1;
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
        let cube = self.mic(frame, cube);
        for i in frame + 1..=self.depth() {
            self.statistic.num_generalize_blocked += 1;
            if let BlockResult::No(_) = self.blocked(i, &cube) {
                return (i, cube);
            }
        }
        (self.depth() + 1, cube)
    }

    fn rec_block(&mut self, frame: usize, cube: Cube) -> bool {
        let mut heap = BinaryHeap::new();
        let mut heap_num = vec![0; frame + 1];
        heap.push(HeapFrameCube::new(frame, cube));
        heap_num[frame] += 1;
        while let Some(HeapFrameCube { frame, cube }) = heap.pop() {
            assert!(cube.is_sorted_by_key(|x| x.var()));
            if frame == 0 {
                return false;
            }
            println!("{:?}", heap_num);
            self.statistic();
            heap_num[frame] -= 1;
            self.statistic.num_rec_block_blocked += 1;
            match self.blocked(frame, &cube) {
                BlockResult::Yes(conflict) => {
                    let conflict = conflict.get_conflict();
                    let (frame, core) = self.generalize(frame, conflict);
                    if frame < self.depth() {
                        heap.push(HeapFrameCube::new(frame + 1, cube));
                        heap_num[frame + 1] += 1;
                    }
                    self.frame_add_cube(frame - 1, core, true);
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
            let frame = take(&mut self.delta_frames[frame_idx]);
            for cube in frame {
                self.statistic.num_propagete_blocked += 1;
                match self.blocked(frame_idx + 1, &cube) {
                    BlockResult::Yes(conflict) => {
                        let conflict = conflict.get_conflict();
                        assert!(conflict.len() <= cube.len());
                        let to_all = conflict.len() < cube.len();
                        self.frame_add_cube(frame_idx + 1, conflict, to_all);
                    }
                    BlockResult::No(_) => {
                        // 利用cex？
                        self.delta_frames[frame_idx].push(cube);
                    }
                };
            }
            if self.delta_frames[frame_idx].is_empty() {
                return true;
            }
        }
        for i in 1..=self.depth() {
            self.solvers[i].simplify();
        }
        false
    }
}

impl Pdr {
    pub fn new(aig: Aig) -> Self {
        let mut solvers = vec![Solver::new()];
        let transition_cnf = aig.get_cnf();
        let init_cube = aig.latch_init_cube().to_cube();
        solvers[0].add_cnf(&transition_cnf);
        for l in aig.latchs.iter() {
            solvers[0].add_clause(&Clause::from([Lit::new(l.input.into(), !l.init)]));
        }
        let state_transform = StateTransform::new(&aig);
        let activity = Activity::new(&aig);
        Self {
            aig,
            transition_cnf,
            init_cube,
            state_transform,
            delta_frames: vec![vec![]],
            solvers,
            activity,
            statistic: Statistic::default(),
        }
    }

    pub fn check(&mut self) -> bool {
        self.new_frame();
        loop {
            let last_frame_index = self.depth();
            while let SatResult::Sat(model) =
                self.solvers[last_frame_index].solve(&[self.aig.bads[0].to_lit()])
            {
                self.statistic.num_get_bad_state += 1;
                let cex = generalize_by_ternary_simulation(&self.aig, model, &[self.aig.bads[0]])
                    .to_cube();
                // self.statistic();
                if !self.rec_block(last_frame_index, cex) {
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

impl Pdr {
    fn statistic(&self) {
        for frame in self.delta_frames.iter() {
            print!("{} ", frame.len())
        }
        println!();
        println!("{:?}", self.statistic);
    }
}

enum BlockResult<'a> {
    Yes(BlockResultYes<'a>),
    No(BlockResultNo<'a>),
}

impl<'a> BlockResult<'a> {
    fn map<Y, N>(self, yes: Y, no: N)
    where
        Y: FnOnce(BlockResultYes<'a>),
        N: FnOnce(BlockResultNo<'a>),
    {
        match self {
            BlockResult::Yes(conflict) => yes(conflict),
            BlockResult::No(model) => no(model),
        }
    }
}

struct BlockResultYes<'a> {
    solver: &'a mut Solver,
    state_transform: &'a StateTransform,
    assumption: Cube,
}

impl BlockResultYes<'_> {
    fn get_conflict(mut self) -> Cube {
        let conflict = unsafe { self.solver.get_conflict() };
        let ans = self
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

struct BlockResultNo<'a> {
    solver: &'a mut Solver,
    aig: &'a Aig,
    assumption: Cube,
}

impl BlockResultNo<'_> {
    fn get_model(mut self) -> Cube {
        let model = unsafe { self.solver.get_model() };
        generalize_by_ternary_simulation(
            self.aig,
            model,
            &AigCube::from_cube(take(&mut self.assumption)),
        )
        .to_cube()
    }
}

struct HeapFrameCube {
    frame: usize,
    cube: Cube,
}

impl HeapFrameCube {
    pub fn new(frame: usize, cube: Cube) -> Self {
        Self { frame, cube }
    }
}

impl PartialEq for HeapFrameCube {
    fn eq(&self, other: &Self) -> bool {
        self.frame == other.frame
    }
}

impl PartialOrd for HeapFrameCube {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.frame.partial_cmp(&self.frame)
    }
}

impl Eq for HeapFrameCube {
    fn assert_receiver_is_total_eq(&self) {}
}

impl Ord for HeapFrameCube {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.frame.cmp(&self.frame)
    }
}

pub fn solve(aig: Aig) -> bool {
    let mut pdr = Pdr::new(aig);
    pdr.check()
}
