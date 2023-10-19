// use aig::Aig;
// use logic_form::{Cube, Lit};
// use std::collections::HashMap;

// impl StateTransform {

//     pub fn cube_next(&self, cube: &Cube) -> Cube {
//         cube.iter().map(|l| self.next_map[l]).collect()
//     }

//     pub fn previous<'a>(
//         &'a self,
//         iter: impl Iterator<Item = Lit> + 'a,
//     ) -> impl Iterator<Item = Lit> + 'a {
//         iter.map(|l| self.previous_map[&l])
//     }
// }
