use logic_form::{Cube, Var, VarMap};

pub struct Activity {
    activity: Vec<f64>,
    idx_map: VarMap<usize>,
    act_inc: f64,
}

impl Activity {
    pub fn new(var: &[Var]) -> Self {
        let mut idx_map = VarMap::new();
        let mut activity = Vec::new();
        for i in 0..var.len() {
            idx_map.reserve(var[i]);
            idx_map[var[i]] = i;
            activity.push(0.0);
        }
        Self {
            activity,
            idx_map,
            act_inc: 1.0,
        }
    }

    #[inline]
    fn bump(&mut self, var: Var) {
        self.activity[self.idx_map[var]] += self.act_inc;
    }

    #[inline]
    pub fn decay(&mut self) {
        for act in self.activity.iter_mut() {
            *act *= 0.99;
        }
    }

    pub fn bump_cube_activity(&mut self, cube: &Cube) {
        self.decay();
        cube.iter().for_each(|l| self.bump(l.var()));
    }

    pub fn sort_by_activity(&self, cube: &mut Cube, ascending: bool) {
        if ascending {
            cube.sort_by(|a, b| {
                self.activity[self.idx_map[*a]]
                    .partial_cmp(&self.activity[self.idx_map[*b]])
                    .unwrap()
            });
        } else {
            cube.sort_by(|a, b| {
                self.activity[self.idx_map[*b]]
                    .partial_cmp(&self.activity[self.idx_map[*a]])
                    .unwrap()
            });
        }
    }

    #[allow(unused)]
    pub fn cube_average_activity(&self, cube: &Cube) -> f64 {
        let sum: f64 = cube.iter().map(|l| self.activity[self.idx_map[*l]]).sum();
        sum / cube.len() as f64
    }
}
