#![feature(assert_matches, is_sorted, get_mut_unchecked)]

mod activity;
mod basic;
mod command;
mod frames;
mod mic;
mod solver;
mod statistic;
mod utils;
mod verify;
mod worker;

pub use command::Args;

use crate::utils::state_transform::StateTransform;
use crate::{basic::BasicShare, statistic::Statistic, worker::Ic3Worker};
use aig::Aig;
use logic_form::{Cube, Lit};
use pic3::Synchronizer;
use std::collections::HashMap;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

pub struct Ic3 {
    worker: Ic3Worker,
    pub share: Arc<BasicShare>,
}

impl Ic3 {
    pub fn new_frame(&mut self) {
        self.worker.new_frame()
    }
}

impl Ic3 {
    pub fn new(args: Args, synchronizer: Option<Synchronizer>) -> Self {
        let aig = Aig::from_file(args.model.as_ref().unwrap()).unwrap();
        let transition_cnf = aig.get_cnf();
        let mut init = HashMap::new();
        for l in aig.latch_init_cube().to_cube() {
            init.insert(l.var(), l.polarity());
        }
        let state_transform = StateTransform::new(&aig);
        let share = Arc::new(BasicShare {
            aig,
            transition_cnf,
            state_transform,
            args,
            init,
            statistic: Mutex::new(Statistic::default()),
        });
        let mut worker = Ic3Worker::new(share.clone(), synchronizer);
        worker.new_frame();
        let mut res = Self { worker, share };
        for l in res.share.aig.latchs.iter() {
            if let Some(init) = l.init {
                let cube = Cube::from([Lit::new(l.input.into(), !init)]);
                res.worker.add_cube(0, cube.clone())
            }
        }
        res
    }

    pub fn check(&mut self) -> bool {
        if self.worker.solvers[0].get_bad().is_some() {
            return false;
        }
        self.new_frame();
        loop {
            let start = Instant::now();
            if !self.worker.start() {
                self.worker.statistic();
                return false;
            }
            let blocked_time = start.elapsed();
            let depth = self.worker.depth();
            if let Some(pic3_synchronizer) = self.worker.pic3_synchronizer.as_mut() {
                pic3_synchronizer.frame_blocked(depth);
            }
            println!(
                "[{}:{}] frame: {}, time: {:?}",
                file!(),
                line!(),
                self.worker.depth(),
                blocked_time,
            );
            if let Some(pic3_synchronizer) = self.worker.pic3_synchronizer.as_mut() {
                pic3_synchronizer.sync();
            }
            self.share.statistic.lock().unwrap().overall_block_time += blocked_time;
            // self.statistic();
            self.new_frame();
            let start = Instant::now();
            let propagate = self.worker.propagate();
            self.share.statistic.lock().unwrap().overall_propagate_time += start.elapsed();
            if propagate {
                self.statistic();
                assert!(self.worker.verify());
                return true;
            }
        }
    }
}
