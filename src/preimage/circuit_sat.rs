use crate::utils::{
    aig_with_bdd::{dnf_to_bdd, sat_up_bdd_logic_next},
    generalize::generalize_by_ternary_simulation,
};
use aig::{Aig, AigDnf, AigEdge, AigLatch};
use biodivine_lib_bdd::{Bdd, BddPartialValuation, BddVariableSet};
use std::collections::HashMap;

fn logic_dnf(aig: &Aig, logic: AigEdge) -> AigDnf {
    let mut solver = sat_solver::abc_circuit::Solver::new(aig);
    let mut dnf = AigDnf::new();
    while let Some(cex) = solver.solve(&[logic]) {
        let cube = generalize_by_ternary_simulation(aig, cex, &[logic]);
        solver.add_clause(&!cube.clone());
        dnf.push(cube);
    }
    dnf
}

pub fn aig_dnf_to_bdd(latchs: &[AigLatch], dnf: &AigDnf) -> Bdd {
    let mut latch_to_bdd_id = HashMap::new();
    for (i, l) in latchs.iter().enumerate() {
        latch_to_bdd_id.insert(l.input, i);
    }
    let vars_set = BddVariableSet::new_anonymous(latchs.len() as _);
    let vars = vars_set.variables();
    let mut bdd = Vec::new();
    for clause in dnf.iter() {
        let mut cube = Vec::new();
        for l in clause.iter() {
            cube.push((vars[latch_to_bdd_id[&l.node_id()]], !l.compl()));
        }
        bdd.push(BddPartialValuation::from_values(&cube));
    }
    vars_set.mk_dnf(&bdd)
}

// pub fn solve(aig: Aig) -> bool {
//     let mut latch_transition = HashMap::new();
//     let mut init = Vec::new();
//     for l in aig.latchs.iter() {
//         init.push(AigEdge::new(l.input, !l.init));
//         assert!(latch_transition.insert(l.input, l.next).is_none());
//     }
//     let logic = aig.bads[0];
//     let mut bad_dnf = logic_dnf(&aig, logic);
//     let mut bad_bdd = aig_dnf_to_bdd(&aig.latchs, &bad_dnf);
//     let mut frontier = bad_dnf.clone();
//     let mut deep = 0;
//     loop {
//         deep += 1;
//         dbg!(deep);
//         dbg!(bad_dnf.len());
//         dbg!(frontier.len());
//         let mut solver = sat_solver::abc_circuit::Solver::new(&aig);
//         let mut new_frontier = Vec::new();
//         let good_cnf: Vec<Vec<AigEdge>> = bad_dnf
//             .iter()
//             .map(|cube| cube.iter().map(|lit| !*lit).collect())
//             .collect();
//         solver.add_cnf(&good_cnf);
//         if solver.solve(&init).is_none() {
//             return false;
//         }
//         for cube in frontier.iter() {
//             let mut assumptions = cube.clone();
//             for lit in assumptions.iter_mut() {
//                 let next = latch_transition.get(&lit.node_id()).unwrap();
//                 lit.set_nodeid(next.node_id());
//                 if next.compl() {
//                     *lit = !*lit;
//                 }
//             }
//             while let Some(cex) = solver.solve(&assumptions) {
//                 let cube = generalize_by_ternary_simulation(&aig, cex, &assumptions);
//                 let clause: Vec<AigEdge> = cube.iter().map(|lit| !*lit).collect();
//                 solver.add_clause(&clause);
//                 new_frontier.push(cube);
//             }
//         }
//         if new_frontier.is_empty() {
//             dbg!(deep);
//             return true;
//         } else {
//             dbg!(new_frontier.len());
//             bad_dnf.extend_from_slice(&new_frontier);
//             dbg!(bad_dnf.len());

//             let bad_new_frontier = dnf_to_bdd(&aig, &new_frontier);
//             bad_bdd = bad_bdd.or(&bad_new_frontier);
//             dbg!(bad_bdd.size());
//             dbg!(new_frontier.len());
//             let bad_dnf_new = bdd_to_dnf(&aig, &bad_bdd);
//             if bad_dnf_new.len() < bad_dnf.len() {
//                 bad_dnf = bad_dnf_new;
//             }
//             let frontier_new = bdd_to_dnf(&aig, &bad_new_frontier);
//             if frontier_new.len() < new_frontier.len() {
//                 frontier = frontier_new;
//             } else {
//                 frontier = new_frontier;
//             }
//             dbg!(frontier.len());

//             // frontier = new_frontier;
//         }
//     }
// }

pub fn solve(aig: Aig) -> bool {
    let mut latch_transition = HashMap::new();
    let mut init = Vec::new();
    for l in aig.latchs.iter() {
        init.push(AigEdge::new(l.input, !l.init));
        assert!(latch_transition.insert(l.input, l.next).is_none());
    }
    let logic = aig.bads[0];
    let mut bad_dnf = logic_dnf(&aig, logic);
    let mut frontier_bdd = aig_dnf_to_bdd(&aig.latchs, &bad_dnf);
    let mut deep = 0;
    loop {
        deep += 1;
        dbg!(deep);
        dbg!(bad_dnf.len());
        let mut aig_tmp = aig.clone();
        let logic = sat_up_bdd_logic_next(&mut aig_tmp, &frontier_bdd);
        let mut solver = sat_solver::abc_circuit::Solver::new(&aig_tmp);
        solver.add_cnf(&!bad_dnf.clone());
        if solver.solve(&init).is_none() {
            return false;
        }
        let mut new_frontier = AigDnf::new();
        while let Some(cex) = solver.solve(&[logic]) {
            let cube = generalize_by_ternary_simulation(&aig_tmp, cex, &[logic]);
            solver.add_clause(&!cube.clone());
            new_frontier.push(cube);
        }
        if new_frontier.is_empty() {
            dbg!(deep);
            return true;
        } else {
            dbg!(new_frontier.len());
            bad_dnf.extend_from_slice(&new_frontier);
            dbg!(bad_dnf.len());
            frontier_bdd = dnf_to_bdd(&aig, &new_frontier);
            // bad_bdd = bad_bdd.or(&frontier_bdd);
            // let bad_dnf_new = bdd_to_dnf(&aig, &bad_bdd);
            // if bad_dnf_new.len() < bad_dnf.len() {
            //     bad_dnf = bad_dnf_new;
            // }
        }
    }
}
