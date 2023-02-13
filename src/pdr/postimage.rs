use crate::utils::state_transform::{aig_cube_next, aig_cube_previous};
use aig::{Aig, AigCube};
use logic_form::{Clause, Cnf, Cube, Dnf};
use sat_solver::{minisat, Model, SatResult, SatSolver};

pub struct ModelChecker {
    aig: Aig,
    transition_cnf: Cnf,
    generalize: minisat::Solver,
}

impl ModelChecker {
    fn generalize(&mut self, mut cube: Cube) -> Cube {
        let mut i = 0;
        while i < cube.len() {
            let removed = cube.swap_remove(i);
            if let SatResult::Unsat = self.generalize.solve(&cube, None) {
                continue;
            }
            cube.push(removed);
            let last_idx = cube.len() - 1;
            cube.swap(i, last_idx);
            i += 1;
        }
        cube
    }
}

impl ModelChecker {
    pub fn new(aig: Aig) -> Self {
        let mut generalize = minisat::Solver::new();
        let transition_cnf = aig.get_cnf();
        generalize.add_cnf(&transition_cnf);
        generalize.add_clause(&Clause::from([aig.bads[0].to_lit()]));
        Self {
            aig,
            transition_cnf,
            generalize,
        }
    }

    pub fn solve(&mut self) -> bool {
        let mut solver = sat_solver::minisat::Solver::new();
        let init = Cube::from_iter(self.aig.latch_init_cube().iter().map(|e| e.to_lit()));
        solver.add_cnf(&self.transition_cnf);
        let mut frontier = Dnf::new();
        solver.add_clause(&!aig_cube_next(&self.aig, &AigCube::from_cube(init.clone())).to_cube());
        frontier.push(init);
        let mut deep = 0;
        loop {
            dbg!(deep);
            deep += 1;
            let mut new_frontier = Dnf::new();
            for cube in frontier.iter() {
                while let sat_solver::SatResult::Sat(m) = solver.solve(&cube, None) {
                    let cube = AigCube::from_iter(
                        self.aig
                            .latchs
                            .iter()
                            .map(|l| l.next.not_if(!m.lit_value(l.next.to_lit()))),
                    );
                    let previous = aig_cube_previous(&self.aig, &cube);
                    let general = self.generalize(previous.to_cube());
                    dbg!(&general);
                    new_frontier.push(general.clone());
                    let blocking_clause =
                        !aig_cube_next(&self.aig, &AigCube::from_cube(general)).to_cube();
                    solver.add_clause(&blocking_clause);
                }
            }
            assert!(!new_frontier.is_empty());
            frontier = new_frontier;
        }
    }
}

pub fn solve(aig: Aig) -> bool {
    let mut mc = ModelChecker::new(aig);
    mc.solve()
}
