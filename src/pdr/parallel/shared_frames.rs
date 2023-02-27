use logic_form::Cube;
use std::sync::Mutex;

pub struct SharedFrames {
    pub frames: Vec<Mutex<Vec<Cube>>>,
}

impl SharedFrames {
    pub fn new() -> Self {
        todo!()
    }

    pub fn new_frame(&self) {
        todo!()
    }

    pub fn add_cube(&self, pdr_id: usize, frame: usize, cube: Cube) {
        todo!()
    }

    pub fn sync_frame(&self, pdr_id: usize) -> Vec<Vec<Cube>> {
        todo!()
    }
}
