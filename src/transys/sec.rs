use crate::transys::Transys;
use logic_form::Clause;
use satif::Satif;

impl Transys {
    pub fn sec(&self) -> Vec<Clause> {
        let mut solver = cadical::Solver::new();
        self.load_trans(&mut solver);
        let mut lemmas = Vec::new();
        for i in 0..self.latchs.len() {
            for j in i + 1..self.latchs.len() {
                let x = self.latchs[i];
                let y = self.latchs[j];
                if self.init_map[x].is_some() && self.init_map[x] == self.init_map[y] {
                    let act = solver.new_var().lit();
                    let xl = x.lit();
                    let yl = y.lit();
                    solver.add_clause(&[!act, xl, !yl]);
                    solver.add_clause(&[!act, !xl, yl]);
                    let nxl = self.lit_next(xl);
                    let nyl = self.lit_next(yl);
                    if !solver.solve(&[act, nxl, !nyl]) && !solver.solve(&[act, !nxl, nyl]) {
                        lemmas.push(Clause::from([xl, !yl]));
                        lemmas.push(Clause::from([!xl, yl]));
                        dbg!(x, y);
                        solver.add_clause(&[act]);
                    } else {
                        solver.add_clause(&[!act]);
                    }
                }
            }
        }
        lemmas
    }
}
