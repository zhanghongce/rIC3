use super::{solver::BlockResult, worker::Ic3Worker};
use crate::{basic::Ic3Error, utils::relation::cube_subsume_init};
use logic_form::{Cube, Lit};
use std::{collections::HashSet, time::Instant};

impl Ic3Worker {
    fn test_down(
        &mut self,
        frame: usize,
        cube: Cube,
        first: Lit,
        second: Lit,
    ) -> Result<Cube, Lit> {
        let first_next = self.share.state_transform.lit_next(first);
        let second_next = self.share.state_transform.lit_next(second);
        if cube_subsume_init(&self.share.init, &cube) {
            let mut tmp = cube.clone();
            tmp.push(first);
            if cube_subsume_init(&self.share.init, &tmp) {
                return Err(second);
            } else {
                return Err(first);
            }
        }
        match self.blocked_with_polarity(frame, &cube, &[first_next, second_next]) {
            // match self.blocked(frame, &cube) {
            BlockResult::Yes(conflict) => Ok(conflict.get_conflict()),
            BlockResult::No(mut model) => {
                let res = if !model.lit_value(first_next) {
                    Err(first)
                } else {
                    assert!(!model.lit_value(second_next));
                    Err(second)
                };
                match (model.lit_value(first_next), model.lit_value(second_next)) {
                    (true, true) => todo!(),
                    (true, false) => self.share.statistic.lock().unwrap().test_x += 1,
                    (false, true) => self.share.statistic.lock().unwrap().test_x += 1,
                    (false, false) => self.share.statistic.lock().unwrap().test_y += 1,
                }
                res
            }
        }
    }

    fn down(&mut self, frame: usize, cube: Cube) -> Result<Option<Cube>, Ic3Error> {
        self.check_stop_block()?;
        if cube_subsume_init(&self.share.init, &cube) {
            return Ok(None);
        }
        self.share.statistic.lock().unwrap().num_down_blocked += 1;
        Ok(match self.blocked(frame, &cube) {
            BlockResult::Yes(conflict) => Some(conflict.get_conflict()),
            BlockResult::No(_) => None,
        })
    }

    fn ctg_down(
        &mut self,
        frame: usize,
        mut cube: Cube,
        keep: &HashSet<Lit>,
    ) -> Result<Option<Cube>, Ic3Error> {
        todo!();
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
                        assert!(!cube_subsume_init(&self.share.init, &model));
                        if self.share.args.cav23 {
                            self.cav23_activity.sort_by_activity_descending(&mut model);
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

    pub fn mic(&mut self, frame: usize, mut cube: Cube, simple: bool) -> Result<Cube, Ic3Error> {
        let start = Instant::now();
        if simple {
            self.share.statistic.lock().unwrap().num_simple_mic += 1;
        } else {
            self.share.statistic.lock().unwrap().num_normal_mic += 1;
        }
        self.share.statistic.lock().unwrap().average_mic_cube_len += cube.len();
        let mut i = 0;
        self.activity.sort_by_activity_ascending(&mut cube);
        let mut keep = HashSet::new();
        let cav23_parent = self.share.args.cav23.then(|| {
            self.cav23_activity.sort_by_activity_ascending(&mut cube);
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
        while i < cube.len() {
            if keep.contains(&cube[i]) {
                i += 1;
                continue;
            }
            let mut removed_cube = cube.clone();
            removed_cube.remove(i);
            let res = if simple {
                self.down(frame, removed_cube)?
            } else {
                self.ctg_down(frame, removed_cube, &keep)?
            };
            match res {
                Some(new_cube) => {
                    cube = new_cube;
                    let clause = !cube.clone();
                    for i in 1..=frame {
                        self.solvers[i].add_clause(&clause);
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
        if let Some(Some(cav23)) = cav23_parent {
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

    pub fn test_mic(
        &mut self,
        frame: usize,
        mut cube: Cube,
        simple: bool,
    ) -> Result<Cube, Ic3Error> {
        self.share.statistic.lock().unwrap().average_mic_cube_len += cube.len();
        let mut i = 0;
        self.activity.sort_by_activity_ascending(&mut cube);
        while i < cube.len() {
            let mut removed_cube = cube.clone();
            if i + 1 < cube.len() {
                let first = removed_cube.remove(i);
                let second = removed_cube.remove(i);
                match self.test_down(frame, removed_cube.clone(), first, second) {
                    Ok(new_cube) => {
                        self.share.statistic.lock().unwrap().test_a += 1;
                        // for j in 0..i {
                        //     if cube[j] != new_cube[j] {
                        //     }
                        // }
                        cube = new_cube;
                        let clause = !cube.clone();
                        for i in 1..=frame {
                            self.solvers[i].add_clause(&clause);
                        }
                    }
                    Err(fail) => {
                        self.share.statistic.lock().unwrap().test_b += 1;
                        assert!(cube[i] == first);
                        assert!(cube[i + 1] == second);
                        cube[i] = fail;
                        cube[i + 1] = if fail == first {
                            second
                        } else {
                            assert!(fail == second);
                            first
                        };
                        i += 1;
                        let mut removed_cube = cube.clone();
                        removed_cube.remove(i);
                        match self.down(frame, removed_cube)? {
                            Some(new_cube) => {
                                cube = new_cube;
                                let clause = !cube.clone();
                                for i in 1..=frame {
                                    self.solvers[i].add_clause(&clause);
                                }
                            }
                            None => {
                                i += 1;
                            }
                        }
                    }
                }
            } else {
                removed_cube.remove(i);
                match self.down(frame, removed_cube)? {
                    Some(new_cube) => {
                        cube = new_cube;
                        let clause = !cube.clone();
                        for i in 1..=frame {
                            self.solvers[i].add_clause(&clause);
                        }
                    }
                    None => {
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
