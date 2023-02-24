use crate::utils::state_transform::StateTransform;
use aig::Aig;
use logic_form::{Cnf, Cube};

pub struct PdrShare {
    pub aig: Aig,
    pub init_cube: Cube,
    pub transition_cnf: Cnf,
    pub state_transform: StateTransform,
}
