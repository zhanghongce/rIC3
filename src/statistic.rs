use crate::IC3;
use giputils::statistic::{Average, Case, RunningTime, SuccessRate};
use std::{fmt::Debug, time::Duration};

#[allow(unused)]
#[derive(Debug, Default)]
pub struct Statistic {
    case: Case,
    time: RunningTime,

    pub num_mic: usize,
    pub avg_mic_cube_len: Average,
    pub avg_po_cube_len: Average,
    pub mic_drop: SuccessRate,
    pub num_down: usize,

    pub overall_mic_time: Duration,
    pub overall_block_time: Duration,
    pub overall_propagate_time: Duration,

    pub num_pred: usize,
    pub can_pred: SuccessRate,
    pub pred_time_a: Duration,
    pub pred_time_b: Duration,
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

impl IC3 {
    pub fn statistic(&self) {
        self.obligations.statistic();
        for f in self.frame.iter() {
            print!("{} ", f.len());
        }
        self.gipsat.statistic();
        println!("{:#?}", self.statistic);
    }
}
