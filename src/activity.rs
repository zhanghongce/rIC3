use logic_form::{Cube, Lit, Var};
use std::collections::HashMap;

pub struct Activity {
    activity: HashMap<Var, f64>,
}

impl Activity {
    pub fn new() -> Self {
        Self {
            activity: HashMap::new(),
        }
    }

    fn decay(&mut self) {
        for (_, act) in self.activity.iter_mut() {
            *act = *act * 0.99
        }
    }

    fn var_activity(&self, lit: Lit) -> f64 {
        match self.activity.get(&lit.var()) {
            Some(a) => *a,
            None => 0.0,
        }
    }

    fn pump_lit_activity(&mut self, lit: &Lit) {
        match self.activity.get_mut(&lit.var()) {
            Some(a) => *a += 1.0,
            None => {
                self.activity.insert(lit.var(), 1.0);
            }
        }
    }

    pub fn pump_cube_activity(&mut self, cube: &Cube) {
        self.decay();
        cube.iter().for_each(|l| self.pump_lit_activity(l));
    }

    pub fn sort_by_activity(&self, cube: &mut Cube, ascending: bool) {
        if ascending {
            cube.sort_by(|a, b| {
                self.var_activity(*a)
                    .partial_cmp(&self.var_activity(*b))
                    .unwrap()
            });
        } else {
            cube.sort_by(|a, b| {
                self.var_activity(*b)
                    .partial_cmp(&self.var_activity(*a))
                    .unwrap()
            });
        }
    }

    pub fn cube_average_activity(&self, cube: &Cube) -> f64 {
        let sum: f64 = cube.iter().map(|l| self.var_activity(*l)).sum();
        sum / cube.len() as f64
    }
}

#[derive(Debug)]
pub struct TriActivity {
    pub activity: HashMap<(Lit, Lit), f64>,
}

impl TriActivity {
    pub fn new() -> Self {
        Self {
            activity: HashMap::new(),
        }
    }

    pub fn lit_activity(&self, mut x: Lit, mut y: Lit) -> f64 {
        assert!(x.var() != y.var());
        if x.var() > y.var() {
            (y, x) = (x, y);
        }
        match self.activity.get(&(x, y)) {
            Some(a) => *a,
            None => 0.0,
        }
    }

    pub fn pump_activity(&mut self, mut x: Lit, mut y: Lit) {
        assert!(x.var() != y.var());
        if x.var() > y.var() {
            (y, x) = (x, y);
        }
        match self.activity.get_mut(&(x, y)) {
            Some(a) => *a += 1.0,
            None => {
                self.activity.insert((x, y), 1.0);
            }
        }
    }

    pub fn depump_activity(&mut self, mut x: Lit, mut y: Lit) {
        assert!(x.var() != y.var());
        if x.var() > y.var() {
            (y, x) = (x, y);
        }
        match self.activity.get_mut(&(x, y)) {
            Some(a) => *a -= 1.0,
            None => {
                self.activity.insert((x, y), -1.0);
            }
        }
    }

    pub fn get_max_couple(&self, cube: &[Lit]) -> (Lit, Lit) {
        let mut ans = (self.lit_activity(cube[0], cube[1]), (cube[0], cube[1]));
        for i in 0..cube.len() {
            for j in i + 1..cube.len() {
                if ans.0 < self.lit_activity(cube[i], cube[j]) {
                    ans = (self.lit_activity(cube[i], cube[j]), (cube[i], cube[j]));
                }
            }
        }
        ans.1
    }

    pub fn get_max(&self, lit: Lit, candidate: &[Lit]) -> usize {
        assert!(!candidate.is_empty());
        let mut ans = (self.lit_activity(lit, candidate[0]), 0);
        for i in 1..candidate.len() {
            if ans.0 < self.lit_activity(lit, candidate[i]) {
                ans = (self.lit_activity(lit, candidate[i]), i);
            }
        }
        ans.1
    }
}
