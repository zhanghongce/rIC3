use logic_form::Cube;
use std::fmt::Debug;
use std::{collections::HashMap, ops::AddAssign};
#[derive(Debug, Default)]
pub struct Statistic {
    pub num_blocked: usize,
    pub num_frames: usize,
    pub num_mic_blocked: usize,
    pub num_generalize_blocked: usize,
    pub num_propagete_blocked: usize,
    pub num_rec_block_blocked: usize,
    pub num_mic_drop_success: usize,
    pub num_mic_drop_fail: usize,
    pub num_get_bad_state: usize,
    pub average_mic_cube_len: StatisticAverage,
}

#[derive(Default)]
pub struct StatisticAverage {
    sum: usize,
    num: usize,
}

impl Debug for StatisticAverage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.sum as f32 / self.num as f32)
    }
}

impl AddAssign<usize> for StatisticAverage {
    fn add_assign(&mut self, rhs: usize) {
        self.sum += rhs;
        self.num += 1;
    }
}
