use btor2::Btor2;
use logic_form::fol::{Term, TermCube, TermType};
use std::{collections::HashMap, ops::Deref};

#[derive(Clone, Debug)]
pub struct Transys {
    pub input: Vec<Term>,
    pub latch: Vec<Term>,
    pub latch_init: HashMap<Term, Term>,
    pub latch_next: HashMap<Term, Term>,
    pub init: Term,
    pub bad: Term,
}

impl Transys {
    pub fn new(btor2: Btor2) -> Self {
        let mut init = Term::bool_const(true);
        for l in btor2.latch.iter() {
            if let Some(i) = btor2.init.get(l) {
                init = init.and(&l.equal(i));
            }
        }
        Self {
            input: btor2.input,
            latch: btor2.latch,
            latch_init: btor2.init,
            latch_next: btor2.next,
            init,
            bad: btor2.bad,
        }
    }

    pub fn term_next(&self, term: &Term) -> Term {
        match term.deref() {
            TermType::Const(_) => term.clone(),
            TermType::Var(_) => self.latch_next[term].clone(),
            TermType::UniOp(op) => {
                let a = self.term_next(&op.a);
                a.uniop(op.ty)
            }
            TermType::BiOp(op) => {
                let a = self.term_next(&op.a);
                let b = self.term_next(&op.b);
                a.biop(&b, op.ty)
            }
            TermType::TriOp(_) => todo!(),
            TermType::ExtOp(op) => {
                let a = self.term_next(&op.a);
                a.extop(op.ty, op.length)
            }
            TermType::SliceOp(op) => {
                let a = self.term_next(&op.a);
                a.slice(op.upper, op.lower)
            }
        }
    }

    pub fn term_cube_next(&self, cube: &[Term]) -> TermCube {
        let mut next = TermCube::new();
        for l in cube.iter() {
            next.push(self.term_next(l));
        }
        next
    }
}
