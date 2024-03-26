#![feature(assert_matches, is_sorted, get_mut_unchecked, format_args_nl)]

mod activity;
mod command;
mod mic;
mod proofoblig;
mod solver;
mod statistic;
mod verify;

use crate::proofoblig::{ProofObligation, ProofObligationQueue};
use crate::statistic::Statistic;
use activity::Activity;
use aig::Aig;
pub use command::Args;
use gipsat::GipSAT;
use logic_form::{Cube, Lemma};
use std::panic::{self, AssertUnwindSafe};
use std::process::exit;
use std::time::Instant;
use transys::Transys;

pub struct Ic3 {
    args: Args,
    ts: Transys,
    gipsat: GipSAT,
    activity: Activity,
    obligations: ProofObligationQueue,
    statistic: Statistic,
}

impl Ic3 {
    pub fn level(&self) -> usize {
        self.gipsat.level()
    }

    fn extend(&mut self) {
        self.gipsat.extend();
    }

    fn generalize(&mut self, frame: usize, cube: Cube) -> (usize, Cube) {
        // let level = if self.args.ctg { 1 } else { 0 };
        let mut cube = self.mic(frame, cube, 0);
        for i in frame + 1..=self.level() {
            if self.gipsat.inductive(i, &cube, true) {
                cube = self.gipsat.inductive_core();
            } else {
                return (i, cube);
            }
        }
        (self.level() + 1, cube)
    }

    fn handle_blocked(&mut self, po: ProofObligation) {
        let conflict = self.gipsat.inductive_core();
        let (frame, core) = self.generalize(po.frame, conflict);
        self.statistic.avg_po_cube_len += po.lemma.len();
        self.add_obligation(ProofObligation::new(frame, po.lemma, po.depth));
        self.gipsat.add_lemma(frame - 1, core);
    }

    fn block(&mut self) -> bool {
        while let Some(po) = self.obligations.pop(self.level()) {
            if po.frame == 0 {
                return false;
            }
            assert!(!self.ts.cube_subsume_init(&po.lemma));
            if self.args.verbose_all {
                self.statistic();
            }
            if self.gipsat.trivial_contained(po.frame, &po.lemma) {
                self.add_obligation(ProofObligation::new(po.frame + 1, po.lemma, po.depth));
                continue;
            }
            if self.blocked_with_ordered(po.frame, &po.lemma, false, true) {
                self.handle_blocked(po);
            } else {
                let model = self.gipsat.get_predecessor();
                self.add_obligation(ProofObligation::new(
                    po.frame - 1,
                    Lemma::new(model),
                    po.depth + 1,
                ));
                self.add_obligation(po);
            }
        }
        true
    }
}

impl Ic3 {
    pub fn new(args: Args) -> Self {
        let aig = Aig::from_file(args.model.as_ref().unwrap()).unwrap();
        let ts = Transys::from_aig(&aig);
        let statistic = Statistic::new(args.model.as_ref().unwrap());
        let activity = Activity::new(&ts.latchs);
        let gipsat = GipSAT::new(ts.clone());
        let mut res = Self {
            args,
            ts,
            activity,
            gipsat,
            statistic,
            obligations: ProofObligationQueue::new(),
        };
        res.extend();
        res
    }

    pub fn check(&mut self) -> bool {
        loop {
            let start = Instant::now();
            loop {
                if !self.block() {
                    self.statistic.overall_block_time += start.elapsed();
                    self.statistic();
                    return false;
                }
                if self.gipsat.has_bad() {
                    let bad = Lemma::new(self.gipsat.get_predecessor());
                    self.add_obligation(ProofObligation::new(self.level(), bad, 0))
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
            let propagate = self.gipsat.propagate();
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
        let ic3 = self as *mut Ic3 as usize;
        ctrlc::set_handler(move || {
            let ic3 = unsafe { &mut *(ic3 as *mut Ic3) };
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
