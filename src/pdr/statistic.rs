use std::{fmt::Debug, ops::AddAssign, time::Duration};

use super::{worker::PdrWorker, Pdr};

#[derive(Debug, Default)]
pub struct Statistic {
    pub num_blocked: usize,
    pub num_sat_solver_restart: usize,
    pub num_down_blocked: usize,
    pub num_mic_drop_success: usize,
    pub num_mic_drop_fail: usize,
    pub num_normal_mic: usize,
    pub num_simple_mic: usize,
    pub num_ctg_down: usize,
    pub num_get_bad_state: usize,
    pub average_mic_cube_len: StatisticAverage,
    pub average_mic_droped_var: StatisticAverage,
    pub average_mic_droped_var_percent: StatisticAverage,
    pub average_mic_single_removable_percent: StatisticAverage,

    pub simple_mic_time: Duration,
    pub mic_time: Duration,
    pub blocked_check_time: Duration,
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
        self.frames.read().unwrap().statistic();
        println!("{:?}", self.share.statistic.lock().unwrap());
    }
}

impl PdrWorker {
    pub fn statistic(&self) {
        self.frames.read().unwrap().statistic();
        println!("{:?}", self.share.statistic.lock().unwrap());
    }
}
