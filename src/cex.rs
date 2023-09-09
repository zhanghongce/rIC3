use crate::{
    basic::BasicShare, broadcast::PdrSolverBroadcastReceiver,
    utils::generalize::generalize_by_ternary_simulation,
};
use logic_form::Cube;
use sat_solver::{minisat::Solver, SatResult, SatSolver};
use std::sync::Arc;

pub struct Cex {
    solver: Solver,
    receiver: PdrSolverBroadcastReceiver,
    share: Arc<BasicShare>,
    cexs: Vec<Vec<Cube>>,
}

impl Cex {
    pub fn new(share: Arc<BasicShare>, receiver: PdrSolverBroadcastReceiver) -> Self {
        let mut solver = Solver::new();
        solver.set_random_seed(91648253_f64);
        solver.add_cnf(&share.as_ref().transition_cnf);

        Self {
            solver,
            receiver,
            share,
            cexs: vec![vec![]],
        }
    }

    pub fn new_frame(&mut self, receiver: PdrSolverBroadcastReceiver) {
        let mut solver = Solver::new();
        solver.set_random_seed(91648253_f64);
        solver.add_cnf(&self.share.as_ref().transition_cnf);
        self.solver = solver;
        self.receiver = receiver;
        self.cexs.push(vec![]);
    }

    fn fetch_clauses(&mut self) {
        while let Some(clause) = self.receiver.receive_clause() {
            self.solver.add_clause(&clause);
        }
        self.solver.simplify()
    }

    fn find_cex(&mut self) -> Option<Cube> {
        self.fetch_clauses();
        let bad = if self.share.aig.bads.is_empty() {
            self.share.aig.outputs[0]
        } else {
            self.share.aig.bads[0]
        };
        if let SatResult::Sat(model) = self.solver.solve(&[bad.to_lit()]) {
            self.share.statistic.lock().unwrap().num_get_bad_state += 1;
            let cex = generalize_by_ternary_simulation(&self.share.aig, model, &[bad]).to_cube();
            return Some(cex);
        }
        None
    }

    pub fn get(&mut self) -> Option<Cube> {
        if let Some(cex) = self.find_cex() {
            self.cexs.last_mut().unwrap().push(cex.clone());
            return Some(cex);
        }
        None
    }
}
