use super::{cdb::CREF_NONE, Solver};

#[derive(Default)]
pub struct Simplify {
    pub last_num_assign: u32,
    pub last_simplify: usize,
}

impl Solver {
    pub fn simplify(&mut self) {
        assert!(self.propagate() == CREF_NONE);
        if self.statistic.num_solve > self.simplify.last_simplify + 1000 {
            if self.simplify.last_num_assign < self.trail.len() {
            self.simplify_satisfied();
                self.simplify.last_simplify = self.statistic.num_solve;
            }
        }
    }
}
