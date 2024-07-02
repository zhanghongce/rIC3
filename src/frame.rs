use crate::{proofoblig::ProofObligation, IC3};
use logic_form::{Cube, Lemma, LitSet};
use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
    rc::Rc,
};
use transys::Transys;

pub struct Frame {
    lemmas: Vec<(Lemma, Option<ProofObligation>)>,
}

impl Frame {
    pub fn new() -> Self {
        Self { lemmas: Vec::new() }
    }
}

impl Deref for Frame {
    type Target = Vec<(Lemma, Option<ProofObligation>)>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.lemmas
    }
}

impl DerefMut for Frame {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lemmas
    }
}

#[derive(Clone)]
pub struct Frames {
    frames: Rc<Vec<Frame>>,
    pub early: usize,
    pub tmp_lit_set: Rc<LitSet>,
}

impl Frames {
    pub fn new(ts: &Rc<Transys>) -> Self {
        let mut tmp_lit_set = LitSet::new();
        tmp_lit_set.reserve(ts.max_latch);
        Self {
            frames: Default::default(),
            early: 1,
            tmp_lit_set: Rc::new(tmp_lit_set),
        }
    }

    #[inline]
    pub fn trivial_contained<'a>(
        &'a mut self,
        frame: usize,
        lemma: &logic_form::Lemma,
    ) -> Option<(usize, &'a mut Option<ProofObligation>)> {
        let tmp_lit_set = unsafe { Rc::get_mut_unchecked(&mut self.tmp_lit_set) };
        let frames = unsafe { Rc::get_mut_unchecked(&mut self.frames) };
        for l in lemma.iter() {
            tmp_lit_set.insert(*l);
        }
        for i in frame..frames.len() {
            for j in 0..frames[i].len() {
                if frames[i][j].0.subsume_set(lemma, tmp_lit_set) {
                    tmp_lit_set.clear();
                    return Some((i, &mut frames[i][j].1));
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
        for (c, _) in self.frames[frame - 1].iter() {
            if c.subsume(lemma) {
                return Some(c.clone());
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
        for (c, _) in self.frames[frame - 1].iter() {
            if c.subsume(lemma) {
                res.push(c.clone());
            }
        }
        res
    }

    #[inline]
    pub fn statistic(&self) {
        for f in self.frames.iter() {
            print!("{} ", f.len());
        }
        println!();
    }
}

impl Deref for Frames {
    type Target = Vec<Frame>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.frames
    }
}

impl DerefMut for Frames {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl Frames {
    #[inline]
    pub fn get_mut(&mut self) -> &mut Vec<Frame> {
        unsafe { Rc::get_mut_unchecked(&mut self.frames) }
    }
}

impl IC3 {
    #[inline]
    pub fn add_lemma(
        &mut self,
        frame: usize,
        lemma: Cube,
        subsume_check: bool,
        po: Option<ProofObligation>,
    ) -> bool {
        let lemma = logic_form::Lemma::new(lemma);
        if frame == 0 {
            assert!(self.frame.len() == 1);
            self.solvers[0].add_lemma(&!lemma.cube());
            self.frame[0].push((lemma, po));
            return false;
        }
        if subsume_check && self.frame.trivial_contained(frame, &lemma).is_some() {
            return false;
        }
        assert!(!self.ts.cube_subsume_init(lemma.cube()));
        let mut begin = None;
        let mut inv_found = false;
        'fl: for i in (1..=frame).rev() {
            let mut j = 0;
            while j < self.frame[i].len() {
                let (l, _) = &self.frame[i][j];
                if begin.is_none() && l.subsume(&lemma) {
                    if l.eq(&lemma) {
                        self.frame[i].swap_remove(j);
                        let clause = !lemma.cube();
                        for k in i + 1..=frame {
                            self.solvers[k].add_lemma(&clause);
                        }
                        self.frame[frame].push((lemma, po));
                        self.frame.early = self.frame.early.min(i + 1);
                        return self.frame[i].is_empty();
                    } else {
                        begin = Some(i + 1);
                        break 'fl;
                    }
                }
                if lemma.subsume(l) {
                    let (remove, _) = self.frame[i].swap_remove(j);
                    self.solvers[i].remove_lemma(&remove);
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
            self.solvers[i].add_lemma(&clause);
        }
        self.frame[frame].push((lemma, po));
        self.frame.early = self.frame.early.min(begin);
        inv_found
    }

    pub fn remove_lemma(&mut self, frame: usize, lemmas: Vec<Cube>) {
        let lemmas: HashSet<Lemma> = HashSet::from_iter(lemmas.into_iter().map(|l| Lemma::new(l)));
        for i in (1..=frame).rev() {
            let mut j = 0;
            while j < self.frame[i].len() {
                if let Some(po) = &mut self.frame[i][j].1 {
                    po.removed = true;
                }
                if lemmas.contains(&self.frame[i][j].0) {
                    for s in self.solvers[..=frame].iter_mut() {
                        s.remove_lemma(&self.frame[i][j].0);
                    }
                    self.frame[i].swap_remove(j);
                } else {
                    j += 1;
                }
            }
        }
    }
}
