use crate::utils::{generalize::generalize_by_ternary_simulation, state_transform::StateTransform};
use aig::{Aig, AigCube};
use logic_form::{Cnf, Cube, Dnf, Lit};
use sat_solver::{minisat::Solver, SatResult, SatSolver, UnsatConflict};
use std::{collections::BinaryHeap, fmt::Debug, mem::take, ops::AddAssign};

pub struct Pdr {
    aig: Aig,
    transition_cnf: Cnf,
    state_transform: StateTransform,
    delta_frames: Vec<Vec<Cube>>,
    solvers: Vec<Solver>,

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

    fn trivial_blocked(&self, cube: Cube) {}

    fn blocked(
        &mut self,
        frame: usize,
        cube: &Cube,
        need_cex: bool,
        need_conflict: bool,
    ) -> (bool, Cube) {
        self.statistic.num_blocked += 1;
        let mut assumption = self.state_transform.cube_next(cube);
        let act = self.solvers[frame - 1].new_var();
        assumption.push(act);
        let mut tmp_cls = !cube.clone();
        tmp_cls.push(!act);
        self.solvers[frame - 1].add_clause(&tmp_cls);
        match self.solvers[frame - 1].solve(&assumption) {
            SatResult::Sat(model) => {
                if need_cex {
                    let last = assumption.len() - 1;
                    let act = !assumption.remove(last);
                    let cex = generalize_by_ternary_simulation(
                        &self.aig,
                        model,
                        &AigCube::from_cube(assumption),
                    )
                    .to_cube();
                    self.solvers[frame - 1].release_var(act);
                    (false, cex)
                } else {
                    (false, Cube::new())
                }
            }
            SatResult::Unsat(conflict) => {
                if need_conflict {
                    let last = assumption.len() - 1;
                    let act = !assumption.remove(last);
                    let ans = self
                        .state_transform
                        .previous(assumption.into_iter().filter(|l| conflict.has_lit(!*l)))
                        .collect();
                    self.solvers[frame - 1].release_var(act);
                    (true, ans)
                } else {
                    (true, Cube::new())
                }
            }
        }
    }

    fn frame_add_cube(&mut self, frame: usize, cube: Cube, to_all: bool) {
        let begin = if to_all { 1 } else { frame };
        self.delta_frames[frame].push(cube.clone());
        let clause = !cube;
        for i in begin..=frame {
            self.solvers[i].add_clause(&clause);
        }
    }

    fn mic(&mut self, frame: usize, mut cube: Cube) -> Cube {
        self.statistic.average_mic_cube_len += cube.len();
        let mut i = 0;
        cube.sort_by_key(|x| *x.var());
        while i < cube.len() {
            // cube.sort_by(|x, y| x.var().cmp(&y.var()));
            let removed = cube.remove(i);
            // cube.sort_by(|x, y| x.var().cmp(&y.var()));
            if !cube.subsume(&self.aig.latch_init_cube().to_cube()) {
                self.statistic.num_mic_blocked += 1;
                if let (true, conflict) = self.blocked(frame, &cube, false, true) {
                    self.statistic.num_mic_drop_success += 1;
                    for j in 0..i {
                        assert!(conflict[j] == cube[j]);
                    }
                    if conflict.len() < cube.len() {
                        cube = conflict;
                    }
                    continue;
                }
                self.statistic.num_mic_drop_fail += 1;
            }
            cube.insert(i, removed);
            // let last_idx = cube.len() - 1;
            // cube.swap(i, last_idx);
            i += 1;
        }
        cube
    }

    fn generalize(&mut self, frame: usize, cube: Cube) -> (usize, Cube) {
        let cube = self.mic(frame, cube);
        for i in frame + 1..=self.depth() {
            self.statistic.num_generalize_blocked += 1;
            if let (false, _) = self.blocked(i, &cube, false, false) {
                return (i, cube);
            }
        }
        (self.depth() + 1, cube)
    }

    fn rec_block(&mut self, frame: usize, cube: Cube) -> bool {
        let mut heap = BinaryHeap::new();
        heap.push(HeapFrameCube::new(frame, cube));
        while let Some(HeapFrameCube { frame, cube }) = heap.pop() {
            // dbg!(heap.len());
            if frame == 0 {
                return false;
            }

            self.statistic.num_rec_block_blocked += 1;
            match self.blocked(frame, &cube, true, true) {
                (true, conflict) => {
                    let (frame, core) = self.generalize(frame, conflict);
                    if frame < self.depth() {
                        heap.push(HeapFrameCube::new(frame + 1, cube));
                    }
                    self.frame_add_cube(frame - 1, core, true);
                }
                (false, cex) => {
                    heap.push(HeapFrameCube::new(frame - 1, cex));
                    heap.push(HeapFrameCube::new(frame, cube));
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
                if let (true, conflict) = self.blocked(frame_idx + 1, &cube, false, true) {
                    assert!(conflict.len() <= cube.len());
                    let to_all = conflict.len() < cube.len();
                    self.frame_add_cube(frame_idx + 1, conflict, to_all);
                } else {
                    // 利用cex？
                    self.delta_frames[frame_idx].push(cube);
                }
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
        let init_iter = aig
            .latchs
            .iter()
            .map(|l| Cube::from([Lit::new(l.input.into(), l.init)]));
        let init = Dnf::from_iter(init_iter.clone());
        solvers[0].add_cnf(&transition_cnf);
        solvers[0].add_cnf(&!init);
        let init = Vec::from_iter(init_iter);
        let state_transform = StateTransform::new(&aig);
        Self {
            aig,
            transition_cnf,
            state_transform,
            delta_frames: vec![init],
            solvers,
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
                if !self.rec_block(last_frame_index, cex) {
                    dbg!(&self.statistic);
                    return false;
                }
            }
            dbg!(&self.statistic);
            self.new_frame();
            if self.propagate() {
                dbg!(&self.statistic);
                return true;
            }
        }
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

#[derive(Debug, Default)]
struct Statistic {
    num_blocked: usize,
    num_frames: usize,
    num_mic_blocked: usize,
    num_generalize_blocked: usize,
    num_propagete_blocked: usize,
    num_rec_block_blocked: usize,
    num_mic_drop_success: usize,
    num_mic_drop_fail: usize,
    num_get_bad_state: usize,
    average_mic_cube_len: StatisticAverage,
}

#[derive(Default)]
struct StatisticAverage {
    sum: usize,
    num: usize,
}

impl Debug for StatisticAverage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.sum as f32 / self.num as f32)
    }
}

impl AddAssign<usize> for StatisticAverage {
    fn add_assign(&mut self, rhs: usize) {
        self.sum += rhs;
        self.num += 1;
    }
}

pub fn solve(aig: Aig) -> bool {
    let mut pdr = Pdr::new(aig);
    pdr.check()
}
