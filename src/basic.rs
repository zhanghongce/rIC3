use crate::command::Args;
use crate::frames::Lemma;
use crate::model::Model;
use aig::Aig;
use logic_form::Cube;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub struct BasicShare {
    pub aig: Aig,
    pub args: Args,
    pub model: Model,
    pub bad: Cube,
}

#[derive(PartialEq, Eq, Clone)]
pub struct ProofObligation {
    pub frame: usize,
    pub lemma: Lemma,
    pub depth: usize,
}

impl ProofObligation {
    pub fn new(frame: usize, lemma: Lemma, depth: usize) -> Self {
        Self {
            frame,
            lemma,
            depth,
        }
    }
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

#[derive(Default)]
pub struct ProofObligationQueue {
    obligations: BinaryHeap<ProofObligation>,
    num: Vec<usize>,
}

impl ProofObligationQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, po: ProofObligation) {
        if self.num.len() <= po.frame {
            self.num.resize(po.frame + 1, 0);
        }
        self.num[po.frame] += 1;
        self.obligations.push(po)
    }

    pub fn pop(&mut self) -> Option<ProofObligation> {
        let po = self.obligations.pop();
        if let Some(po) = &po {
            self.num[po.frame] -= 1;
        }
        po
    }

    pub fn is_empty(&self) -> bool {
        self.obligations.is_empty()
    }

    pub fn statistic(&self) {
        println!("{:?}", self.num);
    }
}
