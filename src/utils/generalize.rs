use aig::{Aig, AigCube, AigEdge, TernaryValue};
use sat_solver::SatModel;
use std::assert_matches::assert_matches;

pub fn generalize_by_ternary_simulation<'a, M: SatModel<'a>>(
    aig: &Aig,
    model: M,
    assumptions: &[AigEdge],
) -> AigCube {
    let mut primary_inputs = Vec::new();
    let mut latch_inputs = Vec::new();
    for input in &aig.inputs {
        primary_inputs.push(model.var_value((*input).into()).into());
    }
    for latch in &aig.latchs {
        latch_inputs.push(model.var_value(latch.input.into()).into());
    }
    let simulation = aig.ternary_simulate(&primary_inputs, &latch_inputs);
    for logic in assumptions {
        assert_matches!(
            simulation[logic.node_id()].not_if(logic.compl()),
            TernaryValue::True
        );
    }
    for i in 0..aig.latchs.len() {
        if let TernaryValue::True | TernaryValue::False = latch_inputs[i] {
            let origin = latch_inputs[i];
            latch_inputs[i] = TernaryValue::X;
            let simulation = aig.ternary_simulate(&primary_inputs, &latch_inputs);
            for logic in assumptions {
                match simulation[logic.node_id()].not_if(logic.compl()) {
                    TernaryValue::True => (),
                    TernaryValue::False => panic!(),
                    TernaryValue::X => latch_inputs[i] = origin,
                }
            }
        }
    }
    let mut cube = AigCube::new();
    for (i, value) in latch_inputs.iter().enumerate().take(aig.latchs.len()) {
        match value {
            TernaryValue::True => {
                cube.push(AigEdge::new(aig.latchs[i].input, false));
            }
            TernaryValue::False => {
                cube.push(AigEdge::new(aig.latchs[i].input, true));
            }
            TernaryValue::X => (),
        }
    }
    cube
}
