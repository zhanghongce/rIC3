use crate::utils::{generalize::generalize_by_ternary_simulation, state_transform::aig_cube_next};
use aig::{Aig, AigClause, AigCube, AigEdge};
use sat_solver::abc_circuit::Solver;
use std::collections::{BTreeSet, HashSet};

pub struct Pdr {
    aig: Aig,
    frames: Vec<HashSet<BTreeSet<AigEdge>>>,
    solvers: Vec<Solver>,
}

impl Pdr {
    fn new_frame(&mut self) {
        println!("pdr new frame: {}", self.frames.len());
        self.solvers.push(Solver::new(&self.aig));
        self.frames.push(HashSet::new());
    }

    fn frame_add_clause(&mut self, frame: usize, clause: AigClause) {
        // dbg!(frame);
        // dbg!(&clause);
        let set = BTreeSet::from_iter(clause.iter().map(|e| *e));
        self.frames[frame].insert(set);
        self.solvers[frame].add_clause(&clause);
    }

    fn can_blocked(&mut self, frame: usize, cube: &AigCube) -> bool {
        let mut assumption = aig_cube_next(&self.aig, cube);
        if frame == 1 {
            assumption.extend(self.aig.latch_init_cube().iter());
        }
        self.solvers[frame - 1].solve(&assumption).is_none()
    }

    fn generalize_blocking_cube(&mut self, frame: usize, mut cube: AigCube) -> AigCube {
        let mut i = 0;
        while i < cube.len() {
            let removed = cube.swap_remove(i);
            if !cube.subsume(&self.aig.latch_init_cube()) {
                if self.can_blocked(frame, &cube) {
                    continue;
                }
            }
            cube.push(removed);
            let last_idx = cube.len() - 1;
            cube.swap(i, last_idx);
            i += 1;
        }
        cube
    }

    fn rec_block(&mut self, frame: usize, s: &AigCube) -> bool {
        // println!("pdr rec block frame {}", n);
        // dbg!(s);
        if frame == 0 {
            return false;
        }
        let mut assumption = aig_cube_next(&self.aig, s);
        // assumption.extend(s.iter().map(|l| !*l));
        self.solvers[frame - 1].add_clause(&!s.clone());
        if frame == 1 {
            assumption.extend(self.aig.latch_init_cube().iter());
        }
        while let Some(cex) = self.solvers[frame - 1].solve(&assumption) {
            let predecessor = generalize_by_ternary_simulation(&self.aig, cex, &assumption);
            if !self.rec_block(frame - 1, &predecessor) {
                return false;
            };
        }
        let clause = !self.generalize_blocking_cube(frame, s.clone());
        for i in 1..=frame {
            self.frame_add_clause(i, clause.clone());
        }
        true
    }

    fn propagate_phase(&mut self) -> bool {
        for frame in 1..self.frames.len() - 1 {
            let mut clause_to_add = Vec::new();
            for lit_set in self.frames[frame].iter() {
                let clause = AigClause::from(Vec::from_iter(lit_set.iter().map(|e| *e)));
                let assumption = aig_cube_next(&self.aig, &!clause.clone());
                if let None = self.solvers[frame].solve(&assumption) {
                    clause_to_add.push(clause);
                }
            }
            for clause in clause_to_add {
                self.frame_add_clause(frame + 1, clause);
            }
            if self.frames[frame] == self.frames[frame + 1] {
                return true;
            }
        }
        false
    }
}

impl Pdr {
    pub fn new(aig: Aig) -> Self {
        let solvers = vec![Solver::new(&aig)];
        Self {
            aig,
            frames: vec![HashSet::new()],
            solvers,
        }
    }

    pub fn solve(&mut self) -> bool {
        self.new_frame();
        loop {
            let last_frame_index = self.frames.len() - 1;
            while let Some(cex) = self.solvers[last_frame_index].solve(&[self.aig.bads[0]]) {
                let cex = generalize_by_ternary_simulation(&self.aig, cex, &[self.aig.bads[0]]);
                if !self.rec_block(last_frame_index, &cex) {
                    return false;
                }
            }
            self.new_frame();
            // dbg!(&self.frames);
            if self.propagate_phase() {
                return true;
            }
        }
    }
}

pub fn solve(aig: Aig) -> bool {
    let mut pdr = Pdr::new(aig);
    pdr.solve()
}
