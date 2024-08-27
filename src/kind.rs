use crate::{
    transys::{unroll::TransysUnroll, Transys},
    verify::witness_encode,
    Engine, Options,
};
use aig::{Aig, AigEdge};
use logic_form::{Clause, Cube};
use satif::Satif;

pub struct Kind {
    uts: TransysUnroll,
    options: Options,
    solver: cadical::Solver,
    pre_lemmas: Vec<Clause>,
}

impl Kind {
    pub fn new(options: Options, ts: Transys, pre_lemmas: Vec<Clause>) -> Self {
        let uts = TransysUnroll::new(&ts);
        let solver = cadical::Solver::new();
        Self {
            uts,
            options,
            pre_lemmas,
            solver,
        }
    }

    fn load_pre_lemmas(&mut self, k: usize) {
        for cls in self.pre_lemmas.iter() {
            let cls: Clause = self.uts.lits_next(cls, k);
            self.solver.add_clause(&cls);
        }
    }

    // pub fn check_in_depth(&mut self, depth: usize) -> bool {
    //     println!("{}", self.options.model);
    //     assert!(depth > 0);
    //     let mut solver = kissat::Solver::new();
    //     self.uts.unroll_to(depth);
    //     for k in 0..=depth {
    //         self.uts.load_trans(&mut solver, k, true);
    //     }
    //     for k in 0..depth {
    //         solver.add_clause(&!self.uts.lits_next(&self.uts.ts.bad, k));
    //         self.load_pre_lemmas(&mut solver, k);
    //     }
    //     for b in self.uts.lits_next(&self.uts.ts.bad, depth).iter() {
    //         solver.add_clause(&[*b]);
    //     }
    //     println!("kind depth: {depth}");
    //     if !solver.solve(&[]) {
    //         println!("kind proofed in depth {depth}");
    //         return true;
    //     }
    //     false
    // }
}

impl Engine for Kind {
    fn check(&mut self) -> Option<bool> {
        let step = self.options.step as usize;
        for k in (step - 1..).step_by(step) {
            self.uts.unroll_to(k);
            let kind_bound = k + 1 - step;
            self.uts.load_trans(&mut self.solver, kind_bound, true);
            self.load_pre_lemmas(kind_bound);
            if kind_bound > 0 {
                if self.options.verbose > 0 {
                    println!("kind depth: {kind_bound}");
                }
                if !self
                    .solver
                    .solve(&self.uts.lits_next(&self.uts.ts.bad, kind_bound))
                {
                    println!("k-induction proofed in depth {kind_bound}");
                    return Some(true);
                }
            }
            for s in kind_bound + 1..=k {
                self.uts.load_trans(&mut self.solver, s, true);
                self.load_pre_lemmas(s);
            }
            if !self.options.kind_options.no_bmc {
                let mut assump = self.uts.ts.init.clone();
                assump.extend_from_slice(&self.uts.lits_next(&self.uts.ts.bad, k));
                if self.options.verbose > 0 {
                    println!("kind bmc depth: {k}");
                }
                if self.solver.solve(&assump) {
                    if self.options.verbose > 0 {
                        println!("bmc found cex in depth {k}");
                    }
                    return Some(false);
                }
            }
            for s in k + 1 - step..=k {
                self.solver
                    .add_clause(&!self.uts.lits_next(&self.uts.ts.bad, s));
            }
        }
        unreachable!();
    }

    fn certifaiger(&mut self, aig: &Aig) -> Aig {
        let mut certifaiger = aig.clone();
        let ni = aig.inputs.len();
        let nl = aig.latchs.len();
        let nc = aig.constraints.len();
        let k = self.uts.num_unroll;
        for _ in 1..k {
            certifaiger.merge(aig);
        }
        let inputs = certifaiger.inputs.clone();
        certifaiger.inputs.truncate(ni);
        let latchs = certifaiger.latchs.clone();
        certifaiger.latchs.truncate(nl);
        let mut bads: Vec<AigEdge> = certifaiger
            .bads
            .iter()
            .chain(certifaiger.outputs.iter())
            .copied()
            .collect();
        certifaiger.bads.clear();
        certifaiger.outputs.clear();
        let mut constrains = Vec::new();
        for i in 0..k {
            let c = certifaiger.constraints[i * nc..(i + 1) * nc].to_vec();
            constrains.push(certifaiger.new_ands_node(c.into_iter()));
        }
        certifaiger.constraints.truncate(nc);
        for i in 0..k {
            bads[i] = certifaiger.new_or_node(bads[i], !constrains[i]);
        }
        let sum = inputs.len() + latchs.len();
        let mut aux_latchs: Vec<AigEdge> = Vec::new();
        for i in 0..k {
            let input = certifaiger.new_leaf_node();
            aux_latchs.push(input.into());
            let (next, init) = if i == 0 {
                (input.into(), Some(true))
            } else {
                (aux_latchs[i - 1], Some(false))
            };
            certifaiger.new_latch(input, next, init);
        }
        for i in 1..k {
            for j in 0..ni {
                certifaiger.new_latch(inputs[j + i * ni], inputs[j + (i - 1) * ni].into(), None);
            }
            for j in 0..nl {
                certifaiger.new_latch(
                    latchs[j + i * nl].input,
                    latchs[j + (i - 1) * nl].input.into(),
                    None,
                );
            }
        }
        for i in 0..k {
            let al = aux_latchs[i];
            let p = certifaiger.new_imply_node(al, !bads[i]);
            certifaiger.bads.push(!p);
        }
        for i in 1..k {
            let al = aux_latchs[i];
            let al_next = aux_latchs[i - 1];
            let p = certifaiger.new_imply_node(al, al_next);
            certifaiger.bads.push(!p);
            let mut eqs = Vec::new();
            let mut init = Vec::new();
            for j in 0..nl {
                if let Some(linit) = latchs[j].init {
                    init.push(AigEdge::new(latchs[(i - 1) * nl + j].input, !linit))
                }
                eqs.push(certifaiger.new_eq_node(
                    latchs[j + i * nl].next,
                    latchs[j + (i - 1) * nl].input.into(),
                ));
            }
            let p = certifaiger.new_ands_node(eqs.into_iter());
            let p = certifaiger.new_imply_node(al, p);
            certifaiger.bads.push(!p);
            let init = certifaiger.new_ands_node(init.into_iter());
            let p = certifaiger.new_and_node(!al, al_next);
            let p = certifaiger.new_imply_node(p, init);
            certifaiger.bads.push(!p);
        }
        certifaiger.bads.push(!aux_latchs[0]);
        let bads: Vec<AigEdge> = certifaiger
            .bads
            .iter()
            .chain(certifaiger.outputs.iter())
            .copied()
            .collect();
        let bads = certifaiger.new_ors_node(bads.into_iter());
        certifaiger.bads.clear();
        certifaiger.outputs.clear();
        certifaiger.outputs.push(bads);
        assert!(certifaiger.inputs.len() + certifaiger.latchs.len() == sum + k);
        certifaiger
    }

    fn witness(&mut self, aig: &Aig) -> String {
        let mut wit = vec![Cube::new()];
        for l in self.uts.ts.latchs.iter() {
            let l = l.lit();
            if let Some(v) = self.solver.sat_value(l) {
                wit[0].push(self.uts.ts.restore(l.not_if(!v)));
            }
        }
        for k in 0..=self.uts.num_unroll {
            let mut w = Cube::new();
            for l in self.uts.ts.inputs.iter() {
                let l = l.lit();
                let kl = self.uts.lit_next(l, k);
                if let Some(v) = self.solver.sat_value(kl) {
                    w.push(self.uts.ts.restore(l.not_if(!v)));
                }
            }
            wit.push(w);
        }
        witness_encode(aig, &wit)
    }
}
