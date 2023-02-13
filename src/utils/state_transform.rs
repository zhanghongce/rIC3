use aig::{Aig, AigCube, AigEdge};
use std::collections::HashMap;

pub fn aig_cube_next(aig: &Aig, cube: &AigCube) -> AigCube {
    let mut map = HashMap::new();
    for l in &aig.latchs {
        map.insert(l.input, l.next);
    }
    let mut ans = AigCube::new();
    for l in cube.iter() {
        let next = map[&l.node_id()];
        ans.push(next.not_if(l.compl()));
    }
    ans
}

pub fn aig_cube_previous(aig: &Aig, cube: &AigCube) -> AigCube {
    let mut map = HashMap::new();
    for l in &aig.latchs {
        map.insert(l.next.node_id(), AigEdge::new(l.input, l.next.compl()));
    }
    let mut ans = AigCube::new();
    for l in cube.iter() {
        let next = map[&l.node_id()];
        ans.push(next.not_if(l.compl()));
    }
    ans
}
