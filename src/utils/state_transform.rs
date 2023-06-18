use aig::Aig;
use logic_form::{Cube, Lit};
use std::collections::HashMap;

pub struct StateTransform {
    next_map: HashMap<Lit, Lit>,
    previous_map: HashMap<Lit, Lit>,
}

impl StateTransform {
    pub fn new(aig: &Aig) -> Self {
        let mut next_map = HashMap::new();
        let mut previous_map = HashMap::new();
        for l in &aig.latchs {
            let origin = Lit::new(l.input.into(), false);
            let next = l.next.to_lit();
            next_map.insert(origin, next);
            next_map.insert(!origin, !next);
            previous_map.insert(next, origin);
            previous_map.insert(!next, !origin);
        }
        Self {
            next_map,
            previous_map,
        }
    }

    pub fn lit_previous(&self, lit: Lit) -> Lit {
        self.previous_map[&lit]
    }

    pub fn cube_next(&self, cube: &Cube) -> Cube {
        cube.iter().map(|l| self.next_map[l]).collect()
    }

    pub fn previous<'a>(
        &'a self,
        iter: impl Iterator<Item = Lit> + 'a,
    ) -> impl Iterator<Item = Lit> + 'a {
        iter.map(|l| self.previous_map[&l])
    }
}
