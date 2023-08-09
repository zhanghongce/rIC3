#![feature(assert_matches, is_sorted)]

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
mod worker;

use cex::Cex;
use clap::Parser;
use std::mem::take;
use std::thread::spawn;
use std::time::{Duration, Instant};

use crate::{basic::BasicShare, frames::Frames, statistic::Statistic, worker::PdrWorker};
use crate::{broadcast::create_broadcast, command::Args, utils::state_transform::StateTransform};
use aig::Aig;
use logic_form::{Cube, Lit};
use std::sync::{Arc, Mutex, RwLock};

pub struct Pdr {
    pub frames: Arc<RwLock<Frames>>,
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
        self.frames.write().unwrap().new_frame(broadcast);
    }
}

impl Pdr {
    pub fn new(share: Arc<BasicShare>, num_worker: usize) -> Self {
        let frames = Arc::new(RwLock::new(Frames::new()));
        let (broadcast, mut receivers) = create_broadcast(num_worker + 1);
        let cex_receiver = receivers.pop().unwrap();
        let cex = Arc::new(Mutex::new(Cex::new(share.clone(), cex_receiver)));
        let mut workers = Vec::new();
        for _ in 0..num_worker {
            workers.push(PdrWorker::new(share.clone(), frames.clone(), cex.clone()))
        }
        for (receiver, worker) in receivers.into_iter().zip(workers.iter_mut()) {
            worker.new_frame(receiver)
        }
        frames.write().unwrap().new_frame(broadcast);
        for l in share.aig.latchs.iter() {
            let cube = Cube::from([Lit::new(l.input.into(), !l.init)]);
            frames.write().unwrap().add_cube(0, cube)
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
            let start = Instant::now();
            self.statistic();
            self.new_frame();
            dbg!(start.elapsed());
            let start = Instant::now();
            if self.workers[0].propagate() {
                self.share.statistic.lock().unwrap().overall_propagate_time += start.elapsed();
                self.statistic();
                if self.share.args.parallel == 1 {
                    self.workers[0].cex.lock().unwrap().store_cex();
                }
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
    let mut pdr = Pdr::new(share, args.parallel);
    let start = Instant::now();
    (pdr.check(), start.elapsed())
}

fn main() {
    let args = command::Args::parse();
    // let aig = aig::Aig::from_file("../MC-Benchmark/examples/counter/10bit/counter.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/crafted/paper_v3/paper_v3.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc19/single/aig/goel/opensource/vis_arrays_buf_bug/vis_arrays_buf_bug.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc08/visbakery.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc08/pdtvishuffman7.aag").unwrap();

    // Safe

    // let aig =
    // aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal227/cal227.aag")
    //     .unwrap(); //

    // let aig =
    //     aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/beem/brp2.6.prop3-back-serstep.aag")
    //         .unwrap(); //

    // let aig = aig::Aig::from_file(
    //     "../MC-Benchmark/hwmcc20/aig/2019/beem/pgm_protocol.7.prop1-back-serstep.aag",
    // )
    // .unwrap(); // 911s vs 600s

    // let aig =
    //     aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal143/cal143.aag")
    //         .unwrap(); // 26s vs 10s

    let aig =
        aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal118/cal118.aag")
            .unwrap(); // 37s vs 13s

    // let aig =
    //     aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal102/cal102.aag")
    //         .unwrap(); // 100s vs 88s

    // let aig =
    //     aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal140/cal140.aag")
    //         .unwrap(); // 23s vs 10s

    // let aig =
    //     aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal112/cal112.aag")
    //         .unwrap(); // 167s vs 158s

    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc17/single/intel007.aag").unwrap(); // 21s

    // let aig =
    //     aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/beem/at.6.prop1-back-serstep.aag")
    //         .unwrap(); // 21s

    dbg!(solve(aig, args));
}
