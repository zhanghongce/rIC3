use super::{solver::BlockResult, Pdr};
use crate::utils::relation::cube_subsume_init;
use logic_form::{Cube, Lit};
use sat_solver::SatSolver;
use std::{collections::HashSet, time::Instant};

impl Pdr {
    fn down(&mut self, frame: usize, cube: Cube) -> Option<Cube> {
        if cube_subsume_init(&cube) {
            return None;
        }
        self.share.statistic.lock().unwrap().num_down_blocked += 1;
        match self.blocked(frame, &cube) {
            BlockResult::Yes(conflict) => Some(conflict.get_conflict()),
            BlockResult::No(_) => None,
        }
    }

    fn ctg_down(&mut self, frame: usize, mut cube: Cube, keep: &HashSet<Lit>) -> Option<Cube> {
        self.share.statistic.lock().unwrap().num_ctg_down += 1;
        let mut ctgs = 0;
        loop {
            if cube_subsume_init(&cube) {
                return None;
            }
            match self.blocked(frame, &cube) {
                BlockResult::Yes(conflict) => return Some(conflict.get_conflict()),
                BlockResult::No(model) => {
                    let model = model.get_model();
                    if ctgs < 3 && frame > 1 && !cube_subsume_init(&model) {
                        if let BlockResult::Yes(conflict) = self.blocked(frame - 1, &model) {
                            ctgs += 1;
                            let conflict = conflict.get_conflict();
                            let mut i = frame;
                            while i <= self.depth() {
                                if let BlockResult::No(_) = self.blocked(i, &conflict) {
                                    break;
                                }
                                i += 1;
                            }
                            let conflict = self.mic(i - 1, conflict, true);
                            self.frame_add_cube(i - 1, conflict);
                            continue;
                        }
                    }
                    ctgs = 0;
                    let cex_set: HashSet<Lit> = HashSet::from_iter(model.into_iter());
                    let mut cube_new = Cube::new();
                    for lit in cube {
                        if cex_set.contains(&lit) {
                            cube_new.push(lit);
                        } else if keep.contains(&lit) {
                            return None;
                        }
                    }
                    cube = cube_new;
                }
            }
        }
    }

    pub fn mic(&mut self, frame: usize, mut cube: Cube, simple: bool) -> Cube {
        let start = Instant::now();
        if simple {
            self.share.statistic.lock().unwrap().num_simple_mic += 1;
        } else {
            self.share.statistic.lock().unwrap().num_normal_mic += 1;
        }
        self.share.statistic.lock().unwrap().average_mic_cube_len += cube.len();
        let mut i = 0;
        assert!(cube.is_sorted_by_key(|x| *x.var()));
        cube = self.activity.sort_by_activity_ascending(cube);
        let mut keep = HashSet::new();
        while i < cube.len() {
            let mut removed_cube = cube.clone();
            removed_cube.remove(i);
            let res = if simple {
                self.down(frame, removed_cube)
            } else {
                self.ctg_down(frame, removed_cube, &keep)
            };
            match res {
                Some(new_cube) => {
                    cube = new_cube;
                    let clause = !cube.clone();
                    for i in 1..=frame {
                        self.solvers[i].add_clause(&clause);
                        self.solvers[i].solver.simplify();
                    }
                    self.share.statistic.lock().unwrap().num_mic_drop_success += 1;
                }
                None => {
                    self.share.statistic.lock().unwrap().num_mic_drop_fail += 1;
                    keep.insert(cube[i]);
                    i += 1;
                }
            }
        }
        cube.sort_by_key(|x| *x.var());
        for l in cube.iter() {
            self.activity.pump_activity(l);
        }
        if simple {
            self.share.statistic.lock().unwrap().simple_mic_time += start.elapsed()
        } else {
            self.share.statistic.lock().unwrap().mic_time += start.elapsed()
        }
        cube
    }
}
