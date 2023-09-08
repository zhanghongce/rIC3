#![feature(assert_matches, is_sorted, get_mut_unchecked)]

mod activity;
mod basic;
mod broadcast;
mod cex;
mod command;
mod frames;
mod mic;
mod solver;
mod statistic;
mod utils;
mod verify;
mod worker;

use crate::{basic::BasicShare, frames::Frames, statistic::Statistic, worker::PdrWorker};
use crate::{broadcast::create_broadcast, command::Args, utils::state_transform::StateTransform};
use aig::Aig;
use cex::Cex;
use clap::Parser;
use logic_form::{Cube, Lit};
use std::{
    mem::take,
    sync::{Arc, Mutex},
    thread::spawn,
    time::{Duration, Instant},
};

pub struct Pdr {
    pub frames: Arc<Frames>,
    workers: Vec<PdrWorker>,
    pub share: Arc<BasicShare>,
}

impl Pdr {
    pub fn new_frame(&mut self) {
        let (broadcast, mut receivers) = create_broadcast(self.workers.len() + 1);
        let cex_receiver = receivers.pop().unwrap();
        for (receiver, worker) in receivers.into_iter().zip(self.workers.iter_mut()) {
            worker.new_frame(receiver)
        }
        self.workers[0].cex.lock().unwrap().new_frame(cex_receiver);
        unsafe { Arc::get_mut_unchecked(&mut self.frames) }.new_frame(broadcast);
    }
}

impl Pdr {
    pub fn new(share: Arc<BasicShare>) -> Self {
        let mut frames = Arc::new(Frames::new());
        let (broadcast, mut receivers) = create_broadcast(share.args.parallel + 1);
        let cex_receiver = receivers.pop().unwrap();
        let cex = Arc::new(Mutex::new(Cex::new(share.clone(), cex_receiver)));
        let mut workers = Vec::new();
        for _ in 0..share.args.parallel {
            workers.push(PdrWorker::new(share.clone(), frames.clone(), cex.clone()))
        }
        for (receiver, worker) in receivers.into_iter().zip(workers.iter_mut()) {
            worker.new_frame(receiver)
        }
        unsafe { Arc::get_mut_unchecked(&mut frames) }.new_frame(broadcast);
        for l in share.aig.latchs.iter() {
            let cube = Cube::from([Lit::new(l.input.into(), !l.init)]);
            frames.add_cube(0, cube)
        }
        Self {
            frames,
            workers,
            share,
        }
    }

    pub fn check(&mut self) -> bool {
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
                    self.statistic();
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
                assert!(self.verify());
                return true;
            }
        }
    }
}

pub fn solve(aig: Aig, args: Args) -> (bool, Duration) {
    let transition_cnf = aig.get_cnf();
    assert!(aig
        .latch_init_cube()
        .to_cube()
        .iter()
        .all(|l| !l.polarity()));
    let state_transform = StateTransform::new(&aig);
    let share = Arc::new(BasicShare {
        aig,
        transition_cnf,
        state_transform,
        args,
        statistic: Mutex::new(Statistic::default()),
    });
    let mut pdr = Pdr::new(share);
    let start = Instant::now();
    (pdr.check(), start.elapsed())
}

fn main() {
    let args = command::Args::parse();

    let aig = // Safe
    // 1000s vs 0.2s vs 500s vs 0.26s
    // "../MC-Benchmark/hwmcc20/aig/2019/beem/pgm_protocol.7.prop1-back-serstep.aag";
    // 35s vs 8.2s vs 10s vs 2.4s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal143/cal143.aag";
    // 44s vs 10s vs 13s vs 3.1s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal118/cal118.aag";
    // 131s vs 47s vs 95s vs 33s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal102/cal102.aag";
    // 216s vs 73s vs 171s vs 54s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal112/cal112.aag";
    // 34s vs 7s vs 10s vs 2s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal140/cal140.aag";
    // 21s vs 19s vs 18s vs 31s
    // "../MC-Benchmark/hwmcc17/single/intel007.aag";
    // ? vs 141s
    // "../MC-Benchmark/hwmcc17/single/6s0.aag";
    // 110s vs 170s
    // "../MC-Benchmark/hwmcc17/single/6s404rb4.aag";
    // 225s vs 260s
    // "../MC-Benchmark/hwmcc17/single/6s109.aag";
    // 71s vs ? 
    // "../MC-Benchmark/hwmcc17/single/bob05.aag";
    // 3s vs 26s 
    // "../MC-Benchmark/hwmcc17/single/neclaftp4002.aag";
    // 
    "../MC-Benchmark/hwmcc17/single/nusmvreactorp5.aag";
    //
    // "../MC-Benchmark/hwmcc17/single/bj08amba5g62.aag";
    // ?
    // "../MC-Benchmark/hwmcc17/single/6s343b08.aag";
    // ?
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal227/cal227.aag";
    // ?
    // "../MC-Benchmark/hwmcc20/aig/2019/beem/brp2.6.prop3-back-serstep.aag";
    // ?
    // "../MC-Benchmark/hwmcc20/aig/2019/beem/at.6.prop1-back-serstep.aag";

    let aig = aig::Aig::from_file(aig).unwrap();

    dbg!(solve(aig, args));
}
