use giputils::statistic::Average;
use std::ops::Add;

#[derive(Debug, Default, Clone, Copy)]
pub struct SolverStatistic {
    pub num_solve: usize,
    pub avg_decide_var: Average,
}

impl Add for SolverStatistic {
    type Output = SolverStatistic;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            num_solve: self.num_solve + rhs.num_solve,
            avg_decide_var: self.avg_decide_var + rhs.avg_decide_var,
        }
    }
}
