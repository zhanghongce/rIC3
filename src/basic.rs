use super::statistic::Statistic;
use crate::command::Args;
use crate::utils::state_transform::StateTransform;
use aig::Aig;
use logic_form::Cnf;
use logic_form::Cube;
use logic_form::Var;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct BasicShare {
    pub aig: Aig,
    pub transition_cnf: Cnf,
    pub state_transform: StateTransform,
    pub args: Args,
    pub init: HashMap<Var, bool>,
    pub statistic: Mutex<Statistic>,
}

pub struct HeapFrameCube {
    pub frame: usize,
    pub cube: Cube,
}

impl HeapFrameCube {
    pub fn new(frame: usize, cube: Cube) -> Self {
        Self { frame, cube }
    }
}

impl PartialEq for HeapFrameCube {
    fn eq(&self, other: &Self) -> bool {
        self.frame == other.frame
    }
}

impl PartialOrd for HeapFrameCube {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match other.frame.partial_cmp(&self.frame) {
            Some(core::cmp::Ordering::Equal) => other.cube.len().partial_cmp(&self.cube.len()),
            ord => ord,
        }
    }
}

impl Eq for HeapFrameCube {
    fn assert_receiver_is_total_eq(&self) {}
}

impl Ord for HeapFrameCube {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.frame.cmp(&self.frame)
    }
}
