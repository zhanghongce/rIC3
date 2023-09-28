use super::statistic::Statistic;
use crate::command::Args;
use crate::utils::state_transform::StateTransform;
use crate::Ic3;
use aig::Aig;
use logic_form::Cnf;
use logic_form::Cube;
use logic_form::Var;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
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

#[derive(PartialEq, Eq, Clone)]
pub struct ProofObligation {
    pub frame: usize,
    pub cube: Cube,
    pub depth: usize,
}

impl PartialOrd for ProofObligation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProofObligation {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.frame.cmp(&self.frame) {
            Ordering::Equal => other.depth.cmp(&self.depth),
            ord => ord,
        }
    }
}

pub struct ProofObligationQueue {
    obligations: BinaryHeap<ProofObligation>,
    num: Vec<usize>,
}

impl ProofObligationQueue {
    pub fn new() -> Self {
        Self {
            obligations: BinaryHeap::new(),
            num: Vec::new(),
        }
    }

    pub fn add(&mut self, po: ProofObligation) {
        if self.num.len() <= po.frame {
            self.num.resize(po.frame + 1, 0);
        }
        self.num[po.frame] += 1;
        self.obligations.push(po)
    }

    pub fn get(&mut self) -> Option<ProofObligation> {
        self.obligations.pop()
    }

    pub fn statistic(&self) {
        println!("{:?}", self.num);
    }
}

#[derive(Debug)]
pub enum Ic3Error {
    StopBlock,
}

impl Ic3 {
    pub fn check_stop_block(&self) -> Result<(), Ic3Error> {
        (!self.stop_block).then_some(()).ok_or(Ic3Error::StopBlock)
    }
}
