use crate::{utils::relation::cube_subsume_init, worker::Ic3Worker};
use logic_form::Cube;
use pic3::{Lemma, LemmaSharer};
use std::{
    fmt::Debug,
    mem::take,
    ops::{Deref, DerefMut},
    sync::Arc,
};

pub struct Frames {
    pub frames: Vec<Vec<Cube>>,
    sharer: Option<LemmaSharer>,
}

impl Frames {
    pub fn new(sharer: Option<LemmaSharer>) -> Self {
        Self {
            frames: Vec::new(),
            sharer,
        }
    }

    pub fn new_frame(&mut self) {
        self.frames.push(Vec::new());
    }

    pub fn trivial_contained(&self, frame: usize, cube: &Cube) -> bool {
        for i in frame..self.frames.len() {
            for c in self.frames[i].iter() {
                if c.ordered_subsume(cube) {
                    return true;
                }
            }
        }
        false
    }

    pub fn statistic(&self) {
        for frame in self.frames.iter() {
            print!("{} ", frame.len());
        }
        println!();
    }

    pub fn similar(&self, cube: &Cube, frame: usize) -> Vec<Cube> {
        let mut cube = cube.clone();
        cube.sort_by_key(|l| l.var());
        let mut res = Vec::new();
        if frame == 1 {
            return res;
        }
        for c in self.frames[frame - 1].iter() {
            if c.ordered_subsume(&cube) {
                res.push(c.clone());
            }
        }
        res
    }
}

impl Debug for Frames {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.frames.fmt(f)
    }
}

impl Deref for Frames {
    type Target = Vec<Vec<Cube>>;

    fn deref(&self) -> &Self::Target {
        &self.frames
    }
}

impl DerefMut for Frames {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.frames
    }
}

impl Ic3Worker {
    pub fn add_cube(&mut self, frame: usize, mut cube: Cube) {
        cube.sort_by_key(|x| x.var());
        if frame == 0 {
            assert!(self.frames.len() == 1);
            self.solvers[0].add_clause(&!&cube);
            self.frames[0].push(cube);
            return;
        }
        if self.frames.trivial_contained(frame, &cube) {
            return;
        }
        assert!(!cube_subsume_init(&self.share.init, &cube));
        let mut begin = 1;
        for i in 1..=frame {
            let cubes = take(&mut self.frames[i]);
            for c in cubes {
                if c.ordered_subsume(&cube) {
                    begin = i + 1;
                }
                if !cube.ordered_subsume(&c) {
                    self.frames[i].push(c);
                }
            }
        }
        self.frames[frame].push(cube.clone());
        if let Some(sharer) = &mut self.frames.sharer {
            sharer.share(Lemma {
                frame_idx: frame,
                cube: cube.clone(),
            })
        }
        let clause = Arc::new(!cube);
        for i in begin..=frame {
            self.solvers[i].add_clause(&clause);
        }
    }

    pub fn acquire_lemma(&mut self) {
        let depth = self.depth();
        if let Some(sharer) = self.frames.sharer.as_mut() {
            if let Some(Lemma { frame_idx, cube }) = sharer.acquire(depth) {
                self.add_cube(frame_idx, cube)
            }
        }
    }
}
