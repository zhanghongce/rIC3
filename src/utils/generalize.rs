use aig::{Aig, AigEdge, TernaryValue, AigCube};
use std::assert_matches::assert_matches;

pub fn generalize_by_ternary_simulation(
    aig: &Aig,
    cex: &[AigEdge],
    assumptions: &[AigEdge],
) -> AigCube {
    let mut value = vec![TernaryValue::X; aig.nodes.len()];
    let mut primary_inputs = Vec::new();
    let mut latch_inputs = Vec::new();
    for lit in cex {
        value[lit.node_id()] = if lit.compl() {
            TernaryValue::False
        } else {
            TernaryValue::True
        };
    }
    for input in &aig.inputs {
        primary_inputs.push(value[*input]);
    }
    for latch in &aig.latchs {
        latch_inputs.push(value[latch.input])
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
