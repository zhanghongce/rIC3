use super::IC3;
use crate::{frame::FrameLemma, proofoblig::ProofObligation, verify::verify_invariant};
use logic_form::{Cube, Lemma};

impl IC3 {
    pub fn add_obligation(&mut self, po: ProofObligation) {
        self.statistic.avg_po_cube_len += po.lemma.len();
        self.obligations.add(po)
    }

    pub fn add_lemma(
        &mut self,
        frame: usize,
        lemma: Cube,
        contained_check: bool,
        po: Option<ProofObligation>,
    ) -> bool {
        let lemma = Lemma::new(lemma);
        if frame == 0 {
            assert!(self.frame.len() == 1);
            self.solvers[0].add_clause(&!lemma.cube());
            self.frame[0].push(FrameLemma::new(lemma, po, None));
            return false;
        }
        if contained_check && self.frame.trivial_contained(frame, &lemma).is_some() {
            return false;
        }
        assert!(!self.ts.cube_subsume_init(lemma.cube()));
        let mut begin = None;
        let mut inv_found = false;
        'fl: for i in (1..=frame).rev() {
            let mut j = 0;
            while j < self.frame[i].len() {
                let l = &self.frame[i][j];
                if begin.is_none() && l.subsume(&lemma) {
                    if l.eq(&lemma) {
                        self.frame[i].swap_remove(j);
                        let clause = !lemma.cube();
                        for k in i + 1..=frame {
                            self.solvers[k].add_clause(&clause);
                        }
                        self.frame[frame].push(FrameLemma::new(lemma, po, None));
                        self.frame.early = self.frame.early.min(i + 1);
                        return self.frame[i].is_empty();
                    } else {
                        begin = Some(i + 1);
                        break 'fl;
                    }
                }
                if lemma.subsume(l) {
                    let _remove = self.frame[i].swap_remove(j);
                    // self.solvers[i].remove_lemma(&remove);
                    continue;
                }
                j += 1;
            }
            if i != frame && self.frame[i].is_empty() {
                inv_found = true;
            }
        }
        let clause = !lemma.cube();
        let begin = begin.unwrap_or(1);
        for i in begin..=frame {
            self.solvers[i].add_clause(&clause);
        }
        self.frame[frame].push(FrameLemma::new(lemma, po, None));
        self.frame.early = self.frame.early.min(begin);
        inv_found
    }

    pub fn statistic(&mut self) {
        if self.options.verbose > 0 {
            self.obligations.statistic();
            for f in self.frame.iter() {
                print!("{} ", f.len());
            }
            println!();
            println!("{:#?}", self.statistic);
        }
    }

    pub fn verify(&mut self) {
        if !self.options.certify {
            return;
        }
        let invariants = self.frame.invariant();
        if !verify_invariant(&self.ts, &invariants) {
            panic!("invariant varify failed");
        }
        if self.options.verbose > 0 {
            println!(
                "inductive invariant verified with {} lemmas!",
                invariants.len()
            );
        }
    }
}
