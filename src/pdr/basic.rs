use crate::utils::state_transform::StateTransform;
use aig::Aig;
use logic_form::Cnf;

pub struct BasicShare {
    pub aig: Aig,
    pub transition_cnf: Cnf,
    pub state_transform: StateTransform,
}
