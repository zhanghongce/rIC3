use crate::Ic3;
use giputils::statistic::{Average, AverageDuration, Case, RunningTime, SuccessRate};
use std::{fmt::Debug, time::Duration};

#[allow(unused)]
#[derive(Debug, Default)]
pub struct Statistic {
    case: Case,
    time: RunningTime,

    pub avg_sat_call_time: AverageDuration,
    pub num_sat_inductive: usize,
    pub sat_inductive_time: Duration,
    pub num_solver_restart: usize,

    pub num_mic: usize,
    pub avg_mic_cube_len: Average,
    pub avg_po_cube_len: Average,
    pub mic_drop: SuccessRate,
    pub num_down: usize,

    pub minimal_predecessor_time: Duration,

    pub overall_mic_time: Duration,
    pub overall_block_time: Duration,
    pub overall_propagate_time: Duration,
}

impl Statistic {
    pub fn new(mut case: &str) -> Self {
        if let Some((_, c)) = case.rsplit_once('/') {
            case = c;
        }
        Self {
            case: Case::new(case),
            ..Default::default()
        }
    }
}

impl Ic3 {
    pub fn statistic(&self) {
        self.obligations.statistic();
        self.gipsat.statistic();
        println!("{:#?}", self.statistic);
    }
}
