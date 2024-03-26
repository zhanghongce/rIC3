use crate::Ic3;
use logic_form::Cube;

impl Ic3 {
    pub fn blocked_with_ordered(
        &mut self,
        frame: usize,
        cube: &Cube,
        ascending: bool,
        strengthen: bool,
    ) -> bool {
        let mut ordered_cube = cube.clone();
        self.activity.sort_by_activity(&mut ordered_cube, ascending);
        self.gipsat
            .inductive(frame, &ordered_cube, strengthen, true)
    }
}
