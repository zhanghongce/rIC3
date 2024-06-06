use super::search::Value;
use logic_form::{Var, VarSet};
use std::{collections::HashSet, rc::Rc, slice};
use transys::Transys;

pub struct Domain {
    pub domain: VarSet,
    pub fixed: u32,
}

impl Domain {
    pub fn new() -> Self {
        Self {
            domain: Default::default(),
            fixed: 0,
        }
    }

    pub fn reserve(&mut self, var: Var) {
        self.domain.reserve(var);
    }

    pub fn calculate_constrain(&mut self, ts: &Rc<Transys>, value: &Value) {
        let mut marked = HashSet::new();
        let mut queue = Vec::new();
        for c in ts.constraints.iter() {
            if !marked.contains(&c.var()) {
                marked.insert(c.var());
                queue.push(c.var());
            }
        }
        while let Some(v) = queue.pop() {
            for d in ts.dependence[v].iter() {
                if !marked.contains(d) {
                    marked.insert(*d);
                    queue.push(*d);
                }
            }
        }
        let mut marked = Vec::from_iter(marked);
        marked.sort();
        for v in marked.iter() {
            if value.v(v.lit()).is_none() {
                self.domain.insert(*v);
            }
        }
        self.fixed = self.domain.len();
    }

    #[inline]
    pub fn reset(&mut self) {
        while self.domain.len() > self.fixed {
            let v = self.domain.set.pop().unwrap();
            self.domain.has[v] = false;
        }
    }

    #[inline]
    pub fn add_domain(&mut self, var: Var) {
        self.reset();
        self.domain.insert(var);
        self.fixed = self.domain.len();
    }

    fn get_coi(&mut self, root: impl Iterator<Item = Var>, ts: &Rc<Transys>, value: &Value) {
        for r in root {
            if value.v(r.lit()).is_none() {
                self.domain.insert(r);
            }
        }
        let mut now = self.fixed;
        while now < self.domain.len() {
            let v = self.domain[now];
            now += 1;
            for d in ts.dependence[v].iter() {
                if value.v(d.lit()).is_none() {
                    self.domain.insert(*d);
                }
            }
        }
    }

    pub fn enable_local(
        &mut self,
        domain: impl Iterator<Item = Var>,
        ts: &Rc<Transys>,
        value: &Value,
    ) {
        self.reset();
        self.get_coi(domain, ts, value);
    }

    #[inline]
    pub fn has(&self, var: Var) -> bool {
        self.domain.has(var)
    }

    pub fn domains(&self) -> slice::Iter<Var> {
        self.domain.iter()
    }
}
