use super::{
    cdb::{CRef, CREF_NONE},
    Solver,
};
use giputils::gvec::Gvec;
use logic_form::{Cube, Lemma, LitMap, Var};
use std::mem::take;

#[derive(Default)]
pub struct Simplify {
    pub last_num_assign: u32,
    pub last_simplify: usize,
    pub lazy_remove: Vec<Cube>,
    pub last_num_lemma: u32,
}

impl Solver {
    pub fn simplify(&mut self) {
        assert!(self.highest_level() == 0);
        assert!(self.propagate() == CREF_NONE);
        if self.statistic.num_solve > self.simplify.last_simplify + 1000 {
            if self.simplify.last_num_assign < self.trail.len() {
                self.simplify_satisfied();
                self.simplify.last_simplify = self.statistic.num_solve;
            }
            if self.simplify.last_num_lemma + 1000 < self.cdb.lemmas.len() {
                let lemmas = take(&mut self.cdb.lemmas);
                self.cdb.lemmas = self.subsume_simplify(lemmas);
                self.simplify.last_num_lemma = self.cdb.lemmas.len();
            }
            self.garbage_collect();
        }
    }

    fn subsume_simplify(&mut self, clauses: Gvec<CRef>) -> Gvec<CRef> {
        let mut clauses: Vec<(CRef, Lemma)> = clauses
            .into_iter()
            .map(|cref| {
                let cls = self.cdb.get(cref);
                let lemma = Lemma::new(Cube::from(cls.slice()));
                (cref, lemma)
            })
            .collect();
        clauses.sort_by_key(|(_, l)| l.len());
        let mut occurs: LitMap<Vec<usize>> = LitMap::new_with(Var::new(self.ts.num_var));
        for i in 0..clauses.len() {
            for l in clauses[i].1.iter() {
                occurs[*l].push(i);
            }
        }
        for cls_idx in 0..clauses.len() {
            let cls = self.cdb.get(clauses[cls_idx].0);
            if cls.is_removed() {
                continue;
            }
            let lemma = &clauses[cls_idx].1;
            let max_occurs = *lemma.iter().min_by_key(|l| occurs[**l].len()).unwrap();
            for subsumed in occurs[max_occurs].iter() {
                if *subsumed <= cls_idx {
                    continue;
                }
                if self.cdb.get(clauses[*subsumed].0).is_removed() {
                    continue;
                }
                if lemma.subsume(&clauses[*subsumed].1) {
                    self.remove_clause(clauses[*subsumed].0);
                    self.statistic.num_simplify_subsume += 1;
                }
            }
        }
        clauses
            .into_iter()
            .map(|(cref, _)| cref)
            .filter(|cref| !self.cdb.get(*cref).is_removed())
            .collect()
    }
}
