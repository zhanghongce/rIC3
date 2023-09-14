use logic_form::{Cube, Var};
use std::collections::HashMap;

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
