#![allow(non_snake_case)]
#![feature(assert_matches, get_mut_unchecked, format_args_nl)]

mod activity;
pub mod bmc;
mod frame;
pub mod general;
mod gipsat;
pub mod imc;
pub mod kind;
mod mic;
mod options;
pub mod portfolio;
mod proofoblig;
mod statistic;
pub mod transys;
mod verify;
pub mod wl;

use crate::proofoblig::{ProofObligation, ProofObligationQueue};
use crate::statistic::Statistic;
use activity::Activity;
use frame::{Frame, Frames};
use gipsat::Solver;
use logic_form::{Clause, Cube, Lemma, Lit, Var};
pub use options::Options;
use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};
use std::process::exit;
use std::rc::Rc;
use std::time::Instant;
use transys::unroll::TransysUnroll;
use transys::Transys;

pub struct IC3 {
    options: Options,
    ts: Rc<Transys>,
    frame: Frames,
    solvers: Vec<Solver>,
    lift: Solver,
    obligations: ProofObligationQueue,
    activity: Activity,
    statistic: Statistic,
    pre_lemmas: Vec<Clause>,
    abs_cst: Cube,

    auxiliary_var: Vec<Var>,
    xor_var: HashMap<(Lit, Lit), Lit>,
}

impl IC3 {
    #[inline]
    pub fn level(&self) -> usize {
        self.solvers.len() - 1
    }

    fn extend(&mut self) {
        let mut solver = Solver::new(
            self.options.clone(),
            Some(self.frame.len()),
            &self.ts,
            &self.frame,
        );
        for v in self.auxiliary_var.iter() {
            solver.add_domain(*v, true);
        }
        self.solvers.push(solver);
        self.frame.push(Frame::new());
        if self.level() == 0 {
            for init in self.ts.init.clone() {
                self.add_lemma(0, Cube::from([!init]), true, None);
            }
            let mut init = Cube::new();
            for l in self.ts.latchs.iter() {
                if self.ts.init_map[*l].is_none() {
                    if let Some(v) = self.solvers[0].sat_value(l.lit()) {
                        let l = l.lit().not_if(!v);
                        init.push(l);
                    }
                }
            }
            let ts = unsafe { Rc::get_mut_unchecked(&mut self.ts) };
            for i in init {
                ts.add_init(i.var(), Some(i.polarity()));
            }
        } else if self.level() == 1 {
            for cls in self.pre_lemmas.clone().iter() {
                self.add_lemma(1, !cls.clone(), true, None);
            }
        }
    }

    fn push_lemma(&mut self, frame: usize, mut cube: Cube) -> (usize, Cube) {
        for i in frame + 1..=self.level() {
            if let Some(true) = self.solvers[i - 1].inductive(&cube, true, false) {
                cube = self.solvers[i - 1].inductive_core();
            } else {
                return (i, cube);
            }
        }
        (self.level() + 1, cube)
    }

    fn generalize(&mut self, mut po: ProofObligation) -> bool {
        let mut mic = self.solvers[po.frame - 1].inductive_core();
        mic = self.mic(
            po.frame,
            mic,
            if self.options.ic3_options.ctg { 1 } else { 0 },
            &[],
        );
        let (frame, mic) = self.push_lemma(po.frame, mic);
        self.statistic.avg_po_cube_len += po.lemma.len();
        po.frame = frame;
        self.add_obligation(po.clone());
        if self.add_lemma(frame - 1, mic.clone(), false, Some(po)) {
            return true;
        }
        if self.options.ic3_options.xor {
            self.xor_generalize2(frame - 1, mic);
        }
        false
    }

    fn block(&mut self) -> Option<bool> {
        while let Some(mut po) = self.obligations.pop(self.level()) {
            if po.removed {
                continue;
            }
            if self.ts.cube_subsume_init(&po.lemma) {
                if !self.options.ic3_options.abs_cst {
                    assert!(po.frame == 0);
                    return Some(false);
                }
                if let Some(c) = self.check_witness_by_bmc(po.clone()) {
                    assert!(self.options.ic3_options.abs_cst);
                    for c in c {
                        assert!(!self.abs_cst.contains(&c));
                        self.abs_cst.push(c);
                    }
                    if self.options.verbose > 1 {
                        println!("abs cst len: {}", self.abs_cst.len());
                    }
                    self.obligations.clear();
                    for f in self.frame.iter_mut() {
                        for l in f.iter_mut() {
                            l.po = None;
                        }
                    }
                    continue;
                } else {
                    return Some(false);
                }
            }
            if let Some((bf, _)) = self.frame.trivial_contained(po.frame, &po.lemma) {
                po.frame = bf + 1;
                self.add_obligation(po);
                continue;
            }
            if self.options.verbose > 2 {
                self.frame.statistic();
            }
            if self
                .blocked_with_ordered(po.frame, &po.lemma, false, false, false)
                .unwrap()
            {
                if self.generalize(po) {
                    return None;
                }
            } else {
                let (model, inputs) = self.get_predecessor(po.frame, true);
                self.add_obligation(ProofObligation::new(
                    po.frame - 1,
                    Lemma::new(model),
                    inputs,
                    po.depth + 1,
                    Some(po.clone()),
                ));
                self.add_obligation(po);
            }
        }
        Some(true)
    }

    #[allow(unused)]
    fn trivial_block(
        &mut self,
        frame: usize,
        lemma: Lemma,
        constrain: &[Clause],
        limit: &mut usize,
    ) -> bool {
        if frame == 0 {
            return false;
        }
        if self.ts.cube_subsume_init(&lemma) {
            return false;
        }
        if *limit == 0 {
            return false;
        }
        *limit -= 1;
        loop {
            if self
                .blocked_with_ordered_with_constrain(
                    frame,
                    &lemma,
                    false,
                    true,
                    constrain.to_vec(),
                    false,
                )
                .unwrap()
            {
                let mut mic = self.solvers[frame - 1].inductive_core();
                mic = self.mic(frame, mic, 0, constrain);
                let (frame, mic) = self.push_lemma(frame, mic);
                self.add_lemma(frame - 1, mic, false, None);
                return true;
            } else {
                let model = Lemma::new(self.get_predecessor(frame, true).0);
                if !self.trivial_block(frame - 1, model, constrain, limit) {
                    return false;
                }
            }
        }
    }

    fn propagate(&mut self) -> bool {
        for frame_idx in self.frame.early..self.level() {
            self.frame[frame_idx].sort_by_key(|x| x.len());
            let frame = self.frame[frame_idx].clone();
            for mut lemma in frame {
                if self.frame[frame_idx].iter().all(|l| l.ne(&lemma)) {
                    continue;
                }
                for ctp in 0..3 {
                    if self
                        .blocked_with_ordered(frame_idx + 1, &lemma, false, false, false)
                        .unwrap()
                    {
                        let core = self.solvers[frame_idx].inductive_core();
                        if let Some(po) = &mut lemma.po {
                            if po.frame < frame_idx + 2 && self.obligations.remove(po) {
                                po.frame = frame_idx + 2;
                                self.obligations.add(po.clone());
                            }
                        }
                        self.add_lemma(frame_idx + 1, core, true, lemma.po);
                        self.statistic.ctp.statistic(ctp > 0);
                        break;
                    }
                    if !self.options.ic3_options.ctp {
                        break;
                    }
                    let (ctp, _) = self.get_predecessor(frame_idx + 1, false);
                    if !self.ts.cube_subsume_init(&ctp)
                        && self.solvers[frame_idx - 1]
                            .inductive(&ctp, true, false)
                            .unwrap()
                    {
                        let core = self.solvers[frame_idx - 1].inductive_core();
                        let mic = self.mic(frame_idx, core, 0, &[]);
                        self.add_lemma(frame_idx, mic, false, None);
                    } else {
                        break;
                    }
                }
            }
            if self.frame[frame_idx].is_empty() {
                return true;
            }
        }
        self.frame.early = self.level();
        false
    }
}

impl IC3 {
    pub fn new(options: Options, mut ts: Transys, pre_lemmas: Vec<Clause>) -> Self {
        if options.ic3_options.inn {
            let mut uts = TransysUnroll::new(&ts);
            uts.unroll();
            ts = uts.interal_signals();
        }
        let ts = Rc::new(ts);
        let statistic = Statistic::new(&options.model);
        let activity = Activity::new(&ts);
        let frame = Frames::new(&ts);
        let lift = Solver::new(options.clone(), None, &ts, &frame);
        let abs_cst = if options.ic3_options.abs_cst {
            Cube::new()
        } else {
            ts.constraints.clone()
        };
        let mut res = Self {
            options,
            ts,
            activity,
            solvers: Vec::new(),
            lift,
            statistic,
            obligations: ProofObligationQueue::new(),
            frame,
            abs_cst,
            pre_lemmas,
            auxiliary_var: Vec::new(),
            xor_var: HashMap::new(),
        };
        res.extend();
        res
    }

    pub fn check(&mut self) -> bool {
        loop {
            let start = Instant::now();
            loop {
                match self.block() {
                    Some(false) => {
                        self.statistic.overall_block_time += start.elapsed();
                        self.statistic();
                        if self.options.witness {
                            dbg!(self.witness());
                        }
                        return false;
                    }
                    None => {
                        self.statistic.overall_block_time += start.elapsed();
                        self.statistic();
                        if self.options.verify {
                            assert!(self.verify());
                        }
                        return true;
                    }
                    _ => (),
                }
                self.statistic.num_get_bad += 1;
                if let Some((bad, inputs)) = self.get_bad() {
                    let bad = Lemma::new(bad);
                    self.add_obligation(ProofObligation::new(self.level(), bad, inputs, 0, None))
                } else {
                    break;
                }
            }
            let blocked_time = start.elapsed();
            if self.options.verbose > 0 {
                self.frame.statistic();
                println!(
                    "[{}:{}] frame: {}, time: {:?}",
                    file!(),
                    line!(),
                    self.level(),
                    blocked_time,
                );
            }
            self.statistic.overall_block_time += blocked_time;
            self.extend();
            let start = Instant::now();
            let propagate = self.propagate();
            self.statistic.overall_propagate_time += start.elapsed();
            if propagate {
                self.statistic();
                if self.options.verify {
                    assert!(self.verify());
                }
                return true;
            }
        }
    }

    pub fn check_with_int_hanlder(&mut self) -> bool {
        let ic3 = self as *mut IC3 as usize;
        ctrlc::set_handler(move || {
            let ic3 = unsafe { &mut *(ic3 as *mut IC3) };
            ic3.statistic();
            exit(130);
        })
        .unwrap();
        panic::catch_unwind(AssertUnwindSafe(|| self.check())).unwrap_or_else(|_| {
            self.statistic();
            panic!();
        })
    }
}
