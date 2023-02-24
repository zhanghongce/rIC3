use std::{fmt::Debug, ops::AddAssign};

use super::pdr::Pdr;

#[derive(Debug, Default)]
pub struct Statistic {
    pub num_blocked: usize,
    pub num_frames: usize,
    pub num_sat_solver_restart: usize,
    pub num_down_blocked: usize,
    pub num_generalize_blocked: usize,
    pub num_propagete_blocked: usize,
    pub num_rec_block_blocked: usize,
    pub num_mic_drop_success: usize,
    pub num_mic_drop_fail: usize,
    pub num_normal_mic: usize,
    pub num_simple_mic: usize,
    pub num_get_bad_state: usize,
    pub num_trivial_contained: usize,
    pub num_trivial_contained_success: usize,
    pub average_mic_cube_len: StatisticAverage,
    pub average_mic_droped_var: StatisticAverage,
    pub average_mic_droped_var_percent: StatisticAverage,
    pub average_mic_single_removable_percent: StatisticAverage,
}

#[derive(Default)]
pub struct StatisticAverage {
    sum: f64,
    num: usize,
}

impl Debug for StatisticAverage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.sum as f32 / self.num as f32)
    }
}

impl AddAssign<usize> for StatisticAverage {
    fn add_assign(&mut self, rhs: usize) {
        self.sum += rhs as f64;
        self.num += 1;
    }
}

impl AddAssign<f64> for StatisticAverage {
    fn add_assign(&mut self, rhs: f64) {
        self.sum += rhs;
        self.num += 1;
    }
}

impl Pdr {
    pub fn statistic(&self) {
        for frame in self.frames.iter() {
            print!("{} ", frame.len())
        }
        println!();
        println!("{:?}", self.statistic);
    }
}
