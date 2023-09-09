use super::broadcast::PdrSolverBroadcastSender;
use crate::utils::relation::{cube_subsume, cube_subsume_init};
use logic_form::Cube;
use std::{
    fmt::Debug,
    mem::take,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};

pub struct Frames {
    time: Mutex<Duration>,
    pub frames: RwLock<Vec<Vec<Cube>>>,
    early_update: Mutex<usize>,
    broadcast: Vec<PdrSolverBroadcastSender>,
}

impl Frames {
    pub fn new() -> Self {
        Self {
            frames: RwLock::new(Vec::new()),
            broadcast: Vec::new(),
            time: Mutex::new(Duration::default()),
            early_update: Mutex::new(1),
        }
    }

    pub fn new_frame(&mut self, broadcast: PdrSolverBroadcastSender) {
        self.frames.write().unwrap().push(Vec::new());
        if !self.broadcast.is_empty() {
            self.broadcast.last_mut().unwrap().senders.pop();
        }
        self.broadcast.push(broadcast);
    }

    pub fn add_cube(&self, frame: usize, cube: Cube) {
        let start = Instant::now();
        assert!(cube.is_sorted_by_key(|x| x.var()));
        let mut frames = self.frames.write().unwrap();
        let mut early_update = self.early_update.lock().unwrap();
        let begin = if frame == 0 {
            assert!(frames.len() == 1);
            0
        } else {
            if Self::trivial_contained_inner(&frames, frame, &cube) {
                *self.time.lock().unwrap() += start.elapsed();
                return;
            }
            assert!(!cube_subsume_init(&cube));
            let mut begin = 1;
            for i in 1..=frame {
                let cubes = take(&mut frames[i]);
                for c in cubes {
                    if cube_subsume(&c, &cube) {
                        begin = i + 1;
                    }
                    if !cube_subsume(&cube, &c) {
                        frames[i].push(c);
                    } else {
                        *early_update = early_update.min(frame);
                    }
                }
            }
            begin
        };
        frames[frame].push(cube.clone());
        *early_update = (early_update.min(frame)).max(1);
        drop(frames);
        let clause = Arc::new(!cube);
        for i in begin..=frame {
            self.broadcast[i].send_clause(clause.clone());
        }
        *self.time.lock().unwrap() += start.elapsed();
    }

    fn trivial_contained_inner(frames: &[Vec<Cube>], frame: usize, cube: &Cube) -> bool {
        for i in frame..frames.len() {
            for c in frames[i].iter() {
                if cube_subsume(c, cube) {
                    return true;
                }
            }
        }
        false
    }

    pub fn trivial_contained(&self, frame: usize, cube: &Cube) -> bool {
        let frames = self.frames.read().unwrap();
        Self::trivial_contained_inner(&frames, frame, cube)
    }

    pub fn early_update(&self) -> usize {
        *self.early_update.lock().unwrap()
    }

    pub fn reset_early_update(&self) {
        *self.early_update.lock().unwrap() = self.frames.read().unwrap().len();
    }

    pub fn statistic(&self) {
        let frames = self.frames.read().unwrap();
        for frame in frames.iter() {
            print!("{} ", frame.len());
        }
        println!();
    }

    pub fn similar(&self, cube: &Cube, frame: usize) -> Option<Cube> {
        if frame == 1 {
            return None;
        }
        let frames = self.frames.read().unwrap();
        for c in frames[frame - 1].iter() {
            if cube_subsume(c, cube) {
                return Some(c.clone());
            }
        }
        None
    }
}

impl Debug for Frames {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.frames.fmt(f)
    }
}
