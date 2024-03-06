use crate::{model::Model, Ic3};
use gipsat::{Sat, Solver, Unsat};
use logic_form::{Clause, Cube, Lit};
use satif::{SatResult, Satif, SatifSat, SatifUnsat};
use std::{mem::take, time::Instant};

pub struct Ic3Solver {
    pub solver: Solver,
    temporary: Vec<Cube>,
}

impl Ic3Solver {
    pub fn new(model: &Model, frame: usize) -> Self {
        let solver = Solver::new(
            &format!("frame{frame}"),
            model.num_var,
            &model.trans,
            &model.dependence,
        );
        Self {
            solver,
            temporary: Vec::new(),
        }
    }

    pub fn add_lemma(&mut self, clause: &Clause) {
        let mut cube = !clause;
        cube.sort_by_key(|x| x.var());
        let temporary = take(&mut self.temporary);
        for t in temporary {
            if !cube.ordered_subsume(&t) {
                self.temporary.push(t);
            }
        }
        self.solver.add_lemma(clause);
    }
}

impl Ic3 {
    pub fn blocked(
        &mut self,
        frame: usize,
        cube: &Cube,
        strengthen: bool,
        domain: bool,
    ) -> BlockResult {
        self.statistic.num_sat_inductive += 1;
        let solver_idx = frame - 1;
        let solver = &mut self.solvers[solver_idx].solver;
        let start = Instant::now();
        let assumption = self.model.cube_next(cube);
        let sat_start = Instant::now();
        let res = if strengthen {
            let constrain = !cube;
            solver.solve_with_constrain(&assumption, constrain, domain)
        } else {
            solver.solve_with_domain(&assumption, domain)
        };
        let res = match res {
            SatResult::Sat(sat) => BlockResult::No(BlockResultNo { sat, assumption }),
            SatResult::Unsat(unsat) => BlockResult::Yes(BlockResultYes {
                unsat,
                cube: cube.clone(),
                assumption,
            }),
        };
        self.statistic.avg_sat_call_time += sat_start.elapsed();
        self.statistic.sat_inductive_time += start.elapsed();
        res
    }

    pub fn blocked_with_ordered(
        &mut self,
        frame: usize,
        cube: &Cube,
        ascending: bool,
        strengthen: bool,
        domain: bool,
    ) -> BlockResult {
        let mut ordered_cube = cube.clone();
        self.activity.sort_by_activity(&mut ordered_cube, ascending);
        self.blocked(frame, &ordered_cube, strengthen, domain)
    }
}

pub enum BlockResult {
    Yes(BlockResultYes),
    No(BlockResultNo),
}

pub struct BlockResultYes {
    unsat: Unsat,
    cube: Cube,
    assumption: Cube,
}

pub struct BlockResultNo {
    sat: Sat,
    assumption: Cube,
}

impl BlockResultNo {
    pub fn lit_value(&self, lit: Lit) -> Option<bool> {
        self.sat.lit_value(lit)
    }
}

impl Ic3 {
    pub fn blocked_conflict(&mut self, block: BlockResultYes) -> Cube {
        let mut ans = Cube::new();
        for i in 0..block.cube.len() {
            if block.unsat.has(block.assumption[i]) {
                ans.push(block.cube[i]);
            }
        }
        if self.model.cube_subsume_init(&ans) {
            ans = Cube::new();
            let new = *block
                .cube
                .iter()
                .find(|l| {
                    self.model
                        .init_map
                        .get(&l.var())
                        .is_some_and(|i| *i != l.polarity())
                })
                .unwrap();
            for i in 0..block.cube.len() {
                if block.unsat.has(block.assumption[i]) || block.cube[i] == new {
                    ans.push(block.cube[i]);
                }
            }
            assert!(!self.model.cube_subsume_init(&ans));
        }
        ans
    }

    pub fn unblocked_model(&mut self, unblock: BlockResultNo) -> Cube {
        self.minimal_predecessor(unblock)
    }
}

pub struct Lift {
    solver: minisat::Solver,
    num_act: usize,
}

impl Lift {
    pub fn new(model: &Model) -> Self {
        let mut solver = minisat::Solver::new();
        let false_lit: Lit = solver.new_var().into();
        solver.add_clause(&[!false_lit]);
        model.load_trans(&mut solver);
        Self { solver, num_act: 0 }
    }
}

impl Ic3 {
    pub fn minimal_predecessor(&mut self, unblock: BlockResultNo) -> Cube {
        let start = Instant::now();
        self.lift.num_act += 1;
        if self.lift.num_act > 1000 {
            self.lift = Lift::new(&self.model)
        }
        let act: Lit = self.lift.solver.new_var().into();
        let mut assumption = Cube::from([act]);
        let mut cls = !&unblock.assumption;
        cls.push(!act);
        self.lift.solver.add_clause(&cls);
        for input in self.model.inputs.iter() {
            let lit = input.lit();
            match unblock.sat.lit_value(lit) {
                Some(true) => assumption.push(lit),
                Some(false) => assumption.push(!lit),
                None => (),
            }
        }
        let mut latchs = Cube::new();
        for latch in self.model.latchs.iter() {
            let lit = latch.lit();
            match unblock.sat.lit_value(lit) {
                Some(true) => latchs.push(lit),
                Some(false) => latchs.push(!lit),
                None => (),
            }
        }
        self.activity.sort_by_activity(&mut latchs, false);
        assumption.extend_from_slice(&latchs);
        let res: Cube = match self.lift.solver.solve(&assumption) {
            SatResult::Sat(_) => panic!(),
            SatResult::Unsat(conflict) => latchs.into_iter().filter(|l| conflict.has(*l)).collect(),
        };
        self.lift.solver.add_clause(&[!act]);
        self.statistic.minimal_predecessor_time += start.elapsed();
        res
    }
}
