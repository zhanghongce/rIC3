use aig::Aig;
use logic_form::{Clause, Cnf, Cube, Lit, Var, VarMap};
use minisat::SimpSolver;
use std::collections::{HashMap, HashSet};

pub struct Model {
    pub inputs: Vec<Var>,
    pub latchs: Vec<Var>,
    pub primes: Vec<Var>,
    pub init: Cube,
    pub bad: Cube,
    pub init_map: HashMap<Var, bool>,
    pub constraints: Vec<Lit>,
    pub trans: Cnf,
    pub num_var: usize,
    next_map: HashMap<Var, Var>,
    previous_map: HashMap<Var, Var>,
    pub dependence: VarMap<Vec<Var>>,
}

impl Model {
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

    fn compress_deps(mut deps: VarMap<Vec<Var>>, cnf: &Cnf) -> VarMap<Vec<Var>> {
        let mut domain = HashSet::new();
        for cls in cnf.iter() {
            for l in cls.iter() {
                domain.insert(l.var());
            }
        }
        let mut compressed: HashSet<Var> = HashSet::new();
        for v in 0..deps.len() {
            let v = Var::new(v);
            Self::compress_deps_rec(v, &mut deps, &domain, &mut compressed)
        }
        for v in 0..deps.len() {
            let v = Var::new(v);
            if !domain.contains(&v) {
                deps[v].clear();
            }
        }
        deps
    }

    pub fn from_aig(aig: &Aig) -> Self {
        let mut simp_solver = SimpSolver::new();
        let false_lit: Lit = simp_solver.new_var().into();
        let mut dependence = VarMap::new();
        dependence.push(vec![]);
        simp_solver.add_clause(&[!false_lit]);
        for node in aig.nodes.iter().skip(1) {
            assert_eq!(Var::new(node.node_id()), simp_solver.new_var());
            let mut dep = Vec::new();
            if node.is_and() {
                dep.push(node.fanin0().to_lit().var());
                dep.push(node.fanin1().to_lit().var());
            }
            dependence.push(dep);
        }
        let inputs: Vec<Var> = aig.inputs.iter().map(|x| Var::new(*x)).collect();
        let mut latchs: Vec<Var> = aig.latchs.iter().map(|x| Var::new(x.input)).collect();
        latchs.push(simp_solver.new_var());
        dependence.push(vec![]);
        let primes: Vec<Var> = latchs
            .iter()
            .map(|_| {
                dependence.push(vec![]);
                simp_solver.new_var()
            })
            .collect();
        let bad_var_lit = latchs.last().unwrap().lit();
        let bad_var_prime_lit = primes.last().unwrap().lit();
        let mut init = aig.latch_init_cube().to_cube();
        init.push(!bad_var_lit);
        let mut init_map = HashMap::new();
        for l in init.iter() {
            init_map.insert(l.var(), l.polarity());
        }
        let constraints: Vec<Lit> = aig.constraints.iter().map(|c| c.to_lit()).collect();
        assert!(constraints.is_empty());
        let aig_bad = if aig.bads.is_empty() {
            aig.outputs[0]
        } else {
            aig.bads[0]
        };
        for v in inputs.iter().chain(latchs.iter()).chain(primes.iter()) {
            simp_solver.set_frozen(*v, true);
        }
        for l in constraints.iter() {
            simp_solver.set_frozen(l.var(), true);
        }
        let mut logic = Vec::new();
        for l in aig.latchs.iter() {
            logic.push(l.next);
        }
        for c in aig.constraints.iter() {
            logic.push(*c);
        }
        logic.push(aig_bad);
        let mut trans = aig.get_cnf(&logic);
        let bad_lit = aig_bad.to_lit();
        trans.push(Clause::from([!bad_lit, bad_var_prime_lit]));
        trans.push(Clause::from([bad_lit, !bad_var_prime_lit]));
        dependence[bad_var_prime_lit].push(bad_lit.var());
        let bad = Cube::from([bad_var_prime_lit]);
        for tran in trans.iter() {
            simp_solver.add_clause(tran);
        }
        for (l, p) in aig.latchs.iter().zip(primes.iter()) {
            let l = l.next.to_lit();
            let p = p.lit();
            simp_solver.add_clause(&Clause::from([l, !p]));
            simp_solver.add_clause(&Clause::from([!l, p]));
            dependence[p].push(l.var())
        }
        for c in constraints.iter() {
            simp_solver.add_clause(&Clause::from([*c]));
        }
        simp_solver.eliminate(true);
        let num_var = simp_solver.num_var();
        let trans = simp_solver.clauses();
        let mut next_map = HashMap::new();
        let mut previous_map = HashMap::new();
        for (l, p) in latchs.iter().zip(primes.iter()) {
            next_map.insert(*l, *p);
            previous_map.insert(*p, *l);
        }
        dependence = Self::compress_deps(dependence, &trans);
        Self {
            inputs,
            latchs,
            primes,
            init,
            bad,
            init_map,
            constraints,
            trans,
            num_var,
            next_map,
            previous_map,
            dependence,
        }
    }

    #[inline]
    pub fn lit_previous(&self, lit: Lit) -> Lit {
        Lit::new(self.previous_map[&lit.var()], lit.polarity())
    }

    #[inline]
    pub fn lit_next(&self, lit: Lit) -> Lit {
        Lit::new(self.next_map[&lit.var()], lit.polarity())
    }

    pub fn cube_previous(&self, cube: &Cube) -> Cube {
        cube.iter().map(|l| self.lit_previous(*l)).collect()
    }

    pub fn cube_next(&self, cube: &Cube) -> Cube {
        cube.iter().map(|l| self.lit_next(*l)).collect()
    }

    pub fn cube_subsume_init(&self, x: &Cube) -> bool {
        for i in 0..x.len() {
            if let Some(init) = self.init_map.get(&x[i].var()) {
                if *init != x[i].polarity() {
                    return false;
                }
            }
        }
        true
    }

    pub fn load_trans(&self, solver: &mut gipsat::Solver) {
        while solver.num_var() < self.num_var {
            solver.new_var();
        }
        for cls in self.trans.iter() {
            solver.add_clause(cls)
        }
    }

    pub fn inits(&self) -> Vec<Cube> {
        self.init_map
            .iter()
            .map(|(latch, init)| Cube::from([Lit::new(*latch, !init)]))
            .collect()
    }
}
