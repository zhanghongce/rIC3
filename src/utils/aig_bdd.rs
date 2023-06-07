use aig::{Aig, AigEdge, AigNodeId};
use cudd::{Bdd, Cudd};
use std::collections::HashMap;
// use biodivine_lib_bdd::{Bdd, BddPartialValuation, BddPointer, BddVariableSet};

// fn sat_up_bdd_logic_recursive(
//     aig: &mut Aig,
//     bdd: &Bdd,
//     p: BddPointer,
//     var_map: &HashMap<usize, AigEdge>,
//     cache: &mut HashMap<BddPointer, AigEdge>,
// ) -> AigEdge {
//     if let Some(ret) = cache.get(&p) {
//         return *ret;
//     }
//     if p.is_terminal() {
//         if p.is_one() {
//             assert!(cache.insert(p, AigEdge::constant_edge(true)).is_none());
//             return AigEdge::constant_edge(true);
//         } else {
//             assert!(cache.insert(p, AigEdge::constant_edge(false)).is_none());
//             return AigEdge::constant_edge(false);
//         }
//     }
//     let high_pointer = bdd.high_link_of(p);
//     let low_pointer = bdd.low_link_of(p);
//     let high_logic = sat_up_bdd_logic_recursive(aig, bdd, high_pointer, var_map, cache);
//     let low_logic = sat_up_bdd_logic_recursive(aig, bdd, low_pointer, var_map, cache);
//     let var = var_map[&Into::<usize>::into(bdd.var_of(p))];
//     let low_sub_node = !aig.new_and_node(!low_logic, !var);
//     let high_sub_node = !aig.new_and_node(!high_logic, var);
//     let ret = aig.new_and_node(low_sub_node, high_sub_node);
//     assert!(cache.insert(p, ret).is_none());
//     ret
// }

// pub fn sat_up_bdd_logic_input(aig: &mut Aig, bdd: &Bdd) -> AigEdge {
//     let mut var_map = HashMap::new();
//     for i in 0..aig.latchs.len() {
//         var_map.insert(i, aig.latchs[i].input.into());
//     }
//     sat_up_bdd_logic_recursive(aig, bdd, bdd.root_pointer(), &var_map, &mut HashMap::new())
// }

// pub fn sat_up_bdd_logic_next(aig: &mut Aig, bdd: &Bdd) -> AigEdge {
//     let mut var_map = HashMap::new();
//     for i in 0..aig.latchs.len() {
//         var_map.insert(i, aig.latchs[i].next);
//     }
//     sat_up_bdd_logic_recursive(aig, bdd, bdd.root_pointer(), &var_map, &mut HashMap::new())
// }

// pub fn dnf_to_bdd(aig: &Aig, dnf: &AigDnf) -> Bdd {
//     let mut latch_to_bdd_id = HashMap::new();
//     for (i, l) in aig.latchs.iter().enumerate() {
//         latch_to_bdd_id.insert(l.input, i);
//     }
//     let vars_set = BddVariableSet::new_anonymous(aig.latchs.len() as _);
//     let vars = vars_set.variables();
//     let mut bdd = Vec::new();
//     for clause in dnf.iter() {
//         let mut cube = Vec::new();
//         for l in clause.iter() {
//             cube.push((vars[latch_to_bdd_id[&l.node_id()]], !l.compl()));
//         }
//         bdd.push(BddPartialValuation::from_values(&cube));
//     }
//     vars_set.mk_dnf(&bdd)
// }

// pub fn bdd_to_dnf(aig: &Aig, bdd: &Bdd) -> Vec<Vec<AigEdge>> {
//     let dnf: Vec<Vec<AigEdge>> = bdd
//         .sat_clauses()
//         .map(|v| {
//             let cube: Vec<AigEdge> = v
//                 .to_values()
//                 .iter()
//                 .map(|(var, val)| AigEdge::new(aig.latchs[Into::<usize>::into(*var)].input, !val))
//                 .collect();
//             cube
//         })
//         .collect();
//     dnf
// }

fn aig_var_map(aig: &Aig) -> HashMap<AigNodeId, usize> {
    let mut res = HashMap::new();
    let mut var = 0;
    for input in aig.inputs.iter() {
        res.insert(*input, var);
        var += 2;
    }
    for latch in aig.latchs.iter() {
        res.insert(latch.input, var);
        var += 2;
    }
    res
}

pub fn aig_trans_logic_bdd_inner(
    aig: &Aig,
    cudd: &Cudd,
    edge: AigEdge,
    var_map: &HashMap<AigNodeId, usize>,
) -> Bdd {
    if edge == AigEdge::constant_edge(true) {
        return cudd.constant(true);
    }
    if edge == AigEdge::constant_edge(false) {
        return cudd.constant(false);
    }
    let node_id = edge.node_id();
    let mut res = if aig.nodes[node_id].is_and() {
        let fanin0 = aig.nodes[node_id].fanin0();
        let fanin1 = aig.nodes[node_id].fanin1();
        let fanin0 = aig_trans_logic_bdd(aig, cudd, fanin0);
        let fanin1 = aig_trans_logic_bdd(aig, cudd, fanin1);
        fanin0 & fanin1
    } else {
        cudd.ith_var(var_map[&node_id])
    };
    if edge.compl() {
        res = !res
    }
    res
}

pub fn aig_trans_logic_bdd(aig: &Aig, cudd: &Cudd, edge: AigEdge) -> Bdd {
    let var_map = aig_var_map(aig);
    aig_trans_logic_bdd_inner(aig, cudd, edge, &var_map)
}

pub fn aig_trans_bdd(aig: &Aig, cudd: &Cudd) -> Bdd {
    let var_map = aig_var_map(aig);
    let mut res = cudd.constant(true);
    for i in 0..aig.latchs.len() {
        dbg!(i);
        let bdd = aig_trans_logic_bdd_inner(aig, cudd, aig.latchs[i].next, &var_map);
        let next_id = cudd.ith_var(var_map[&aig.latchs[i].input] + 1);
        let tran = !(bdd ^ next_id);
        res &= tran;
    }
    res
}

pub fn aig_trans_init_bdd(aig: &Aig, cudd: &Cudd) -> Bdd {
    let mut res = cudd.constant(true);
    let var_map = aig_var_map(aig);
    for i in 0..aig.latchs.len() {
        let mut init = cudd.ith_var(var_map[&aig.latchs[i].input]);
        if !aig.latchs[i].init {
            init = !init
        }
        res &= init;
    }
    res
}
