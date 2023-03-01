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
    let mut simulation = aig.ternary_simulate(&primary_inputs, &latch_inputs);
    for logic in assumptions {
        assert_matches!(
            simulation[logic.node_id()].not_if(logic.compl()),
            TernaryValue::True
        );
    }
    for (i, li) in latch_inputs.iter_mut().enumerate().take(aig.latchs.len()) {
        assert_matches!(*li, TernaryValue::True | TernaryValue::False);
        let origin = *li;
        *li = TernaryValue::X;
        simulation = aig.update_ternary_simulate(simulation, aig.latchs[i].input, TernaryValue::X);
        for logic in assumptions {
            match simulation[logic.node_id()].not_if(logic.compl()) {
                TernaryValue::True => (),
                TernaryValue::False => panic!(),
                TernaryValue::X => {
                    *li = origin;
                    simulation =
                        aig.update_ternary_simulate(simulation, aig.latchs[i].input, origin);
                    break;
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
