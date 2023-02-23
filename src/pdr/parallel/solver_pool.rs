use logic_form::Clause;

use crate::pdr::{share::PdrShare, solver::PdrSolver};
use std::sync::Arc;

pub struct SolverPool {
    solvers: Vec<PdrSolver>,
    num_solver: usize,
}

impl SolverPool {
    pub fn new(share: Arc<PdrShare>, num_solver: usize) -> Self {
        let mut solvers = Vec::new();
        for _ in 0..num_solver {
            solvers.push(PdrSolver::new(share.clone()))
        }
        Self {
            solvers,
            num_solver,
        }
    }

    pub fn fetch(&mut self) -> Option<PdrSolver> {
        if self.solvers.is_empty() {
            None
        } else {
            let last = self.solvers.len() - 1;
            Some(self.solvers.remove(last))
        }
    }

    pub fn release(&mut self, solver: PdrSolver) {
        self.solvers.push(solver)
    }

    pub fn add_clause(&mut self, clause: &Clause) {
        assert!(self.num_solver == self.solvers.len());
        for solver in self.solvers.iter_mut() {
            solver.add_clause(clause)
        }
    }
}
