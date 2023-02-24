use super::pdr::Pdr;

pub mod partition;

pub struct ParallelPdr {
    pdrs: Vec<Pdr>,
}

impl ParallelPdr {
    pub fn check(&mut self) -> bool {
        todo!()
    }
}
