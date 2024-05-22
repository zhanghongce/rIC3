use crate::IC3;
use logic_form::{Cube, Lit, Var, VarMap};
use std::collections::HashMap;

pub struct Activity {
    activity: VarMap<f64>,
    act_inc: f64,
}

impl Activity {
    pub fn new(var: &[Var]) -> Self {
        let mut activity = VarMap::new();
        for i in 0..var.len() {
            activity.reserve(var[i]);
        }
        Self {
            activity,
            act_inc: 1.0,
        }
    }

    #[inline]
    fn bump(&mut self, var: Var) {
        self.activity[var] += self.act_inc;
    }

    #[inline]
    pub fn decay(&mut self) {
        for act in self.activity.iter_mut() {
            *act *= 0.9;
        }
    }

    pub fn bump_cube_activity(&mut self, cube: &Cube) {
        self.decay();
        cube.iter().for_each(|l| self.bump(l.var()));
    }

    pub fn sort_by_activity(&self, cube: &mut Cube, ascending: bool) {
        let ascending_func =
            |a: &Lit, b: &Lit| self.activity[*a].partial_cmp(&self.activity[*b]).unwrap();
        if ascending {
            cube.sort_by(ascending_func);
        } else {
            cube.sort_by(|a, b| ascending_func(b, a));
        }
    }

    #[allow(unused)]
    pub fn cube_average_activity(&self, cube: &Cube) -> f64 {
        let sum: f64 = cube.iter().map(|l| self.activity[*l]).sum();
        sum / cube.len() as f64
    }
}

impl IC3 {
    pub fn sort_by_group_activity(&self, cube: &mut Cube, ascending: bool) {
        let mut group = HashMap::new();
        for i in 0..cube.len() {
            let g = self.ts.latch_group[cube[i]];
            if group.contains_key(&g) {
                continue;
            }
            let mut num = 0;
            let mut sum = 0.0;
            for j in i..cube.len() {
                if self.ts.latch_group[cube[j].var()] == g {
                    num += 1;
                    sum += self.activity.activity[cube[j]];
                }
            }
            group.insert(g, sum / num as f64);
        }
        let ascending_func = |a: &Lit, b: &Lit| {
            if self.ts.latch_group[*a] == self.ts.latch_group[*b] {
                self.activity.activity[*a]
                    .partial_cmp(&self.activity.activity[*b])
                    .unwrap()
            } else {
                group[&self.ts.latch_group[*a]]
                    .partial_cmp(&group[&self.ts.latch_group[*b]])
                    .unwrap()
            }
        };
        if ascending {
            cube.sort_by(ascending_func);
        } else {
            cube.sort_by(|a, b| ascending_func(b, a));
        }
    }
}
