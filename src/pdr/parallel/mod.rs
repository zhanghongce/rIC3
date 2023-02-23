// mod solver_pool;

// use self::solver_pool::SolverPool;
// use super::{activity::Activity, share::PdrShare, solver::BlockResult, statistic::Statistic};
// use crate::{
//     pdr::heap_frame_cube::HeapFrameCube,
//     utils::{generalize::generalize_by_ternary_simulation, state_transform::StateTransform},
// };
// use aig::Aig;
// use logic_form::{Clause, Cube, Lit};
// use sat_solver::SatResult;
// use std::{
//     collections::{BinaryHeap, HashSet},
//     mem::take,
//     sync::Arc,
// };

// pub struct Pdr {
//     delta_frames: Vec<Vec<Cube>>,
//     solvers: Vec<SolverPool>,
//     share: Arc<PdrShare>,
//     activity: Activity,
//     num_solver_per_frame: usize,

//     statistic: Statistic,
// }

// impl Pdr {
//     fn depth(&self) -> usize {
//         self.delta_frames.len() - 1
//     }

//     fn new_frame(&mut self) {
//         self.solvers.push(SolverPool::new(
//             self.share.clone(),
//             self.num_solver_per_frame,
//         ));
//         self.delta_frames.push(Vec::new());
//         self.statistic.num_frames = self.depth();
//     }

//     fn frame_add_cube(&mut self, frame: usize, cube: Cube, to_all: bool) {
//         assert!(cube.is_sorted_by_key(|x| x.var()));
//         for i in 1..=frame {
//             let cubes = take(&mut self.delta_frames[i]);
//             for c in cubes {
//                 if !cube.subsume(&c) {
//                     self.delta_frames[i].push(c);
//                 }
//             }
//         }
//         let begin = if to_all { 1 } else { frame };
//         self.delta_frames[frame].push(cube.clone());
//         let clause = !cube;
//         for i in begin..=frame {
//             self.solvers[i].add_clause(&clause);
//         }
//     }

//     fn blocked<'a>(&'a mut self, frame: usize, cube: &Cube) -> BlockResult<'a> {
//         self.solvers[frame - 1].pump_act_and_check_restart(&self.delta_frames[frame - 1..]);
//         self.solvers[frame - 1].blocked(cube)
//     }

//     fn down(&mut self, frame: usize, cube: Cube) -> Option<Cube> {
//         if cube.subsume(&self.share.init_cube) {
//             return None;
//         }
//         self.statistic.num_down_blocked += 1;
//         match self.blocked(frame, &cube) {
//             BlockResult::Yes(conflict) => Some(conflict.get_conflict()),
//             BlockResult::No(_) => None,
//         }
//     }

//     fn ctg_down(
//         &mut self,
//         frame: usize,
//         mut cube: Cube,
//         rec_depth: usize,
//         keep: &HashSet<Lit>,
//     ) -> Option<Cube> {
//         let mut ctgs = 0;
//         loop {
//             if cube.subsume(&self.share.init_cube) {
//                 return None;
//             }
//             match self.blocked(frame, &cube) {
//                 BlockResult::Yes(conflict) => return Some(conflict.get_conflict()),
//                 BlockResult::No(model) => {
//                     if rec_depth > 1 {
//                         return None;
//                     }
//                     let model = model.get_model();
//                     if ctgs < 3 && frame > 1 && !model.subsume(&self.share.init_cube) {
//                         if let BlockResult::Yes(conflict) = self.blocked(frame - 1, &model) {
//                             ctgs += 1;
//                             let conflict = conflict.get_conflict();
//                             let mut i = frame;
//                             while i <= self.depth() {
//                                 if let BlockResult::No(_) = self.blocked(i, &conflict) {
//                                     break;
//                                 }
//                                 i += 1;
//                             }
//                             let conflict = self.rec_mic(i - 1, conflict, rec_depth + 1);
//                             self.frame_add_cube(i - 1, conflict, true);
//                             continue;
//                         }
//                     }
//                     ctgs = 0;
//                     let cex_set: HashSet<Lit> = HashSet::from_iter(model.into_iter());
//                     let mut cube_new = Cube::new();
//                     for lit in cube {
//                         if cex_set.contains(&lit) {
//                             cube_new.push(lit);
//                         } else if keep.contains(&lit) {
//                             return None;
//                         }
//                     }
//                     cube = cube_new;
//                 }
//             }
//         }
//     }

//     fn rec_mic(&mut self, frame: usize, mut cube: Cube, rec_depth: usize) -> Cube {
//         self.statistic.average_mic_cube_len += cube.len();
//         let origin_len = cube.len();
//         let mut i = 0;
//         assert!(cube.is_sorted_by_key(|x| *x.var()));
//         cube = self.activity.sort_by_activity_ascending(cube);
//         let mut keep = HashSet::new();
//         while i < cube.len() {
//             let mut removed_cube = cube.clone();
//             removed_cube.remove(i);
//             match self.ctg_down(frame, removed_cube, rec_depth, &keep) {
//                 // match self.down(frame, removed_cube) {
//                 Some(new_cube) => {
//                     cube = new_cube;
//                     self.statistic.num_mic_drop_success += 1;
//                 }
//                 None => {
//                     self.statistic.num_mic_drop_fail += 1;
//                     keep.insert(cube[i]);
//                     i += 1;
//                 }
//             }
//         }
//         cube.sort_by_key(|x| *x.var());
//         for l in cube.iter() {
//             self.activity.pump_activity(l);
//         }
//         self.statistic.average_mic_droped_var += origin_len - cube.len();
//         self.statistic.average_mic_droped_var_percent +=
//             (origin_len - cube.len()) as f64 / origin_len as f64;
//         cube
//     }

//     fn mic(&mut self, frame: usize, cube: Cube) -> Cube {
//         self.rec_mic(frame, cube, 1)
//     }

//     fn generalize(&mut self, frame: usize, cube: Cube) -> (usize, Cube) {
//         let cube = self.mic(frame, cube);
//         for i in frame + 1..=self.depth() {
//             self.statistic.num_generalize_blocked += 1;
//             if let BlockResult::No(_) = self.blocked(i, &cube) {
//                 return (i, cube);
//             }
//         }
//         (self.depth() + 1, cube)
//     }

//     fn trivial_contained(&mut self, frame: usize, cube: &Cube) -> bool {
//         self.statistic.num_trivial_contained += 1;
//         for i in frame..=self.depth() {
//             for c in self.delta_frames[i].iter() {
//                 if c.subsume(cube) {
//                     self.statistic.num_trivial_contained_success += 1;
//                     return true;
//                 }
//             }
//         }
//         false
//     }

//     // fn sat_contained(&mut self, frame: usize, cube: &Cube) -> bool {
//     //     assert!(frame > 0);
//     //     self.statistic.num_sat_contained += 1;
//     //     match self.solvers[frame].solve(&cube) {
//     //         SatResult::Sat(_) => false,
//     //         SatResult::Unsat(_) => {
//     //             self.statistic.num_sat_contained_success += 1;
//     //             true
//     //         }
//     //     }
//     // }

//     fn rec_block(&mut self, frame: usize, cube: Cube) -> bool {
//         let mut heap = BinaryHeap::new();
//         let mut heap_num = vec![0; frame + 1];
//         heap.push(HeapFrameCube::new(frame, cube));
//         heap_num[frame] += 1;
//         while let Some(HeapFrameCube { frame, cube }) = heap.pop() {
//             assert!(cube.is_sorted_by_key(|x| x.var()));
//             if frame == 0 {
//                 return false;
//             }
//             println!("{:?}", heap_num);
//             self.statistic();
//             heap_num[frame] -= 1;
//             if self.trivial_contained(frame, &cube) {
//                 continue;
//             }
//             self.statistic.num_rec_block_blocked += 1;
//             match self.blocked(frame, &cube) {
//                 BlockResult::Yes(conflict) => {
//                     let conflict = conflict.get_conflict();
//                     let (frame, core) = self.generalize(frame, conflict);
//                     if frame < self.depth() {
//                         heap.push(HeapFrameCube::new(frame + 1, cube));
//                         heap_num[frame + 1] += 1;
//                     }
//                     self.frame_add_cube(frame - 1, core, true);
//                 }
//                 BlockResult::No(model) => {
//                     heap.push(HeapFrameCube::new(frame - 1, model.get_model()));
//                     heap.push(HeapFrameCube::new(frame, cube));
//                     heap_num[frame - 1] += 1;
//                     heap_num[frame] += 1;
//                 }
//             }
//         }
//         true
//     }

//     fn propagate(&mut self) -> bool {
//         for frame_idx in 1..self.depth() {
//             let frame = take(&mut self.delta_frames[frame_idx]);
//             for cube in frame {
//                 self.statistic.num_propagete_blocked += 1;
//                 match self.blocked(frame_idx + 1, &cube) {
//                     BlockResult::Yes(conflict) => {
//                         let conflict = conflict.get_conflict();
//                         assert!(conflict.len() <= cube.len());
//                         assert!(conflict.subsume(&cube));
//                         let to_all = conflict.len() < cube.len();
//                         self.frame_add_cube(frame_idx + 1, conflict, to_all);
//                     }
//                     BlockResult::No(_) => {
//                         // 利用cex？
//                         self.delta_frames[frame_idx].push(cube);
//                     }
//                 };
//             }
//             if self.delta_frames[frame_idx].is_empty() {
//                 return true;
//             }
//         }
//         false
//     }
// }

// impl Pdr {
//     pub fn new(aig: Aig) -> Self {
//         let transition_cnf = aig.get_cnf();
//         let init_cube = aig.latch_init_cube().to_cube();
//         let state_transform = StateTransform::new(&aig);
//         let share = Arc::new(PdrShare {
//             aig,
//             init_cube,
//             transition_cnf,
//             state_transform,
//         });
//         let num_solver_per_frame = 4;
//         let mut solvers = vec![SolverPool::new(share.clone(), num_solver_per_frame)];
//         let mut init_frame = Vec::new();
//         for l in share.aig.latchs.iter() {
//             let clause = Clause::from([Lit::new(l.input.into(), !l.init)]);
//             init_frame.push(!clause.clone());
//             solvers[0].add_clause(&clause);
//         }
//         let activity = Activity::new(&share.aig);
//         Self {
//             delta_frames: vec![init_frame],
//             solvers,
//             activity,
//             statistic: Statistic::default(),
//             share,
//             num_solver_per_frame,
//         }
//     }

//     pub fn check(&mut self) -> bool {
//         self.new_frame();
//         loop {
//             let last_frame_index = self.depth();
//             let solver = self.solvers[last_frame_index].fetch().unwrap();
//             while let SatResult::Sat(model) = solver.solve(&[self.share.aig.bads[0].to_lit()]) {
//                 self.statistic.num_get_bad_state += 1;
//                 let cex = generalize_by_ternary_simulation(
//                     &self.share.aig,
//                     model,
//                     &[self.share.aig.bads[0]],
//                 )
//                 .to_cube();
//                 self.solvers[last_frame_index].release(solver);
//                 // self.statistic();
//                 if !self.rec_block(last_frame_index, cex) {
//                     self.statistic();
//                     return false;
//                 }
//             }
//             self.statistic();
//             self.new_frame();
//             if self.propagate() {
//                 self.statistic();
//                 return true;
//             }
//         }
//     }
// }

// impl Pdr {
//     fn statistic(&self) {
//         for frame in self.delta_frames.iter() {
//             print!("{} ", frame.len())
//         }
//         println!();
//         println!("{:?}", self.statistic);
//     }
// }

// pub fn solve(aig: Aig) -> bool {
//     let mut pdr = Pdr::new(aig);
//     pdr.check()
// }
