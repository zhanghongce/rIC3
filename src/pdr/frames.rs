use super::broadcast::PdrSolverBroadcastSender;
use crate::utils::relation::{cube_subsume, cube_subsume_init};
use logic_form::Cube;
use std::{fmt::Debug, mem::take, sync::Arc};

pub struct Frames {
    pub frames: Vec<Vec<Cube>>,
    broadcast: Vec<PdrSolverBroadcastSender>,
}

impl Frames {
    // pub fn new(init: Vec<Cube>, broadcast: PdrSolverBroadcastSender) -> Self {
    //     Self {
    //         frames: vec![init],
    //         broadcast: vec![broadcast],
    //     }
    // }

    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            broadcast: Vec::new(),
        }
    }

    pub fn new_frame(&mut self, broadcast: PdrSolverBroadcastSender) {
        self.frames.push(Vec::new());
        self.broadcast.push(broadcast);
    }

    pub fn add_cube(&mut self, frame: usize, cube: Cube) {
        assert!(cube.is_sorted_by_key(|x| x.var()));
        assert!(!cube_subsume_init(&cube));
        if self.trivial_contained(frame, &cube) {
            return;
        }
        let begin = if frame == 0 {
            assert!(self.frames.len() == 1);
            0
        } else {
            let mut begin = 1;
            for i in 1..=frame {
                let cubes = take(&mut self.frames[i]);
                for c in cubes {
                    if cube_subsume(&c, &cube) {
                        begin = i + 1;
                    }
                    if !cube_subsume(&cube, &c) {
                        self.frames[i].push(c);
                    }
                }
            }
            begin
        };
        self.frames[frame].push(cube.clone());
        let clause = Arc::new(!cube);
        for i in begin..=frame {
            self.broadcast[i].send_clause(clause.clone());
        }
    }

    pub fn trivial_contained(&self, frame: usize, cube: &Cube) -> bool {
        for i in frame..self.frames.len() {
            for c in self.frames[i].iter() {
                if cube_subsume(c, cube) {
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
}

impl Debug for Frames {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.frames.fmt(f)
    }
}
