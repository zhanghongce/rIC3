use crate::Ic3;
use logic_form::{Cube, Lemma};

impl Ic3 {
    pub fn add_cube(&mut self, frame: usize, cube: Cube) {
        let lemma = Lemma::new(cube);
        self.gipsat.add_lemma(frame, lemma);
    }
}
