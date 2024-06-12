#![feature(assert_matches, is_sorted, get_mut_unchecked, format_args_nl)]

mod activity;
mod args;
mod frame;
mod gipsat;
mod mic;
mod proofoblig;
mod statistic;
mod verify;

use crate::proofoblig::{ProofObligation, ProofObligationQueue};
use crate::statistic::Statistic;
use activity::Activity;
use aig::Aig;
pub use args::Args;
use frame::Frame;
use gipsat::{GipSAT, Solver};
use logic_form::{Cube, Lemma};
use std::collections::HashSet;
use std::panic::{self, AssertUnwindSafe};
use std::process::exit;
use std::rc::Rc;
use std::time::Instant;
use transys::Transys;

pub struct IC3 {
    args: Args,
    ts: Rc<Transys>,
    gipsat: GipSAT,
    activity: Activity,
    obligations: ProofObligationQueue,
    statistic: Statistic,
    frame: Frame,
}

impl IC3 {
    #[inline]
    pub fn level(&self) -> usize {
        self.gipsat.level()
    }

    fn extend(&mut self) {
        self.gipsat
            .solvers
            .push(Solver::new(Some(self.frame.len()), &self.ts, &self.frame));
        self.frame.push(Vec::new());
        if self.level() == 0 {
            for cube in self.ts.inits() {
                self.add_lemma(0, cube)
            }
        }
    }

    fn push_lemma(&mut self, frame: usize, mut cube: Cube) -> (usize, Cube) {
        for i in frame + 1..=self.level() {
            if self.gipsat.inductive(i, &cube, true) {
                cube = self.gipsat.inductive_core();
            } else {
                return (i, cube);
            }
        }
        (self.level() + 1, cube)
    }

    fn generalize(&mut self, mut po: ProofObligation) {
        let mut mic = self.gipsat.inductive_core();
        let poframe = po.frame;
        // mic.sort();
        // println!("b {:?}", mic);
        let mut keep = HashSet::new();
        if let Some(fa) = &mut po.next {
            if fa.frame == poframe + 1 {
                // fa.last_pred_mic.sort();
                // println!("l {:?}", fa.last_pred_mic);
                for l in fa.last_pred_mic.iter() {
                    keep.insert(*l);
                }
            }
        }
        mic = self.mic(po.frame, mic, 0, &mut keep);
        // mic.sort();
        // println!("m {:?}", mic);
        let (frame, mic) = self.push_lemma(po.frame, mic);
        if let Some(fa) = &mut po.next {
            if fa.frame == poframe + 1 {
                fa.last_pred_mic = mic.clone();
            }
        }
        self.statistic.avg_po_cube_len += po.lemma.len();
        po.frame = frame;
        po.last_pred_mic = Cube::default();
        self.add_obligation(po);
        self.add_lemma(frame - 1, mic);
    }

    fn block(&mut self) -> bool {
        while let Some(mut po) = self.obligations.pop(self.level()) {
            if po.frame == 0 {
                self.add_obligation(po);
                return false;
            }
            assert!(!self.ts.cube_subsume_init(&po.lemma));
            if self.args.verbose_all {
                self.statistic();
            }
            if let Some(bf) = self.frame.trivial_contained(po.frame, &po.lemma) {
                po.frame = bf + 1;
                po.last_pred_mic = Default::default();
                self.add_obligation(po);
                continue;
            }
            if self.blocked_with_ordered(po.frame, &po.lemma, false, false) {
                self.generalize(po);
            } else {
                let model = self.get_predecessor();
                self.add_obligation(ProofObligation::new(
                    po.frame - 1,
                    Lemma::new(model),
                    po.depth + 1,
                    Some(po.clone()),
                ));
                self.add_obligation(po);
            }
        }
        true
    }

    pub fn propagate(&mut self) -> bool {
        for frame_idx in self.frame.early..self.level() {
            self.frame[frame_idx].sort_by_key(|x| x.len());
            let frame = self.frame[frame_idx].clone();
            for lemma in frame {
                if self.frame[frame_idx].iter().all(|l| l.ne(&lemma)) {
                    continue;
                }
                if self.blocked_with_ordered(frame_idx + 1, &lemma, false, false) {
                    let core = self.gipsat.inductive_core();
                    self.add_lemma(frame_idx + 1, core);
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
    pub fn new(args: Args) -> Self {
        let aig = Aig::from_file(args.model.as_ref().unwrap()).unwrap();
        let ts = Rc::new(Transys::from_aig(&aig));
        let statistic = Statistic::new(args.model.as_ref().unwrap());
        let activity = Activity::new(&ts);
        let frame = Frame::new(&ts);
        let gipsat = GipSAT::new(ts.clone(), frame.clone());
        let mut res = Self {
            args,
            ts,
            activity,
            gipsat,
            statistic,
            obligations: ProofObligationQueue::new(),
            frame,
        };
        res.extend();
        res
    }

    pub fn check(&mut self) -> bool {
        // let mut fake_bad_proofoblig = ProofObligation::new(0, Lemma::default(), 0, None);
        loop {
            let start = Instant::now();
            // fake_bad_proofoblig.frame = self.level() + 1;
            loop {
                if !self.block() {
                    self.statistic.overall_block_time += start.elapsed();
                    self.statistic();
                    if self.args.witness {
                        dbg!(self.witness());
                    }
                    return false;
                }
                if self.gipsat.has_bad() {
                    let bad = Lemma::new(self.get_predecessor());
                    self.add_obligation(ProofObligation::new(self.level(), bad, 0, None))
                } else {
                    break;
                }
            }
            let blocked_time = start.elapsed();
            if self.args.verbose {
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
                if self.args.verify {
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
