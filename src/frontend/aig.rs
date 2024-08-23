use super::abc::abc_preprocess;
use crate::Options;
use aig::Aig;
use std::collections::{HashMap, HashSet};

pub fn aig_preprocess(aig: &Aig, options: &Options) -> (Aig, HashMap<usize, usize>) {
    let (mut aig, mut remap) = aig.coi_refine();
    if !(options.ic3 && options.ic3_options.inn) {
        let mut remap_retain = HashSet::new();
        remap_retain.insert(0);
        for i in aig.inputs.iter() {
            remap_retain.insert(*i);
        }
        for l in aig.latchs.iter() {
            remap_retain.insert(l.input);
        }
        remap.retain(|x, _| remap_retain.contains(x));
        aig = abc_preprocess(aig);
    }
    aig.constraints.retain(|e| !e.is_constant(true));
    (aig, remap)
}
