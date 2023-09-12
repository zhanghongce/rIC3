use super::statistic::Statistic;
use crate::command::Args;
use crate::utils::state_transform::StateTransform;
use aig::Aig;
use logic_form::Cnf;
use logic_form::Cube;
use logic_form::Var;
use std::cmp::Ordering;
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

pub struct ProofObligation {
    pub frame: usize,
    pub cube: Cube,
}

impl ProofObligation {
    pub fn new(frame: usize, cube: Cube) -> Self {
        Self { frame, cube }
    }
}

impl PartialEq for ProofObligation {
    fn eq(&self, other: &Self) -> bool {
        self.frame == other.frame
    }
}

impl PartialOrd for ProofObligation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ProofObligation {
    fn assert_receiver_is_total_eq(&self) {}
}

impl Ord for ProofObligation {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.frame.cmp(&self.frame) {
            Ordering::Equal => other.cube.len().cmp(&self.cube.len()),
            ord => ord,
        }
    }
}
