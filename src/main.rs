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

use crate::{basic::BasicShare, statistic::Statistic, worker::PdrWorker};
use crate::{command::Args, utils::state_transform::StateTransform};
use aig::Aig;
use clap::Parser;
use logic_form::{Cube, Lit};
use std::collections::HashMap;
use std::{
    mem::take,
    sync::{Arc, Mutex},
    thread::spawn,
    time::{Duration, Instant},
};

pub struct Pdr {
    workers: Vec<PdrWorker>,
    pub share: Arc<BasicShare>,
}

impl Pdr {
    pub fn new_frame(&mut self) {
        for worker in self.workers.iter_mut() {
            worker.new_frame()
        }
    }
}

impl Pdr {
    pub fn new(share: Arc<BasicShare>) -> Self {
        let mut workers = Vec::new();
        for _ in 0..share.args.parallel {
            workers.push(PdrWorker::new(share.clone()))
        }
        for worker in workers.iter_mut() {
            worker.new_frame()
        }
        let mut res = Self { workers, share };
        for l in res.share.aig.latchs.iter() {
            if let Some(init) = l.init {
                let cube = Cube::from([Lit::new(l.input.into(), !init)]);
                for worker in res.workers.iter_mut() {
                    worker.add_cube(0, cube.clone())
                }
            }
        }
        res
    }

    pub fn check(&mut self) -> bool {
        if self.workers[0].solvers[0].get_bad().is_some() {
            return false;
        }
        self.new_frame();
        loop {
            let mut joins = Vec::new();
            let workers = take(&mut self.workers);
            let start = Instant::now();
            for mut worker in workers.into_iter() {
                joins.push(spawn(move || {
                    let res = worker.start();
                    (worker, res)
                }));
            }
            for join in joins {
                let (worker, res) = join.join().unwrap();
                if !res {
                    worker.statistic();
                    return false;
                }
                self.workers.push(worker)
            }
            let blocked_time = start.elapsed();
            println!(
                "[{}:{}] frame: {}, time: {:?}",
                file!(),
                line!(),
                self.workers[0].depth(),
                blocked_time,
            );
            self.share.statistic.lock().unwrap().overall_block_time += blocked_time;
            self.statistic();
            self.new_frame();
            let start = Instant::now();
            let propagate = self.workers[0].propagate();
            self.share.statistic.lock().unwrap().overall_propagate_time += start.elapsed();
            if propagate {
                self.statistic();
                assert!(self.workers[0].verify());
                return true;
            }
        }
    }
}

pub fn solve(aig: Aig, args: Args) -> (bool, Duration) {
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
    let mut pdr = Pdr::new(share);
    let start = Instant::now();
    (pdr.check(), start.elapsed())
}

fn main() {
    let args = command::Args::parse();

    let aig = // Safe
    // 1000s vs 0.2s
    // "../MC-Benchmark/hwmcc20/aig/2019/beem/pgm_protocol.7.prop1-back-serstep.aag";
    // 31s vs 17s
    "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal143/cal143.aag";
    // 47s vs 23s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal118/cal118.aag";
    // 131s vs 47s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal102/cal102.aag";
    // 216s vs 73s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal112/cal112.aag";
    // 34s vs 11s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal140/cal140.aag";
    // 21s vs 19s
    // "../MC-Benchmark/hwmcc17/single/intel007.aag";
    // ? vs 141s
    // "../MC-Benchmark/hwmcc17/single/6s0.aag";
    // ? vs 216s
    // "../MC-Benchmark/hwmcc17/single/6s269r.aag";
    // ? vs
    // "../MC-Benchmark/hwmcc17/single/6s281b35.aag";
    // 110s vs 170s
    // "../MC-Benchmark/hwmcc17/single/6s404rb4.aag";
    // 225s vs 260s
    // "../MC-Benchmark/hwmcc17/single/6s109.aag";
    // 61s vs 43s
    // "../MC-Benchmark/hwmcc17/single/bob05.aag";
    // 3s vs 3s
    // "../MC-Benchmark/hwmcc17/single/neclaftp4002.aag";
    //
    // "../MC-Benchmark/hwmcc17/single/nusmvreactorp5.aag";
    // ?
    // "../MC-Benchmark/hwmcc17/single/6s343b08.aag";
    // ?
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal227/cal227.aag";
    // ?
    // "../MC-Benchmark/hwmcc20/aig/2019/beem/brp2.6.prop3-back-serstep.aag";
    // ?
    // "../MC-Benchmark/hwmcc20/aig/2019/beem/at.6.prop1-back-serstep.aag";

    let aig = if let Some(model) = &args.model {
        model
    } else {
        aig
    };

    let aig = aig::Aig::from_file(aig).unwrap();

    dbg!(solve(aig, args));
}
