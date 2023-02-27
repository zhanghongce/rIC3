use crate::{pdr::basic::BasicShare, utils::generalize::generalize_by_ternary_simulation};
use logic_form::{Cube, Lit};
use sat_solver::{minisat::Solver, SatResult, SatSolver};
use std::collections::HashSet;

pub fn bad_state_partition(share: &BasicShare, frame: &Vec<Cube>) -> Vec<Cube> {
    let mut solver = Solver::new();
    solver.add_cnf(&share.transition_cnf);
    for c in frame {
        solver.add_clause(&!c.clone());
    }
    let mut cexs: Vec<Cube> = Vec::new();
    while let SatResult::Sat(model) = solver.solve(&[share.aig.bads[0].to_lit()]) {
        // optimize generalize priority for partition
        let mut cex =
            generalize_by_ternary_simulation(&share.aig, model, &[share.aig.bads[0]]).to_cube();
        let cex_clone = cex.clone();
        for c in cexs.iter() {
            let c_set: HashSet<&Lit> = HashSet::from_iter(c.iter());
            cex = cex.into_iter().filter(|lit| c_set.contains(lit)).collect();
        }
        cexs.push(cex_clone);
        solver.add_clause(&!cex);
    }
    cexs
}
