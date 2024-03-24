use crate::Ic3;
use gipsat::{BlockResult, BlockResultNo};
use logic_form::Cube;

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
        self.gipsat.minimal_predecessor(unblock)
    }
}
