#![feature(assert_matches, is_sorted)]

mod activity;
mod basic;
mod broadcast;
mod command;
mod frames;
mod mic;
mod solver;
mod statistic;
mod utils;
mod worker;

use clap::Parser;
use std::time::Instant;

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
        let (broadcast, receivers) = create_broadcast(self.workers.len());
        for (receiver, worker) in receivers.into_iter().zip(self.workers.iter_mut()) {
            worker.new_frame(receiver)
        }
        self.frames.write().unwrap().new_frame(broadcast);
        if self.frames.read().unwrap().frames.len() == 1 {
            for l in self.share.aig.latchs.iter() {
                let cube = Cube::from([Lit::new(l.input.into(), l.init)]);
                self.frames.write().unwrap().add_cube(0, cube)
            }
        }
    }
}

impl Pdr {
    pub fn new(share: Arc<BasicShare>) -> Self {
        let frames = Arc::new(RwLock::new(Frames::new()));
        let mut workers = Vec::new();
        for _ in 0..1 {
            workers.push(PdrWorker::new(share.clone(), frames.clone()))
        }
        let mut ret = Self {
            frames,
            workers,
            share,
        };
        ret.new_frame();
        ret
    }

    pub fn check(&mut self) -> bool {
        self.new_frame();
        loop {
            while let Some(cex) = self.workers[0].get_cex() {
                if !self.workers[0].block(cex) {
                    self.statistic();
                    return false;
                }
            }
            self.statistic();
            self.new_frame();
            if self.workers[0].propagate() {
                self.statistic();
                return true;
            }
        }
    }
}

pub fn solve(aig: Aig, args: Args) -> bool {
    let transition_cnf = aig.get_cnf();
    assert!(aig.latch_init_cube().to_cube().iter().all(|l| l.compl()));
    let state_transform = StateTransform::new(&aig);
    let share = Arc::new(BasicShare {
        aig,
        transition_cnf,
        state_transform,
        args,
        statistic: Mutex::new(Statistic::default()),
    });
    let mut pdr = Pdr::new(share);
    pdr.check()
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

    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/beem/at.6.prop1-back-serstep.aag").unwrap(); // 21s

    let start = Instant::now();
    dbg!(solve(aig, args));
    println!("{:?}", start.elapsed());
}
