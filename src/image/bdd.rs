use crate::utils::aig_bdd::{aig_trans_bdd, aig_trans_init_bdd, aig_trans_logic_bdd};
use aig::Aig;
use cudd::Cudd;
use std::time::Instant;

pub fn solve(aig: Aig, forword: bool) -> bool {
    let cudd = Cudd::new();
    let trans = aig_trans_bdd(&aig, &cudd);
    let bad = aig_trans_logic_bdd(&aig, &cudd, aig.bads[0]);
    let init = aig_trans_init_bdd(&aig, &cudd);
    let mut reach = if forword { init.clone() } else { bad.clone() };
    let mut frontier = reach.clone();
    let to = if forword { bad } else { init };
    let mut deep = 0;
    let start = Instant::now();
    loop {
        deep += 1;
        // dbg!(deep);
        if &frontier & &to != cudd.constant(false) {
            dbg!(start.elapsed());
            return false;
        }
        let new_frontier = if forword {
            frontier.post_image(&trans)
        } else {
            frontier.pre_image(&trans)
        };
        let new_frontier = !&reach & new_frontier;
        if new_frontier == cudd.constant(false) {
            dbg!(start.elapsed());
            return true;
        }
        reach |= &new_frontier;
        frontier = new_frontier;
    }
}
