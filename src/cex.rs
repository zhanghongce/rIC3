use crate::{
    basic::BasicShare, broadcast::PdrSolverBroadcastReceiver,
    utils::generalize::generalize_by_ternary_simulation,
};
use logic_form::Cube;
use sat_solver::{minisat::Solver, SatResult, SatSolver};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    sync::Arc,
};

pub struct Cex {
    solver: Solver,
    receiver: PdrSolverBroadcastReceiver,
    share: Arc<BasicShare>,
    cexs: Vec<Vec<Cube>>,
    cached_cex: Option<Vec<Vec<Cube>>>,
}

impl Cex {
    pub fn new(share: Arc<BasicShare>, receiver: PdrSolverBroadcastReceiver) -> Self {
        let mut solver = Solver::new();
        solver.set_random_seed(91648253_f64);
        solver.add_cnf(&share.as_ref().transition_cnf);
        let cached_cex = File::open("cexs.json").ok().map(|mut file| {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            serde_json::from_slice(&buffer).unwrap()
        });

        Self {
            solver,
            receiver,
            share,
            cached_cex,
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
        self.cached_cex
            .as_mut()
            .map(|cached_cex| cached_cex.remove(0));
    }

    fn fetch_clauses(&mut self) {
        while let Some(clause) = self.receiver.receive_clause() {
            self.solver.add_clause(&clause);
        }
        self.solver.simplify()
    }

    fn find_cex(&mut self) -> Option<Cube> {
        self.fetch_clauses();
        if let SatResult::Sat(model) = self.solver.solve(&[self.share.aig.bads[0].to_lit()]) {
            self.share.statistic.lock().unwrap().num_get_bad_state += 1;
            let cex =
                generalize_by_ternary_simulation(&self.share.aig, model, &[self.share.aig.bads[0]])
                    .to_cube();
            return Some(cex);
        }
        None
    }

    pub fn get(&mut self) -> Option<Cube> {
        if let Some(cached_cex) = &mut self.cached_cex {
            if cached_cex[0].is_empty() {
                return None;
            }
            return Some(cached_cex[0].remove(0));
        };
        // todo!();
        if let Some(cex) = self.find_cex() {
            self.cexs.last_mut().unwrap().push(cex.clone());
            return Some(cex);
        }
        None
    }

    pub fn store_cex(&mut self) {
        if self.cached_cex.is_none() {
            let json = serde_json::to_string(&self.cexs).unwrap();
            let file_path = Path::new("cexs.json");
            let mut file = File::create(&file_path).unwrap();
            file.write_all(json.as_bytes()).unwrap();
        }
    }
}
