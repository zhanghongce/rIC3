use super::Transys;
use logic_form::{Clause, Cube, Var, VarMap};

impl Transys {
    pub fn encode_bad_to_latch(&mut self) {
        assert!(self.bad.len() == 1);
        let bad = self.bad[0];
        if self.is_latch(bad.var()) {
            return;
        }
        let bad_latch = self.new_var().lit();
        let bad_next = self.new_var().lit();
        let trans = vec![
            Clause::from([bad, !bad_latch]),
            Clause::from([!bad, bad_latch]),
        ];
        self.add_latch(bad_latch.var(), bad_next, None, trans, vec![bad.var()]);
        self.bad = Cube::from([bad_latch]);
    }

    pub fn reverse(&self) -> Self {
        let mut res = self.clone();
        res.encode_bad_to_latch();
        let latchs: Vec<Var> = res
            .latchs
            .iter()
            .map(|v| res.lit_next(v.lit()).var())
            .collect();
        let bad = res.bad[0];
        let mut init_map = VarMap::new();
        init_map.reserve(Var::new(res.num_var));
        init_map[bad.var()] = Some(bad.polarity());
        let mut is_latch = VarMap::new();
        is_latch.reserve(Var::new(res.num_var));
        for l in latchs.iter() {
            is_latch[*l] = true;
        }
        Self {
            inputs: res.inputs,
            latchs,
            init: res.bad,
            bad: res.init,
            init_map,
            constraints: res.constraints,
            trans: res.trans,
            num_var: res.num_var,
            is_latch,
            next_map: res.prev_map,
            prev_map: res.next_map,
            dependence: res.dependence,
            max_latch: res.max_latch,
            latch_group: res.latch_group,
            groups: res.groups,
        }
    }
}
