use super::{solver::BlockResult, Ic3};
use crate::{basic::Ic3Error, utils::relation::cube_subsume_init};
use logic_form::{Cube, Lit};
use std::{collections::HashSet, time::Instant};

impl Ic3 {
    fn down(&mut self, frame: usize, cube: &Cube) -> Result<Option<Cube>, Ic3Error> {
        self.check_stop_block()?;
        if cube_subsume_init(&self.share.init, cube) {
            return Ok(None);
        }
        self.share.statistic.lock().unwrap().num_down_blocked += 1;
        Ok(match self.blocked_with_ordered(frame, cube, false) {
            BlockResult::Yes(conflict) => Some(conflict.get_conflict()),
            BlockResult::No(_) => None,
        })
    }

    fn double_drop_down(
        &mut self,
        frame: usize,
        cube: &Cube,
        first: Lit,
        second: Lit,
    ) -> Result<Cube, Option<Lit>> {
        let first_next = self.share.state_transform.lit_next(first);
        let second_next = self.share.state_transform.lit_next(second);
        if cube_subsume_init(&self.share.init, &cube) {
            let mut cube = cube.clone();
            cube.push(second);
            return if cube_subsume_init(&self.share.init, &cube) {
                Err(Some(first))
            } else {
                Err(Some(second))
            };
        }
        match self.blocked_with_polarity_with_ordered(
            frame,
            &cube,
            &[first_next, second_next],
            false,
        ) {
            BlockResult::Yes(conflict) => Ok(conflict.get_conflict()),
            BlockResult::No(mut model) => Err(
                match (model.lit_value(first_next), model.lit_value(second_next)) {
                    (true, false) => Some(second),
                    (false, true) => Some(first),
                    (false, false) => None,
                    (true, true) => panic!(),
                },
            ),
        }
    }

    fn ctg_down(
        &mut self,
        frame: usize,
        mut cube: &Cube,
        keep: &HashSet<Lit>,
    ) -> Result<Option<Cube>, Ic3Error> {
        let mut cube = cube.clone();
        self.share.statistic.lock().unwrap().num_ctg_down += 1;
        let mut ctgs = 0;
        loop {
            self.check_stop_block()?;
            if cube_subsume_init(&self.share.init, &cube) {
                return Ok(None);
            }
            match self.blocked(frame, &cube) {
                BlockResult::Yes(conflict) => return Ok(Some(conflict.get_conflict())),
                BlockResult::No(model) => {
                    let mut model = model.get_model();
                    if ctgs < 3 && frame > 1 && !cube_subsume_init(&self.share.init, &model) {
                        if self.share.args.cav23 {
                            self.cav23_activity.sort_by_activity(&mut model, false);
                        }
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
                            let conflict = self.mic(i - 1, conflict, true)?;
                            self.add_cube(i - 1, conflict);
                            continue;
                        }
                    }
                    ctgs = 0;
                    let cex_set: HashSet<Lit> = HashSet::from_iter(model);
                    let mut cube_new = Cube::new();
                    for lit in cube {
                        if cex_set.contains(&lit) {
                            cube_new.push(lit);
                        } else if keep.contains(&lit) {
                            return Ok(None);
                        }
                    }
                    cube = cube_new;
                }
            }
        }
    }

    // fn double_drop_ctg_down(
    //     &mut self,
    //     frame: usize,
    //     mut cube: Cube,
    //     first: Lit,
    //     second: Lit,
    //     keep: &HashSet<Lit>,
    // ) -> Result<Cube, Option<Lit>> {
    //     let mut ctgs = 0;
    //     let first_next = self.share.state_transform.lit_next(first);
    //     let second_next = self.share.state_transform.lit_next(second);
    //     let mut err: Option<Option<Lit>> = None;
    //     loop {
    //         if cube_subsume_init(&self.share.init, &cube) {
    //             if err.is_none() {
    //                 cube.push(second);
    //                 err = Some(if cube_subsume_init(&self.share.init, &cube) {
    //                     Some(first)
    //                 } else {
    //                     Some(second)
    //                 });
    //             }
    //             return Err(err.unwrap());
    //         }
    //         match self.blocked_with_polarity(frame, &cube, &[first_next, second_next]) {
    //             BlockResult::Yes(conflict) => return Ok(conflict.get_conflict()),
    //             BlockResult::No(mut model) => {
    //                 if err.is_none() {
    //                     err = Some(
    //                         match (model.lit_value(first_next), model.lit_value(second_next)) {
    //                             (true, false) => Some(second),
    //                             (false, true) => Some(first),
    //                             (false, false) => None,
    //                             (true, true) => panic!(),
    //                         },
    //                     );
    //                 }
    //                 let model = model.get_model();
    //                 if ctgs < 3 && frame > 1 && !cube_subsume_init(&self.share.init, &model) {
    //                     if let BlockResult::Yes(conflict) = self.blocked(frame - 1, &model) {
    //                         ctgs += 1;
    //                         let conflict = conflict.get_conflict();
    //                         let mut i = frame;
    //                         while i <= self.depth() {
    //                             if let BlockResult::No(_) = self.blocked(i, &conflict) {
    //                                 break;
    //                             }
    //                             i += 1;
    //                         }
    //                         let conflict = self.double_drop_mic(i - 1, conflict, true).unwrap();
    //                         self.add_cube(i - 1, conflict);
    //                         continue;
    //                     }
    //                 }
    //                 ctgs = 0;
    //                 let cex_set: HashSet<Lit> = HashSet::from_iter(model);
    //                 let mut cube_new = Cube::new();
    //                 for lit in cube {
    //                     if cex_set.contains(&lit) {
    //                         cube_new.push(lit);
    //                     } else if keep.contains(&lit) {
    //                         return Err(err.unwrap());
    //                     }
    //                 }
    //                 cube = cube_new;
    //             }
    //         }
    //     }
    // }

    fn handle_down_success(&mut self, cube: Cube, i: usize, mut new_cube: Cube) -> (Cube, usize) {
        new_cube = cube
            .iter()
            .filter(|l| new_cube.contains(l))
            .cloned()
            .collect();
        let new_i = new_cube
            .iter()
            .position(|l| !(cube[0..i]).contains(l))
            .unwrap_or(new_cube.len());
        if new_i < new_cube.len() {
            assert!(!(cube[0..=i]).contains(&new_cube[new_i]))
        }
        (new_cube, new_i)
    }

    pub fn mic(&mut self, frame: usize, mut cube: Cube, simple: bool) -> Result<Cube, Ic3Error> {
        let start = Instant::now();
        self.share.statistic.lock().unwrap().average_mic_cube_len += cube.len();
        self.activity.sort_by_activity(&mut cube, true);
        let mut keep = HashSet::new();
        let cav23_parent = self.share.args.cav23.then(|| {
            self.cav23_activity.sort_by_activity(&mut cube, true);
            let mut similar = self.frames.similar(&cube, frame);
            similar.sort_by(|a, b| {
                self.cav23_activity
                    .cube_average_activity(b)
                    .partial_cmp(&self.cav23_activity.cube_average_activity(a))
                    .unwrap()
            });
            let similar = similar.into_iter().nth(0);
            if let Some(similar) = &similar {
                for l in similar.iter() {
                    keep.insert(*l);
                }
            }
            similar
        });
        let mut i = 0;
        while i < cube.len() {
            assert!(!keep.contains(&cube[i]));
            let mut removed_cube = cube.clone();
            removed_cube.remove(i);
            let res = if simple {
                self.down(frame, &removed_cube)?
            } else {
                self.ctg_down(frame, &removed_cube, &keep)?
            };
            match res {
                Some(new_cube) => {
                    // (cube, i) = self.handle_down_success(cube, i, new_cube);
                    cube = removed_cube;
                    self.share.statistic.lock().unwrap().num_mic_drop_success += 1;
                }
                None => {
                    self.share.statistic.lock().unwrap().num_mic_drop_fail += 1;
                    keep.insert(cube[i]);
                    i += 1;
                }
            }
        }
        if let Some(Some(cav23)) = cav23_parent {
            cube.sort_by_key(|x| *x.var());
            if cube.ordered_subsume(&cav23) {
                self.cav23_activity.pump_cube_activity(&cube);
            }
        }
        self.activity.pump_cube_activity(&cube);
        if simple {
            self.share.statistic.lock().unwrap().simple_mic_time += start.elapsed()
        } else {
            self.share.statistic.lock().unwrap().mic_time += start.elapsed()
        }
        Ok(cube)
    }

    pub fn double_drop_mic(
        &mut self,
        frame: usize,
        mut cube: Cube,
        simple: bool,
    ) -> Result<Cube, Ic3Error> {
        self.share.statistic.lock().unwrap().average_mic_cube_len += cube.len();
        self.activity.sort_by_activity(&mut cube, true);
        let mut keep = HashSet::new();
        let mut i = 0;
        while i < cube.len() {
            assert!(!keep.contains(&cube[i]));
            let mut removed_cube = cube.clone();
            if i + 1 < cube.len() {
                assert!(!keep.contains(&cube[i + 1]));
                let first = removed_cube.remove(i);
                let second = removed_cube.remove(i);
                let res = if simple {
                    self.double_drop_down(frame, &removed_cube, first, second)
                } else {
                    todo!()
                    // self.double_drop_ctg_down(frame, removed_cube, first, second, &keep)
                };
                match res {
                    Ok(new_cube) => {
                        self.share.statistic.lock().unwrap().test_a += 1;
                        // (cube, i) = self.handle_down_success(cube, i, new_cube);
                        cube = removed_cube;
                    }
                    Err(Some(fail)) => {
                        self.share.statistic.lock().unwrap().test_b += 1;
                        if fail != first {
                            cube.swap(i, i + 1);
                        }
                        assert!(cube[i] == fail);
                        keep.insert(cube[i]);
                        i += 1;
                    }
                    Err(None) => {
                        self.share.statistic.lock().unwrap().test_c += 1;
                        assert!(cube[i] == first);
                        let mut removed_cube = cube.clone();
                        removed_cube.remove(i);
                        match self.down(frame, &removed_cube)? {
                            Some(new_cube) => {
                                // (cube, i) = self.handle_down_success(cube, i, new_cube);
                                cube = removed_cube;
                            }
                            None => {
                                keep.insert(cube[i]);
                                i += 1;
                            }
                        }
                    }
                }
            } else {
                removed_cube.remove(i);
                match self.down(frame, &removed_cube)? {
                    Some(new_cube) => {
                        cube = removed_cube;
                        // (cube, i) = self.handle_down_success(cube, i, new_cube);
                    }
                    None => {
                        keep.insert(cube[i]);
                        i += 1;
                    }
                }
            }
        }
        cube.sort_by_key(|x| *x.var());
        self.activity.pump_cube_activity(&cube);
        Ok(cube)
    }
}
