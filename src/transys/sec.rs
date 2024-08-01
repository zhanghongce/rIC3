use crate::transys::Transys;
use logic_form::Lit;
use satif::Satif;

impl Transys {
    pub fn sec(&self) -> Vec<(Lit, Lit)> {
        let mut eqs = Vec::new();
        let simulations = self.simulations();
        let Some(simulations) = self.simulation_bv(simulations) else {
            return eqs;
        };
        let mut solver = cadical::Solver::new();
        self.load_trans(&mut solver);
        for i in 0..self.latchs.len() {
            for j in i + 1..self.latchs.len() {
                let x = self.latchs[i];
                let y = self.latchs[j];
                if self.init_map[x].is_some()
                    && self.init_map[x] == self.init_map[y]
                    && simulations[&x] == simulations[&y]
                {
                    let act = solver.new_var().lit();
                    let xl = x.lit();
                    let yl = y.lit();
                    solver.add_clause(&[!act, xl, !yl]);
                    solver.add_clause(&[!act, !xl, yl]);
                    let nxl = self.lit_next(xl);
                    let nyl = self.lit_next(yl);
                    if !solver.solve(&[act, nxl, !nyl]) && !solver.solve(&[act, !nxl, nyl]) {
                        eqs.push((xl, yl));
                        dbg!(x, y);
                        solver.add_clause(&[act]);
                        break;
                    } else {
                        solver.add_clause(&[!act]);
                    }
                }
            }
        }
        eqs
    }
}
