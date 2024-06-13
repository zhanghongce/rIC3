mod analyze;
mod cdb;
mod domain;
mod propagate;
mod search;
mod simplify;
mod statistic;
mod utils;
mod vsids;

pub use cdb::{CRef, CREF_NONE};

use crate::{frame::Frame, IC3};
use analyze::Analyze;
use cdb::{ClauseDB, ClauseKind};
use domain::Domain;
use giputils::gvec::Gvec;
use logic_form::{Clause, Cnf, Cube, Lit, LitSet, Var, VarMap};
use propagate::Watchers;
use rand::{prelude::SliceRandom, rngs::StdRng, SeedableRng};
use satif::{SatResult, SatifSat, SatifUnsat};
use search::Value;
use simplify::Simplify;
use statistic::{GipSATStatistic, SolverStatistic};
use std::{
    collections::HashSet,
    mem::take,
    ops::{Deref, DerefMut},
    rc::Rc,
    time::Instant,
};
use transys::Transys;
use utils::Lbool;
use vsids::Vsids;

pub struct Solver {
    _id: Option<usize>,
    cdb: ClauseDB,
    watchers: Watchers,
    value: Value,
    trail: Gvec<Lit>,
    pos_in_trail: Vec<u32>,
    level: VarMap<u32>,
    reason: VarMap<CRef>,
    propagated: u32,
    vsids: Vsids,
    phase_saving: VarMap<Lbool>,
    analyze: Analyze,
    simplify: Simplify,
    unsat_core: LitSet,
    domain: Domain,
    temporary_domain: bool,
    prepared_vsids: bool,
    constrain_act: Var,

    ts: Rc<Transys>,
    _frame: Frame,

    rng: StdRng,
    statistic: SolverStatistic,
}

impl Solver {
    pub fn new(id: Option<usize>, ts: &Rc<Transys>, frame: &Frame) -> Self {
        let mut solver = Self {
            _id: id,
            ts: ts.clone(),
            _frame: frame.clone(),
            cdb: Default::default(),
            watchers: Default::default(),
            value: Default::default(),
            trail: Default::default(),
            pos_in_trail: Default::default(),
            level: Default::default(),
            reason: Default::default(),
            propagated: Default::default(),
            vsids: Default::default(),
            phase_saving: Default::default(),
            analyze: Default::default(),
            simplify: Default::default(),
            unsat_core: Default::default(),
            domain: Domain::new(),
            temporary_domain: Default::default(),
            prepared_vsids: false,
            statistic: Default::default(),
            constrain_act: Var(0),
            rng: StdRng::seed_from_u64(0),
        };
        while solver.num_var() < solver.ts.num_var {
            solver.new_var();
        }
        for cls in ts.trans.iter() {
            solver.add_clause_inner(cls, ClauseKind::Trans);
        }
        if id.is_some() {
            for c in ts.constraints.iter() {
                solver.add_clause_inner(&[*c], ClauseKind::Trans);
            }
        }
        assert!(solver.highest_level() == 0);
        assert!(solver.propagate() == CREF_NONE);
        solver.simplify_satisfied();
        if id.is_some() {
            solver.domain.calculate_constrain(&solver.ts, &solver.value);
        }
        solver
    }

    pub fn new_var(&mut self) -> Var {
        self.reset();
        let v = self.constrain_act;
        let var = Var::new(self.num_var() + 1);
        self.value.reserve(var);
        self.level.reserve(var);
        self.reason.reserve(var);
        self.watchers.reserve(var);
        self.vsids.reserve(var);
        self.phase_saving.reserve(var);
        self.analyze.reserve(var);
        self.unsat_core.reserve(var);
        self.domain.reserve(var);
        self.constrain_act = var;
        v
    }

    #[inline]
    pub fn num_var(&self) -> usize {
        self.constrain_act.into()
    }

    fn simplify_clause(&mut self, cls: &[Lit]) -> Option<logic_form::Clause> {
        assert!(self.highest_level() == 0);
        let mut clause = logic_form::Clause::new();
        for l in cls.iter() {
            assert!(self.num_var() + 1 > l.var().into());
            match self.value.v(*l) {
                Lbool::TRUE => return None,
                Lbool::FALSE => (),
                _ => clause.push(*l),
            }
        }
        assert!(!clause.is_empty());
        Some(clause)
    }

    fn add_clause_inner(&mut self, clause: &[Lit], mut kind: ClauseKind) -> CRef {
        let clause = match self.simplify_clause(clause) {
            Some(clause) => clause,
            None => return CREF_NONE,
        };
        for l in clause.iter() {
            if self.constrain_act == l.var() {
                kind = ClauseKind::Temporary;
            }
        }
        if clause.len() == 1 {
            assert!(!matches!(kind, ClauseKind::Temporary));
            match self.value.v(clause[0]) {
                Lbool::TRUE | Lbool::FALSE => todo!(),
                _ => {
                    self.assign(clause[0], CREF_NONE);
                    assert!(self.propagate() == CREF_NONE);
                    CREF_NONE
                }
            }
        } else {
            self.attach_clause(&clause, kind)
        }
    }

    #[inline]
    pub fn add_lemma(&mut self, lemma: &[Lit]) -> CRef {
        self.reset();
        for l in lemma.iter() {
            self.domain.add_domain(l.var());
            assert!(self.ts.dependence[l.var()].is_empty());
            // let mut queue = Vec::new();
            // queue.push(l.var());
            // while let Some(v) = queue.pop() {
            //     for d in self.ts.dependence[v].iter() {
            //         if !self.domain.domain.has(*d) {
            //             self.domain.add_domain(*d);
            //             queue.push(*d);
            //         }
            //     }
            // }
        }
        self.add_clause_inner(lemma, ClauseKind::Lemma)
    }

    fn reset(&mut self) {
        self.backtrack(0, false);
        self.clean_temporary();
        self.prepared_vsids = false;
        self.domain.reset();
        assert!(!self.temporary_domain);
    }

    fn new_round(
        &mut self,
        domain: impl Iterator<Item = Var>,
        constrain: Vec<Clause>,
        bucket: bool,
    ) -> bool {
        self.backtrack(0, self.temporary_domain);
        self.clean_temporary();
        self.prepared_vsids = false;
        // dbg!(&self.name);
        // self.vsids.activity.print();
        // dbg!(self.num_var());
        // dbg!(self.trail.len());
        // dbg!(self.cdb.num_leanrt());
        // dbg!(self.cdb.num_lemma());

        for mut c in constrain {
            c.push(!self.constrain_act.lit());
            if let Some(c) = self.simplify_clause(&c) {
                if c.len() == 1 {
                    return false;
                }
                self.add_clause_inner(&c, ClauseKind::Temporary);
            }
        }

        if !self.temporary_domain {
            self.domain.enable_local(domain, &self.ts, &self.value);
            assert!(!self.domain.domain.has(self.constrain_act));
            self.domain.domain.insert(self.constrain_act);
            if bucket {
                self.vsids.enable_bucket = true;
                self.vsids.bucket.clear();
            } else {
                self.vsids.enable_bucket = false;
                self.vsids.heap.clear();
            }
        }
        self.statistic.avg_decide_var += self.domain.domains().len() as f64
            / (self.ts.num_var - self.trail.len() as usize) as f64;
        true
    }

    pub fn solve_with_domain(
        &mut self,
        assump: &[Lit],
        constrain: Vec<Clause>,
        bucket: bool,
    ) -> SatResult<Sat, Unsat> {
        assert!(!assump.is_empty());
        if self.temporary_domain {
            assert!(bucket);
        }
        let mut assumption;
        let assump = if !constrain.is_empty() {
            assumption = Cube::new();
            assumption.push(self.constrain_act.lit());
            assumption.extend_from_slice(assump);
            let mut cc = Vec::new();
            for c in constrain.iter() {
                for l in c.iter() {
                    cc.push(*l);
                }
            }
            if !self.new_round(
                assump.iter().chain(cc.iter()).map(|l| l.var()),
                constrain,
                bucket,
            ) {
                self.unsat_core.clear();
                return SatResult::Unsat(Unsat { solver: self });
            };
            &assumption
        } else {
            assert!(self.new_round(assump.iter().map(|l| l.var()), vec![], bucket));
            assump
        };
        self.statistic.num_solve += 1;
        self.clean_leanrt(true);
        self.simplify();
        self.garbage_collect();
        self.search_with_restart(assump)
    }

    pub fn set_domain(&mut self, domain: impl Iterator<Item = Lit>) {
        self.reset();
        self.temporary_domain = true;
        self.domain
            .enable_local(domain.map(|l| l.var()), &self.ts, &self.value);
        assert!(!self.domain.domain.has(self.constrain_act));
        self.domain.domain.insert(self.constrain_act);
        self.vsids.enable_bucket = true;
        self.vsids.bucket.clear();
        for d in self.domain.domains() {
            self.vsids.push(*d);
        }
    }

    pub fn unset_domain(&mut self) {
        self.temporary_domain = false;
    }
}

pub struct Sat {
    solver: *mut Solver,
}

impl SatifSat for Sat {
    #[inline]
    fn lit_value(&self, lit: Lit) -> Option<bool> {
        let solver = unsafe { &*self.solver };
        match solver.value.v(lit) {
            Lbool::TRUE => Some(true),
            Lbool::FALSE => Some(false),
            _ => None,
        }
    }
}

pub struct Unsat {
    solver: *mut Solver,
}

impl SatifUnsat for Unsat {
    #[inline]
    fn has(&self, lit: Lit) -> bool {
        let solver = unsafe { &*self.solver };
        solver.unsat_core.has(lit)
    }
}

pub enum BlockResult {
    Yes(BlockResultYes),
    No(BlockResultNo),
}

pub struct BlockResultYes {
    pub unsat: Unsat,
    pub cube: Cube,
    pub assumption: Cube,
}

pub struct BlockResultNo {
    pub sat: Sat,
    pub assumption: Cube,
}

impl BlockResultNo {
    #[inline]
    pub fn lit_value(&self, lit: Lit) -> Option<bool> {
        self.sat.lit_value(lit)
    }
}

pub struct GipSAT {
    ts: Rc<Transys>,
    pub solvers: Vec<Solver>,
    lift: Solver,
    last_ind: Option<BlockResult>,
    statistic: GipSATStatistic,
}

impl GipSAT {
    /// create a new GipSAT instance from a transition system
    pub fn new(ts: Rc<Transys>, frame: Frame) -> Self {
        let lift = Solver::new(None, &ts, &frame);
        Self {
            ts,
            solvers: Default::default(),
            lift,
            last_ind: None,
            statistic: Default::default(),
        }
    }

    #[inline]
    pub fn level(&self) -> usize {
        self.solvers.len() - 1
    }

    pub fn inductive_with_constrain(
        &mut self,
        frame: usize,
        cube: &[Lit],
        strengthen: bool,
        mut constrain: Vec<Clause>,
    ) -> bool {
        let start = Instant::now();
        self.statistic.num_sat += 1;
        let solver_idx = frame - 1;
        let assumption = self.ts.cube_next(cube);
        if strengthen {
            constrain.push(Clause::from_iter(cube.iter().map(|l| !*l)));
        }
        self.last_ind = Some(
            match self.solvers[solver_idx].solve_with_domain(&assumption, constrain, true) {
                SatResult::Sat(sat) => BlockResult::No(BlockResultNo { sat, assumption }),
                SatResult::Unsat(unsat) => BlockResult::Yes(BlockResultYes {
                    unsat,
                    cube: Cube::from(cube),
                    assumption,
                }),
            },
        );
        self.statistic.avg_sat_time += start.elapsed();
        matches!(self.last_ind.as_ref().unwrap(), BlockResult::Yes(_))
    }

    pub fn inductive(&mut self, frame: usize, cube: &[Lit], strengthen: bool) -> bool {
        self.inductive_with_constrain(frame, cube, strengthen, vec![])
    }

    pub fn inductive_core(&mut self) -> Cube {
        let last_ind = take(&mut self.last_ind);
        let block = match last_ind.unwrap() {
            BlockResult::Yes(block) => block,
            BlockResult::No(_) => panic!(),
        };
        let mut ans = Cube::new();
        for i in 0..block.cube.len() {
            if block.unsat.has(block.assumption[i]) {
                ans.push(block.cube[i]);
            }
        }
        if self.ts.cube_subsume_init(&ans) {
            ans = Cube::new();
            let new = *block
                .cube
                .iter()
                .find(|l| self.ts.init_map[l.var()].is_some_and(|i| i != l.polarity()))
                .unwrap();
            for i in 0..block.cube.len() {
                if block.unsat.has(block.assumption[i]) || block.cube[i] == new {
                    ans.push(block.cube[i]);
                }
            }
            assert!(!self.ts.cube_subsume_init(&ans));
        }
        ans
    }

    pub fn unblocked_value(&self, lit: Lit) -> Option<bool> {
        let unblock = match self.last_ind.as_ref().unwrap() {
            BlockResult::Yes(_) => panic!(),
            BlockResult::No(unblock) => unblock,
        };
        unblock.lit_value(lit)
    }

    pub fn has_bad(&mut self) -> bool {
        let start = Instant::now();
        self.statistic.num_sat += 1;
        let res =
            match self
                .solvers
                .last_mut()
                .unwrap()
                .solve_with_domain(&[self.ts.bad], vec![], false)
            {
                SatResult::Sat(sat) => {
                    self.last_ind = Some(BlockResult::No(BlockResultNo {
                        sat,
                        assumption: Cube::from([self.ts.bad]),
                    }));
                    true
                }
                SatResult::Unsat(_) => false,
            };
        self.statistic.avg_sat_time += start.elapsed();
        res
    }

    pub fn set_domain(&mut self, frame: usize, domain: impl Iterator<Item = Lit>) {
        self.solvers[frame].set_domain(domain)
    }

    pub fn unset_domain(&mut self, frame: usize) {
        self.solvers[frame].unset_domain()
    }

    pub fn statistic(&self) {
        println!();
        let mut statistic = SolverStatistic::default();
        for s in self.solvers.iter() {
            statistic = statistic + s.statistic;
        }
        println!("{:#?}", statistic);
        println!("{:#?}", self.statistic);
    }
}

impl Deref for GipSAT {
    type Target = [Solver];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.solvers
    }
}

impl DerefMut for GipSAT {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.solvers
    }
}

impl IC3 {
    pub fn blocked_with_ordered(
        &mut self,
        frame: usize,
        cube: &Cube,
        ascending: bool,
        strengthen: bool,
    ) -> bool {
        let mut ordered_cube = cube.clone();
        self.activity.sort_by_activity(&mut ordered_cube, ascending);
        self.gipsat.inductive(frame, &ordered_cube, strengthen)
    }

    pub fn blocked_with_ordered_with_constrain(
        &mut self,
        frame: usize,
        cube: &Cube,
        ascending: bool,
        strengthen: bool,
        constrain: Vec<Clause>,
    ) -> bool {
        let mut ordered_cube = cube.clone();
        self.activity.sort_by_activity(&mut ordered_cube, ascending);
        self.gipsat
            .inductive_with_constrain(frame, &ordered_cube, strengthen, constrain)
    }

    pub fn get_predecessor(&mut self) -> Cube {
        let last_ind = take(&mut self.gipsat.last_ind);
        let BlockResult::No(unblock) = last_ind.unwrap() else {
            panic!()
        };
        let mut assumption = Cube::new();
        let mut cls = unblock.assumption.clone();
        cls.extend_from_slice(&self.ts.constraints);
        let in_cls: HashSet<Var> = HashSet::from_iter(cls.iter().map(|l| l.var()));
        let cls = !cls;
        for input in self.ts.inputs.iter() {
            let lit = input.lit();
            match unblock.sat.lit_value(lit) {
                Some(true) => assumption.push(lit),
                Some(false) => assumption.push(!lit),
                None => (),
            }
        }
        let mut latchs = Cube::new();
        for latch in self.ts.latchs.iter() {
            let lit = latch.lit();
            if let Some(v) = unblock.sat.lit_value(lit) {
                let solver = unsafe { &mut *unblock.sat.solver };
                if in_cls.contains(latch) || !solver.flip_to_none(*latch) {
                    latchs.push(lit.not_if(!v));
                }
            }
        }
        self.activity.sort_by_activity(&mut latchs, false);
        let mut res = latchs;
        for i in 0..5 {
            if i == 1 {
                res.reverse();
            } else if i > 1 {
                res.shuffle(&mut self.gipsat.lift.rng);
            }
            let mut lift_assump = assumption.clone();
            lift_assump.extend_from_slice(&res);
            let constrain = vec![cls.clone()];
            let SatResult::Unsat(conflict) =
                self.gipsat
                    .lift
                    .solve_with_domain(&lift_assump, constrain, false)
            else {
                panic!();
            };
            let olen = res.len();
            res = res.into_iter().filter(|l| conflict.has(*l)).collect();
            if res.len() == olen {
                break;
            }
        }
        res
    }

    pub fn new_var(&mut self) -> Var {
        let ts = unsafe { Rc::get_mut_unchecked(&mut self.ts) };
        let var = ts.new_var();
        for s in self.gipsat.solvers.iter_mut() {
            assert!(var == s.new_var());
        }
        assert!(var == self.gipsat.lift.new_var());
        var
    }

    pub fn add_latch(
        &mut self,
        state: Var,
        next: Lit,
        init: Option<bool>,
        trans: Cnf,
        dep: Vec<Var>,
        dep_next: Vec<Var>,
    ) {
        let ts = unsafe { Rc::get_mut_unchecked(&mut self.ts) };
        ts.add_latch(state, next, init, trans.clone(), dep, dep_next);
        let tmp_lit_set = unsafe { Rc::get_mut_unchecked(&mut self.frame.tmp_lit_set) };
        tmp_lit_set.reserve(self.ts.max_latch);
        for s in self.gipsat.solvers.iter_mut() {
            for cls in trans.iter() {
                s.add_clause_inner(cls, ClauseKind::Trans);
            }
        }
        for cls in trans.iter() {
            self.gipsat.lift.add_clause_inner(cls, ClauseKind::Trans);
        }
    }
}
