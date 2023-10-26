use crate::Ic3;
use std::{fmt::Debug, ops::AddAssign, time::Duration};

#[derive(Debug, Default)]
pub struct Statistic {
    pub num_blocked: usize,
    pub num_mic: usize,
    pub num_solver_restart: usize,
    pub num_down_blocked: usize,
    pub mic_drop: SuccessRate,
    pub num_ctg_down: usize,
    pub num_get_bad_state: usize,
    pub average_mic_cube_len: StatisticAverage,

    pub simple_mic_time: Duration,
    pub mic_time: Duration,
    pub blocked_check_time: Duration,

    pub overall_block_time: Duration,
    pub overall_propagate_time: Duration,
}

#[derive(Default)]
pub struct StatisticAverage {
    sum: f64,
    num: usize,
}

impl Debug for StatisticAverage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.sum as f32 / self.num as f32)
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

#[derive(Default)]
pub struct SuccessRate {
    succ: usize,
    fail: usize,
}

impl SuccessRate {
    pub fn success(&mut self) {
        self.succ += 1;
    }

    pub fn fail(&mut self) {
        self.fail += 1;
    }

    pub fn statistic(&mut self, success: bool) {
        if success {
            self.success()
        } else {
            self.fail()
        }
    }
}

impl Debug for SuccessRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "success: {}, fail: {}, success rate: {:.2}",
            self.succ,
            self.fail,
            self.succ as f64 / (self.succ + self.fail) as f64
        )
    }
}

impl Ic3 {
    pub fn statistic(&self) {
        self.obligations.statistic();
        self.frames.statistic();
        println!("{:#?}", self.share.statistic.lock().unwrap());
    }
}
