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
            Clause::from([bad_next, !bad]),
            Clause::from([!bad_next, bad]),
        ];
        self.latchs.push(bad_latch.var());
        self.init_map[bad_latch.var()] = None;
        self.is_latch[bad_latch.var()] = true;
        self.next_map[bad_latch] = bad_next;
        self.next_map[!bad_latch] = !bad_next;
        self.prev_map[bad_next] = bad_latch;
        self.prev_map[!bad_next] = !bad_latch;
        self.max_latch = self.max_latch.max(bad_latch.var());
        self.dependence[bad_next.var()] = vec![bad.var()];
        for t in trans {
            self.trans.push(t);
        }
        self.bad = Cube::from([bad_next]);
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
        let max_latch = *latchs.iter().max().unwrap();
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
            max_latch,
            latch_group: res.latch_group,
            groups: res.groups,
        }
    }
}
