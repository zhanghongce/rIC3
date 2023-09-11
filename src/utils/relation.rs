use logic_form::{Cube, Var};
use std::collections::HashMap;

pub fn cube_subsume(x: &Cube, y: &Cube) -> bool {
    if x.len() > y.len() {
        return false;
    }
    let mut j = 0;
    for i in 0..x.len() {
        while j < y.len() && x[i].var() > y[j].var() {
            j += 1;
        }
        if j == y.len() || x[i] != y[j] {
            return false;
        }
    }
    true
}

pub fn cube_subsume_init(init: &HashMap<Var, bool>, x: &Cube) -> bool {
    for i in 0..x.len() {
        if let Some(init) = init.get(&x[i].var()) {
            if *init != x[i].polarity() {
                return false;
            }
        }
    }
    true
}
