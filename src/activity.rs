use aig::Aig;
use logic_form::{Cube, Lit};

pub struct Activity {
    activity: Vec<f64>,
}

impl Activity {
    pub fn new(aig: &Aig) -> Self {
        Self {
            activity: vec![0_f64; aig.nodes.len()],
        }
    }

    pub fn var_activity(&self, lit: Lit) -> f64 {
        self.activity[Into::<usize>::into(lit.var())]
    }

    pub fn cube_average_activity(&self, cube: &Cube) -> f64 {
        let sum: f64 = cube.iter().map(|l| self.var_activity(*l)).sum();
        sum / cube.len() as f64
    }

    pub fn pump_lit_activity(&mut self, lit: &Lit) {
        self.activity[Into::<usize>::into(lit.var())] += 1.0;
    }

    pub fn pump_cube_activity(&mut self, cube: &Cube) {
        cube.iter().for_each(|l| self.pump_lit_activity(l))
    }

    pub fn sort_by_activity_ascending(&self, cube: &mut Cube) {
        cube.sort_by(|a, b| {
            self.var_activity(*a)
                .partial_cmp(&self.var_activity(*b))
                .unwrap()
        });
    }

    pub fn sort_by_activity_descending(&self, cube: &mut Cube) {
        cube.sort_by(|a, b| {
            self.var_activity(*b)
                .partial_cmp(&self.var_activity(*a))
                .unwrap()
        });
    }
}
