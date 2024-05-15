use crate::{
    gipsat::{CRef, CREF_NONE},
    IC3,
};
use logic_form::{Cube, LitSet};
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};
use transys::Transys;

#[derive(Debug, Clone)]
pub struct Lemma {
    pub lemma: logic_form::Lemma,
    pub begin: usize,
    pub cref: Vec<CRef>,
}

impl Deref for Lemma {
    type Target = logic_form::Lemma;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.lemma
    }
}

impl Lemma {
    #[inline]
    pub fn get_cref(&self, id: usize) -> CRef {
        if id < self.begin {
            CREF_NONE
        } else {
            self.cref[id - self.begin]
        }
    }

    #[inline]
    pub fn set_cref(&mut self, id: usize, cref: CRef) {
        assert!(id >= self.begin);
        self.cref[id - self.begin] = cref
    }
}

#[derive(Clone)]
pub struct Frame {
    frames: Rc<Vec<Vec<Lemma>>>,
    pub early: usize,
    tmp_lit_set: Rc<LitSet>,
}

impl Frame {
    pub fn new(ts: &Rc<Transys>) -> Self {
        let mut tmp_lit_set = LitSet::new();
        tmp_lit_set.reserve(ts.max_latch);
        tmp_lit_set.reserve(ts.max_latch);
        Self {
            frames: Default::default(),
            early: 1,
            tmp_lit_set: Rc::new(tmp_lit_set),
        }
    }

    #[inline]
    pub fn trivial_contained(&mut self, frame: usize, lemma: &logic_form::Lemma) -> Option<usize> {
        let tmp_lit_set = unsafe { Rc::get_mut_unchecked(&mut self.tmp_lit_set) };
        for l in lemma.iter() {
            tmp_lit_set.insert(*l);
        }
        for i in frame..self.frames.len() {
            for l in self.frames[i].iter() {
                if l.subsume_set(lemma, tmp_lit_set) {
                    tmp_lit_set.clear();
                    return Some(i);
                }
            }
        }
        tmp_lit_set.clear();
        None
    }

    pub fn _parent_lemma(
        &self,
        lemma: &logic_form::Lemma,
        frame: usize,
    ) -> Option<logic_form::Lemma> {
        if frame == 1 {
            return None;
        }
        for c in self.frames[frame - 1].iter() {
            if c.subsume(lemma) {
                return Some(c.lemma.clone());
            }
        }
        None
    }

    pub fn _parent_lemmas(
        &self,
        lemma: &logic_form::Lemma,
        frame: usize,
    ) -> Vec<logic_form::Lemma> {
        let mut res = Vec::new();
        if frame == 1 {
            return res;
        }
        for c in self.frames[frame - 1].iter() {
            if c.subsume(&lemma) {
                res.push(c.lemma.clone());
            }
        }
        res
    }
}

impl Deref for Frame {
    type Target = Vec<Vec<Lemma>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.frames
    }
}

impl DerefMut for Frame {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl Frame {
    #[inline]
    pub fn get_mut(&mut self) -> &mut Vec<Vec<Lemma>> {
        unsafe { Rc::get_mut_unchecked(&mut self.frames) }
    }
}

impl IC3 {
    #[inline]
    pub fn add_lemma(&mut self, frame: usize, lemma: Cube) {
        let lemma = logic_form::Lemma::new(lemma);
        if frame == 0 {
            assert!(self.frame.len() == 1);
            assert!(self.gipsat.solvers[0].add_lemma(&!lemma.cube()) == CREF_NONE);
            self.frame[0].push(Lemma {
                lemma,
                cref: Vec::new(),
                begin: 1,
            });
            return;
        }
        if self.frame.trivial_contained(frame, &lemma).is_some() {
            return;
        }
        assert!(!self.ts.cube_subsume_init(lemma.cube()));
        let mut begin = None;
        'fl: for i in (1..=frame).rev() {
            let mut j = 0;
            while j < self.frame[i].len() {
                let l = &self.frame[i][j];
                if begin.is_none() && l.subsume(&lemma) {
                    if l.eq(&lemma) {
                        let mut eq_lemma = self.frame[i].swap_remove(j);
                        let clause = !lemma.cube();
                        for k in i + 1..=frame {
                            eq_lemma
                                .cref
                                .push(self.gipsat.solvers[k].add_lemma(&clause));
                        }
                        self.frame[frame].push(eq_lemma);
                        self.frame.early = self.frame.early.min(i + 1);
                        return;
                    } else {
                        begin = Some(i + 1);
                        break 'fl;
                    }
                }
                if lemma.subsume(l) {
                    for k in l.begin..=i {
                        let cref = l.get_cref(k);
                        if cref != CREF_NONE {
                            self.gipsat.solvers[k].remove_lemma(cref);
                        }
                    }
                    self.frame[i].swap_remove(j);
                    continue;
                }
                j += 1;
            }
        }
        let clause = !lemma.cube();
        let begin = begin.unwrap_or(1);
        let mut cref = Vec::new();
        for i in begin..=frame {
            cref.push(self.gipsat.solvers[i].add_lemma(&clause))
        }
        self.frame[frame].push(Lemma { lemma, cref, begin });
        self.frame.early = self.frame.early.min(begin);
    }
}
