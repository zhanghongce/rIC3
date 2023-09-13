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

pub struct ProofObligationQueue {
    obligations: Vec<Vec<Cube>>,
}

impl ProofObligationQueue {
    pub fn new() -> Self {
        Self {
            obligations: Vec::new(),
        }
    }

    pub fn add(&mut self, frame: usize, cube: Cube) {
        while self.obligations.len() <= frame {
            self.obligations.push(Vec::new());
        }
        self.obligations[frame].push(cube);
        self.obligations[frame].sort_by_key(|c| c.len());
    }

    pub fn get(&mut self) -> Option<(usize, Cube)> {
        for i in 0..self.obligations.len() {
            if !self.obligations[i].is_empty() {
                return Some((i, self.obligations[i].remove(0)));
            }
        }
        None
    }
}
