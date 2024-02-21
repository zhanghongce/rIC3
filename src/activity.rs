use logic_form::{Cube, Var, VarMap};

pub struct Activity {
    activity: VarMap<f64>,
    act_inc: f64,
}

impl Activity {
    pub fn new(var: &[Var]) -> Self {
        let mut activity = VarMap::new();
        for v in var.iter() {
            activity.reserve(*v);
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
            *act *= 0.99;
        }
    }

    pub fn bump_cube_activity(&mut self, cube: &Cube) {
        self.decay();
        cube.iter().for_each(|l| self.bump(l.var()));
    }

    pub fn sort_by_activity(&self, cube: &mut Cube, ascending: bool) {
        if ascending {
            cube.sort_by(|a, b| self.activity[*a].partial_cmp(&self.activity[*b]).unwrap());
        } else {
            cube.sort_by(|a, b| self.activity[*b].partial_cmp(&self.activity[*a]).unwrap());
        }
    }

    #[allow(unused)]
    pub fn cube_average_activity(&self, cube: &Cube) -> f64 {
        let sum: f64 = cube.iter().map(|l| self.activity[*l]).sum();
        sum / cube.len() as f64
    }
}
