// use crate::utils::{self, aig_with_bdd::dnf_to_bdd};
// use aig::{Aig, AigCube, AigDnf, AigEdge};
// use logic_form::{Clause, Cnf, Lit, Var};
// use sat_solver::SatSolver;
// use std::{collections::HashMap, ops::DerefMut};

// struct VarAlloc {
//     var_base: usize,
// }

// impl VarAlloc {
//     fn new(var_base: usize) -> Self {
//         Self { var_base }
//     }

//     fn new_var(&mut self) -> Var {
//         self.var_base += 1;
//         (self.var_base - 1).into()
//     }
// }

// struct SortNetwork {
//     var_alloc: VarAlloc,
//     cnf: Cnf,
// }

// impl SortNetwork {
//     fn new(var_alloc: VarAlloc) -> Self {
//         Self {
//             var_alloc,
//             cnf: Cnf::new(),
//         }
//     }

//     fn sort_gate(&mut self, x: Lit, y: Lit) -> (Lit, Lit) {
//         let new_x: Lit = self.var_alloc.new_var().into();
//         let new_y: Lit = self.var_alloc.new_var().into();
//         self.cnf.push(Clause::from([!new_x, x, y]));
//         self.cnf.push(Clause::from([new_x, !x]));
//         self.cnf.push(Clause::from([new_x, !y]));
//         self.cnf.push(Clause::from([new_y, !x, !y]));
//         self.cnf.push(Clause::from([!new_y, x]));
//         self.cnf.push(Clause::from([!new_y, y]));
//         (new_x, new_y)
//     }

//     fn finish(mut self, vars: &[Lit]) -> (Vec<Lit>, Cnf) {
//         let mut vars = vars.to_vec();
//         for round in 0..vars.len() {
//             if round % 2 > 0 {
//                 for i in (0..vars.len() - 1).step_by(2) {
//                     let (out0, out1) = self.sort_gate(vars[i], vars[i + 1]);
//                     vars[i] = out0;
//                     vars[i + 1] = out1;
//                 }
//             } else {
//                 for i in (1..vars.len() - 1).step_by(2) {
//                     let (out0, out1) = self.sort_gate(vars[i], vars[i + 1]);
//                     vars[i] = out0;
//                     vars[i + 1] = out1;
//                 }
//             }
//         }
//         (vars, self.cnf)
//     }
// }

// // pub fn dnf_to_bdd(&self, dnf: &DNF) -> Bdd {
// //     let mut latch_to_bdd_id = HashMap::new();
// //     for i in 0..self.latchs.len() {
// //         latch_to_bdd_id.insert(self.latchs[i].input, i);
// //     }
// //     let mut bad_bdd = Vec::new();
// //     let vars_set = BddVariableSet::new_anonymous(self.latchs.len() as _);
// //     let vars = vars_set.variables();
// //     for c in dnf.iter() {
// //         let mut cube = Vec::new();
// //         for l in c.iter() {
// //             cube.push((vars[latch_to_bdd_id[&l.node_id()]], !l.compl()));
// //         }
// //         bad_bdd.push(BddPartialValuation::from_values(&cube));
// //     }
// //     vars_set.mk_dnf(&bad_bdd)
// // }

// // pub fn preimage(aig: Aig, logic: &[AigEdge], reached_states: &[Vec<AigEdge>]) -> AigDnf {
// //     let mut block_reached_cnf = Cnf::new();
// //     for cube in reached_states {
// //         let mut clause = Clause::new();
// //         for lit in cube {
// //             clause.push(!lit.to_lit());
// //         }
// //         block_reached_cnf.push(clause);
// //     }
// //     let mut cnf = aig.get_optimized_cnf(logic);
// //     let mut var_alloc = VarAlloc::new(aig.num_nodes());
// //     let mut origin_to_dual = HashMap::new();
// //     let mut dual_to_origin = HashMap::new();
// //     let mut duals = Vec::new();
// //     let mut dual_constraint = Cnf::new();
// //     for latch in &aig.latchs {
// //         let edge = AigEdge::new(latch.input, false);
// //         let pos = edge.to_lit();
// //         let neg = AigEdge::new(var_alloc.new_var().into(), false).to_lit();
// //         origin_to_dual.insert(edge.to_lit(), pos);
// //         origin_to_dual.insert((!edge).to_lit(), neg);
// //         dual_to_origin.insert(pos.var(), edge);
// //         dual_to_origin.insert(neg.var(), !edge);
// //         dual_constraint.push(Clause::from([!pos, !neg]));
// //         duals.push(pos);
// //         duals.push(neg);
// //     }
// //     for clause in cnf.deref_mut() {
// //         for lit in clause.deref_mut() {
// //             if let Some(dual) = origin_to_dual.get(lit) {
// //                 *lit = *dual
// //             }
// //         }
// //     }
// //     let mut ans = AigDnf::new();
// //     let mut solver = sat_solver::minisat::Solver::new();
// //     solver.add_cnf(&cnf);
// //     solver.add_cnf(&block_reached_cnf);
// //     solver.add_cnf(&dual_constraint);
// //     let sort_network = SortNetwork::new(var_alloc);
// //     let (mut sort_outputs, sort_cnf) = sort_network.finish(&duals);
// //     solver.add_cnf(&sort_cnf);
// //     for out in &mut sort_outputs {
// //         *out = !*out;
// //     }
// //     assert!(sort_outputs.len() / 2 == aig.latchs.len());
// //     for limit in 0..aig.latchs.len() {
// //         sort_outputs[limit] = !sort_outputs[limit];
// //         while let Some(cex) = solver.solve(&sort_outputs) {
// //             let mut blocking_clause = Clause::new();
// //             let mut cube = AigCube::new();
// //             for lit in cex {
// //                 if let Some(origin) = dual_to_origin.get(&lit.var()) {
// //                     if !lit.compl() {
// //                         blocking_clause.push(!*lit);
// //                         cube.push(*origin);
// //                     }
// //                 }
// //             }
// //             solver.add_clause(&blocking_clause);
// //             ans.push(cube);
// //             // dbg!(ans.len());
// //         }
// //         dbg!(limit);
// //         dbg!(ans.len());
// //     }
// //     // dbg!(&ans.len());
// //     // dbg!(&ans);
// //     ans
// //     // get_bdd(&aig.latchs, &ans)
// // }

// // pub fn solve(aig: Aig) -> bool {
// //     let logic = aig.bads[0];
// //     let mut frontier = preimage(aig.clone(), &[logic], &[]);
// //     panic!();
// //     let mut bad_states_dnf = frontier.clone();
// //     let mut bad_states_bdd = get_bdd(&aig.latchs, &bad_states_dnf);
// //     let mut latch_map = HashMap::new();
// //     for l in &aig.latchs {
// //         latch_map.insert(l.input, l.next);
// //     }
// //     let mut deep = 0;
// //     loop {
// //         dbg!(deep);
// //         deep += 1;
// //         // let mut tmp_aig = aig.clone();
// //         dbg!(frontier.len());
// //         // let avai_logic = utils::bdd2aig::sat_up_bdd_logic_next(&mut tmp_aig, &image);
// //         // let block_logic = utils::bdd2aig::sat_up_bdd_logic_input(&mut tmp_aig, &bad_bdd);
// //         // let logic = tmp_aig.new_and_node(avai_logic, !block_logic);
// //         let mut new_frontier = Vec::new();
// //         for cube in &frontier {
// //             let next_cube: Vec<AigEdge> = cube
// //                 .iter()
// //                 .map(|e| {
// //                     let next = *latch_map.get(&e.node_id()).unwrap();
// //                     if e.compl() {
// //                         !next
// //                     } else {
// //                         next
// //                     }
// //                 })
// //                 .collect();
// //             new_frontier.extend(preimage(aig.clone(), &next_cube, &bad_states_dnf));
// //             bad_states_dnf.extend_from_slice(&new_frontier);
// //         }
// //         dbg!(new_frontier.len());
// //         bad_states_bdd = get_bdd(&aig.latchs, &bad_states_dnf);
// //         dbg!(bad_states_bdd.size());
// //         bad_states_dnf = bdd_to_dnf(aig.clone(), &bad_states_bdd);
// //         let new_frontier_bdd = get_bdd(&aig.latchs, &new_frontier);
// //         frontier = bdd_to_dnf(aig.clone(), &new_frontier_bdd);
// //     }
// //     todo!()
// // }

// // pub fn bdd_to_dnf(aig: Aig, bdd: &Bdd) -> Vec<Vec<AigEdge>> {
// //     let dnf: Vec<Vec<AigEdge>> = bdd
// //         .sat_clauses()
// //         .map(|v| {
// //             let cube: Vec<AigEdge> = v
// //                 .to_values()
// //                 .iter()
// //                 .map(|(var, val)| AigEdge::new(aig.latchs[Into::<usize>::into(*var)].input, !val))
// //                 .collect();
// //             cube.into()
// //         })
// //         .collect();
// //     dnf.into()
// // }

// pub fn solve(aig: Aig) -> bool {
//     let logic = aig.bads[0];
//     let frontier = preimage(aig.clone(), &[logic], &[]);
//     dbg!(frontier.len());
//     let mut frontier_bdd = dnf_to_bdd(&aig, &frontier);
//     dbg!(frontier_bdd.size());
//     // let mut bad_states_dnf = frontier.clone();
//     let mut bad_states_bdd = frontier_bdd.clone();
//     let mut deep = 0;
//     loop {
//         dbg!(deep);
//         dbg!(bad_states_bdd.size());
//         deep += 1;
//         let mut tmp_aig = aig.clone();
//         let avai_logic = utils::aig_with_bdd::sat_up_bdd_logic_next(&mut tmp_aig, &frontier_bdd);
//         let block_logic =
//             utils::aig_with_bdd::sat_up_bdd_logic_input(&mut tmp_aig, &bad_states_bdd);
//         let logic = tmp_aig.new_and_node(avai_logic, !block_logic);
//         let mut new_frontier = AigDnf::new();
//         new_frontier = new_frontier + preimage(tmp_aig, &[logic], &[]);
//         // bad_states_dnf.extend_from_slice(&new_frontier);
//         dbg!(new_frontier.len());
//         if new_frontier.is_empty() {
//             return true;
//         }
//         // bad_states_bdd = get_bdd(&aig.latchs, &bad_states_dnf);
//         // dbg!(bad_states_bdd.size());
//         // bad_states_dnf = bdd_to_dnf(aig.clone(), &bad_states_bdd);
//         frontier_bdd = dnf_to_bdd(&aig, &new_frontier);
//         bad_states_bdd = bad_states_bdd.or(&frontier_bdd);
//         // frontier = bdd_to_dnf(aig.clone(), &new_frontier_bdd);
//     }
// }
