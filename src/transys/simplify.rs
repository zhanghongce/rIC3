use super::{abc::abc_preprocess, Transys};
use aig::{Aig, AigEdge};
use logic_form::{Clause, Lit, LitMap, Var, VarMap};
use minisat::SimpSolver;
use satif::Satif;
use std::collections::{HashMap, HashSet};

impl Transys {
    pub fn preprocess(aig: &Aig) -> (Aig, HashMap<usize, usize>) {
        // let mut remap = HashMap::new();
        // for l in aig.inputs.iter() {
        //     remap.insert(*l, *l);
        // }
        // for l in aig.latchs.iter() {
        //     remap.insert(l.input, l.input);
        // }
        // (aig.clone(), remap)
        let (mut aig, mut remap) = aig.coi_refine();
        let mut remap_retain = HashSet::new();
        remap_retain.insert(AigEdge::constant_edge(false).node_id());
        for i in aig.inputs.iter() {
            remap_retain.insert(*i);
        }
        for l in aig.latchs.iter() {
            remap_retain.insert(l.input);
        }
        remap.retain(|x, _| remap_retain.contains(x));
        // let mut aig = abc_preprocess(aig);
        aig.constraints
            .retain(|e| *e != AigEdge::constant_edge(true));
        (aig, remap)
    }

    fn compress_deps_rec(
        v: Var,
        deps: &mut VarMap<Vec<Var>>,
        domain: &HashSet<Var>,
        compressed: &mut HashSet<Var>,
    ) {
        if compressed.contains(&v) {
            return;
        }
        for d in 0..deps[v].len() {
            Self::compress_deps_rec(deps[v][d], deps, domain, compressed);
        }
        let mut dep = HashSet::new();
        for d in deps[v].iter() {
            if domain.contains(d) {
                dep.insert(*d);
                continue;
            }
            for dd in deps[*d].iter() {
                dep.insert(*dd);
            }
        }
        deps[v] = dep.into_iter().collect();
        deps[v].sort();
        compressed.insert(v);
    }

    fn compress_deps(mut deps: VarMap<Vec<Var>>, domain: &HashSet<Var>) -> VarMap<Vec<Var>> {
        let mut compressed: HashSet<Var> = HashSet::new();
        for v in 0..deps.len() {
            let v = Var::new(v);
            Self::compress_deps_rec(v, &mut deps, domain, &mut compressed)
        }
        for v in 0..deps.len() {
            let v = Var::new(v);
            if !domain.contains(&v) {
                deps[v].clear();
            }
        }
        deps
    }

    pub fn simplify(&self, lemmas: &[Clause], keep_dep: bool, assert_constrain: bool) -> Self {
        let mut simp_solver: Box<dyn Satif> = if keep_dep {
            Box::new(SimpSolver::new())
        } else {
            Box::new(cadical::Solver::new())
        };
        let false_lit: Lit = simp_solver.new_var().into();
        simp_solver.new_var_to(self.max_var);
        for c in self.trans.iter().chain(lemmas.iter()) {
            simp_solver.add_clause(c);
        }
        for i in self.inputs.iter() {
            simp_solver.set_frozen(*i, true)
        }
        for l in self.latchs.iter() {
            simp_solver.set_frozen(*l, true);
            simp_solver.set_frozen(self.var_next(*l), true)
        }
        for b in self.bad.iter() {
            simp_solver.set_frozen(b.var(), true);
        }
        for c in self.constraints.iter() {
            if assert_constrain {
                simp_solver.add_clause(&[*c]);
            } else {
                simp_solver.set_frozen(c.var(), true);
            }
        }
        simp_solver.simplify();
        let mut trans = simp_solver.clauses();
        trans.push(Clause::from([!false_lit]));
        let mut max_var = false_lit.var();

        let mut domain = HashSet::new();
        for cls in trans.iter() {
            for l in cls.iter() {
                max_var = max_var.max(l.var());
                domain.insert(l.var());
            }
        }
        for l in self.latchs.iter().chain(self.inputs.iter()) {
            domain.insert(*l);
        }
        let dep = Self::compress_deps(self.dependence.clone(), &domain);
        let mut domain = Vec::from_iter(domain);
        domain.sort();
        let mut domain_map = HashMap::new();
        for (i, d) in domain.iter().enumerate() {
            domain_map.insert(*d, Var::new(i));
        }
        let map_lit = |l: &Lit| Lit::new(domain_map[&l.var()], l.polarity());
        let inputs = self.inputs.iter().map(|v| domain_map[&v]).collect();
        let latchs: Vec<Var> = self.latchs.iter().map(|v| domain_map[&v]).collect();
        let init = self.init.iter().map(map_lit).collect();
        let bad = self.bad.iter().map(map_lit).collect();
        let max_latch = domain_map[&self.max_latch];
        let mut init_map: VarMap<Option<bool>> = VarMap::new_with(max_latch);
        for l in self.latchs.iter() {
            init_map[domain_map[l]] = self.init_map[*l];
        }
        let constraints = if assert_constrain {
            Default::default()
        } else {
            self.constraints.iter().map(map_lit).collect()
        };
        for c in trans.iter_mut() {
            *c = c.iter().map(|l| map_lit(l)).collect();
        }
        let max_var = domain_map[&max_var];
        let mut next_map = LitMap::new_with(max_var);
        let mut prev_map = LitMap::new_with(max_var);
        for l in self.latchs.iter() {
            let l = l.lit();
            let p = self.lit_next(l);
            let l = map_lit(&l);
            let p = map_lit(&p);
            next_map[l] = p;
            next_map[!l] = !p;
            prev_map[p] = l;
            prev_map[!p] = !l;
        }
        let mut dependence = VarMap::new();
        for d in domain.iter() {
            let dep = dep[*d].clone();
            let dep: Vec<Var> = dep.into_iter().map(|v| domain_map[&v]).collect();
            dependence.push(dep);
        }
        let mut is_latch = VarMap::new_with(max_var);
        for l in latchs.iter() {
            is_latch[*l] = true;
        }
        let mut restore = HashMap::new();
        for d in domain.iter() {
            if let Some(r) = self.restore.get(d) {
                restore.insert(domain_map[d], *r);
            }
        }
        Self {
            inputs,
            latchs,
            init,
            bad,
            init_map,
            constraints,
            trans,
            max_var,
            is_latch,
            next_map,
            prev_map,
            dependence,
            max_latch,
            restore,
        }
    }
}
