use super::pdr::Pdr;

pub mod partition;

pub struct ParallelPdr {
    pdrs: Vec<Pdr>,
}

impl ParallelPdr {
    fn new_frame(&mut self) {
        for pdr in self.pdrs.iter_mut() {
            pdr.new_frame();
        }
    }
}

impl ParallelPdr {
    pub fn check(&mut self) -> bool {
        self.new_frame();
        todo!()
    }
}
