use aig::Aig;
use logic_form::{Cube, Lit};

pub struct Activity {
    activity: Vec<u32>,
}

impl Activity {
    pub fn new(aig: &Aig) -> Self {
        Self {
            activity: vec![0; aig.nodes.len()],
        }
    }

    pub fn pump_activity(&mut self, lit: &Lit) {
        self.activity[Into::<usize>::into(lit.var())] += 1;
    }

    pub fn sort_by_activity_ascending(&self, mut cube: Cube) -> Cube {
        cube.sort_by_key(|l| self.activity[Into::<usize>::into(l.var())]);
        cube
    }

    pub fn var_activity(&self, var: &Lit) -> u32 {
        self.activity[Into::<usize>::into(var.var())]
    }
}
