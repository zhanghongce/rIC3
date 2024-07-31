use crate::{transys::Transys, IC3};
use logic_form::{Cube, Lit, Var, VarMap};
use std::{collections::HashMap, ops::MulAssign};

pub struct Activity {
    activity: VarMap<f64>,
    max_act: f64,
    act_inc: f64,
}

impl Activity {
    pub fn new(ts: &Transys) -> Self {
        let mut activity = VarMap::new();
        activity.reserve(ts.max_latch);
        Self {
            activity,
            max_act: 0.0,
            act_inc: 1.0,
        }
    }

    #[inline]
    pub fn reserve(&mut self, var: Var) {
        self.activity.reserve(var);
    }

    #[inline]
    fn bump(&mut self, var: Var) {
        self.activity[var] += self.act_inc;
        self.max_act = self.max_act.max(self.activity[var]);
        if self.activity[var] > 1e100 {
            self.activity.iter_mut().for_each(|a| a.mul_assign(1e-100));
            self.act_inc *= 1e-100;
            self.max_act *= 1e-100;
        }
    }

    #[inline]
    pub fn set_max_act(&mut self, var: Var) {
        self.activity[var] = self.max_act;
    }

    #[inline]
    pub fn decay(&mut self) {
        self.act_inc *= 1.0 / 0.9
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
