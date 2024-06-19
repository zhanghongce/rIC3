use super::{cdb::CREF_NONE, Solver};

#[derive(Default)]
pub struct Simplify {
    pub last_num_assign: u32,
}

impl Solver {
    pub fn simplify(&mut self) {
        assert!(self.propagate() == CREF_NONE);
        if self.statistic.num_solve % 1000 == 0 {
            self.simplify_satisfied();
        }
    }
}
