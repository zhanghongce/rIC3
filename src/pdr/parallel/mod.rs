use self::shared_frames::SharedFrames;
use std::{mem::take, sync::Arc, thread::spawn};

use super::{basic::BasicShare, pdr::Pdr};

pub mod partition;
mod shared_frames;

pub struct ParallelPdr {
    pdrs: Vec<Box<Pdr>>,
    frmaes: Arc<SharedFrames>,
    basic: Arc<BasicShare>,
}

impl ParallelPdr {
    fn depth(&self) -> usize {
        todo!()
    }

    fn new_frame(&mut self) {
        for pdr in self.pdrs.iter_mut() {
            pdr.new_frame();
        }
    }

    fn propagate(&mut self) {}
}

impl ParallelPdr {
    pub fn new() -> Self {
        todo!()
    }

    pub fn check(&mut self) -> bool {
        self.new_frame();
        loop {
            // let last_frame = self.depth();
            // let partitioned =
            //     partition::bad_state_partition(&self.basic, &self.pdrs[0].frames[last_frame]);
            // let mut joins = Vec::new();
            // let mut pdrs = take(&mut self.pdrs);
            // for p in partitioned {
            //     let pdr = pdrs.pop().unwrap();
            //     spawn(move || pdr.frames);
            // }
            todo!()
        }
    }
}
