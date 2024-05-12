use crate::IC3;
use logic_form::Lemma;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::ops::Deref;
use std::rc::Rc;

#[derive(PartialEq, Eq, Debug)]
pub struct ProofObligationInner {
    pub frame: usize,
    pub lemma: Lemma,
    pub depth: usize,
    pub next: Option<ProofObligation>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ProofObligation {
    inner: Rc<ProofObligationInner>,
}

impl ProofObligation {
    pub fn new(frame: usize, lemma: Lemma, depth: usize, next: Option<Self>) -> Self {
        Self {
            inner: Rc::new(ProofObligationInner {
                frame,
                lemma,
                depth,
                next,
            }),
        }
    }

    pub fn set_frame(&mut self, frame: usize) {
        let inner = unsafe { Rc::get_mut_unchecked(&mut self.inner) };
        inner.frame = frame;
    }
}

impl Deref for ProofObligation {
    type Target = ProofObligationInner;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PartialOrd for ProofObligation {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProofObligation {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match other.frame.cmp(&self.frame) {
            Ordering::Equal => match self.depth.cmp(&other.depth) {
                Ordering::Equal => match other.lemma.len().cmp(&self.lemma.len()) {
                    Ordering::Equal => other.lemma.cmp(&self.lemma),
                    ord => ord,
                },
                ord => ord,
            },
            ord => ord,
        }
    }
}

#[derive(Default, Debug)]
pub struct ProofObligationQueue {
    obligations: BTreeSet<ProofObligation>,
    num: Vec<usize>,
}

impl ProofObligationQueue {
    pub fn new() -> Self {
        Self::default()
    }

    fn add(&mut self, po: ProofObligation) {
        if self.num.len() <= po.frame {
            self.num.resize(po.frame + 1, 0);
        }
        self.num[po.frame] += 1;
        assert!(self.obligations.insert(po));
    }

    pub fn pop(&mut self, depth: usize) -> Option<ProofObligation> {
        if let Some(po) = self.obligations.last().filter(|po| po.frame <= depth) {
            self.num[po.frame] -= 1;
            self.obligations.pop_last()
        } else {
            None
        }
    }

    pub fn statistic(&self) {
        println!("{:?}", self.num);
    }
}

impl IC3 {
    pub fn add_obligation(&mut self, po: ProofObligation) {
        self.statistic.avg_po_cube_len += po.lemma.len();
        self.obligations.add(po)
    }
}
