use crate::Ic3;
use gipsat::{BlockResult, BlockResultNo};
use logic_form::Cube;
use satif::SatifSat;

impl Ic3 {
    pub fn blocked_with_ordered(
        &mut self,
        frame: usize,
        cube: &Cube,
        ascending: bool,
        strengthen: bool,
        bucket: bool,
    ) -> BlockResult {
        let mut ordered_cube = cube.clone();
        self.activity.sort_by_activity(&mut ordered_cube, ascending);
        self.gipsat
            .blocked(frame, &ordered_cube, strengthen, bucket)
    }
}

impl Ic3 {
    pub fn unblocked_model(&mut self, unblock: BlockResultNo) -> Cube {
        let mut latchs = Cube::new();
        for latch in self.ts.latchs.iter() {
            let lit = latch.lit();
            match unblock.sat.lit_value(lit) {
                Some(true) => latchs.push(lit),
                Some(false) => latchs.push(!lit),
                None => (),
            }
        }
        self.activity.sort_by_activity(&mut latchs, false);
        self.gipsat.minimal_predecessor(unblock, latchs)
    }
}
