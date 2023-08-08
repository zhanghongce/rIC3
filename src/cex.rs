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
    cexs: Vec<Cube>,
    acts: Cube,
    begin_drop: bool,
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
            cexs: Vec::new(),
            acts: Cube::new(),
            begin_drop: false,
        }
    }

    fn fetch_clauses(&mut self) {
        while let Some(clause) = self.receiver.receive_clause() {
            self.solver.add_clause(&clause);
        }
        self.solver.simplify()
    }

    fn block_cex(&mut self, cex: &Cube) {
        let mut generalize_cex = cex.clone();
        for c in self.cexs.iter() {
            generalize_cex = cex.intersection(c);
        }
        self.cexs.push(cex.clone());
        let act = self.solver.new_var().into();
        self.acts.push(act);
        let mut tmp_cls = !generalize_cex;
        tmp_cls.push(!act);
        self.solver.add_clause(&tmp_cls);
    }

    fn drop_act(&mut self) -> bool {
        assert!(self.acts.len() == self.cexs.len());
        if self.acts.is_empty() {
            return false;
        }
        let act = !self.acts.remove(0);
        self.solver.release_var(act);
        self.cexs.remove(0);
        true
    }

    fn find_cex(&mut self) -> Option<Cube> {
        self.fetch_clauses();
        let mut assumption = self.acts.clone();
        assumption.push(self.share.aig.bads[0].to_lit());
        if let SatResult::Sat(model) = self.solver.solve(&assumption) {
            self.share.statistic.lock().unwrap().num_get_bad_state += 1;
            let cex =
                generalize_by_ternary_simulation(&self.share.aig, model, &[self.share.aig.bads[0]])
                    .to_cube();
            return Some(cex);
        }
        None
    }

    pub fn get(&mut self) -> Option<Cube> {
        if !self.begin_drop {
            if let Some(cex) = self.find_cex() {
                self.block_cex(&cex);
                // for l in cex.iter() {
                //     SatSolver::set_polarity(&mut self.solver, !l.clone());
                // }
                return Some(cex);
            }
        }
        // dbg!("xxx");
        self.begin_drop = true;
        while self.drop_act() {}
        // while self.drop_act() {
        //     if let Some(cex) = self.find_cex() {
        //         self.block_cex(&cex);
        //         return Some(cex);
        //     }
        // }
        if let Some(cex) = self.find_cex() {
            // self.block_cex(&cex);
            return Some(cex);
        }
        None
    }
}
