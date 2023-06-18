mod activity;
mod basic;
mod broadcast;
mod frames;
mod mic;
mod solver;
mod statistic;
mod worker;

use self::{basic::BasicShare, frames::Frames, statistic::Statistic, worker::PdrWorker};
use crate::{
    command::Args, pdr::broadcast::create_broadcast, utils::state_transform::StateTransform,
};
use aig::Aig;
use logic_form::{Cube, Lit};
use std::sync::{Arc, Mutex, RwLock};

pub struct Pdr {
    frames: Arc<RwLock<Frames>>,
    workers: Vec<PdrWorker>,
    share: Arc<BasicShare>,
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
