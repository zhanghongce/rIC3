mod mic;
mod others;
mod solver;

use crate::{
    activity::Activity,
    frame::{Frame, Frames},
    options::Options,
    proofoblig::{ProofObligation, ProofObligationQueue},
    statistic::Statistic,
    transys::Transys,
    verify::witness_encode,
    Engine,
};
use aig::{Aig, AigEdge};
use logic_form::{Cube, Lemma};
use solver::{BlockResult, BlockResultYes, Ic3Solver, Lift};
use std::{rc::Rc, time::Instant};

pub struct IC3 {
    options: Options,
    ts: Rc<Transys>,
    frame: Frames,
    solvers: Vec<Ic3Solver>,
    lift: Lift,
    obligations: ProofObligationQueue,
    activity: Activity,
    statistic: Statistic,
}

impl IC3 {
    #[inline]
    pub fn level(&self) -> usize {
        self.solvers.len() - 1
    }

    fn extend(&mut self) {
        self.frame.push(Frame::new());
        self.solvers
            .push(Ic3Solver::new(&self.ts, self.solvers.len()));
        if self.level() == 0 {
            for init in self.ts.init.clone() {
                self.add_lemma(0, Cube::from([!init]), true, None);
            }
        }
    }

    fn push_lemma(&mut self, frame: usize, mut cube: Cube) -> (usize, Cube) {
        for i in frame + 1..=self.level() {
            if let BlockResult::Yes(block) = self.blocked(i, &cube, true) {
                cube = self.inductive_core(block);
            } else {
                return (i, cube);
            }
        }
        (self.level() + 1, cube)
    }

    fn generalize(&mut self, mut po: ProofObligation, blocked: BlockResultYes) -> bool {
        let mut mic = self.inductive_core(blocked);
        mic = self.mic(
            po.frame,
            mic,
            if self.options.ic3_options.ctg { 1 } else { 0 },
        );
        let (frame, mic) = self.push_lemma(po.frame, mic);
        self.statistic.avg_po_cube_len += po.lemma.len();
        po.frame = frame;
        self.add_obligation(po.clone());
        if self.add_lemma(frame - 1, mic.clone(), false, Some(po)) {
            return true;
        }
        false
    }

    fn block(&mut self) -> Option<bool> {
        while let Some(mut po) = self.obligations.pop(self.level()) {
            if po.removed {
                continue;
            }
            if po.frame == 0 {
                self.add_obligation(po);
                return Some(false);
            }
            assert!(!self.ts.cube_subsume_init(&po.lemma));
            if let Some((bf, _)) = self.frame.trivial_contained(po.frame, &po.lemma) {
                po.frame = bf + 1;
                self.add_obligation(po);
                continue;
            }
            if self.options.verbose > 1 {
                self.frame.statistic();
            }
            match self.blocked_with_ordered(po.frame, &po.lemma, false, true) {
                BlockResult::Yes(blocked) => {
                    if self.generalize(po, blocked) {
                        return None;
                    }
                }
                BlockResult::No(unblocked) => {
                    let (model, inputs) = self.get_predecessor(unblocked);
                    self.add_obligation(ProofObligation::new(
                        po.frame - 1,
                        Lemma::new(model),
                        inputs,
                        po.depth + 1,
                        Some(po.clone()),
                    ));
                    self.add_obligation(po);
                }
            }
        }
        Some(true)
    }

    #[allow(unused)]
    fn trivial_block(&mut self, frame: usize, lemma: Lemma, limit: &mut usize) -> bool {
        if frame == 0 {
            return false;
        }
        if self.ts.cube_subsume_init(&lemma) {
            return false;
        }
        if *limit == 0 {
            return false;
        }
        *limit -= 1;
        loop {
            match self.blocked_with_ordered(frame, &lemma, false, true) {
                BlockResult::Yes(blocked) => {
                    let mut mic = self.inductive_core(blocked);
                    mic = self.mic(frame, mic, 0);
                    let (frame, mic) = self.push_lemma(frame, mic);
                    self.add_lemma(frame - 1, mic, false, None);
                    return true;
                }
                BlockResult::No(ub) => {
                    let model = Lemma::new(self.get_predecessor(ub).0);
                    if !self.trivial_block(frame - 1, model, limit) {
                        return false;
                    }
                }
            }
        }
    }

    fn propagate(&mut self) -> bool {
        for frame_idx in self.frame.early..self.level() {
            self.frame[frame_idx].sort_by_key(|x| x.len());
            let frame = self.frame[frame_idx].clone();
            for lemma in frame {
                if self.frame[frame_idx].iter().all(|l| l.ne(&lemma)) {
                    continue;
                }
                match self.blocked(frame_idx + 1, &lemma, false) {
                    BlockResult::Yes(blocked) => {
                        let conflict = self.inductive_core(blocked);
                        self.add_lemma(frame_idx + 1, conflict, true, lemma.po);
                    }
                    BlockResult::No(_) => {}
                }
            }
            if self.frame[frame_idx].is_empty() {
                return true;
            }
        }
        self.frame.early = self.level();
        false
    }
}

impl IC3 {
    pub fn new(options: Options, mut ts: Transys) -> Self {
        if options.ic3_options.bwd {
            ts = ts.reverse();
        }
        let ts = Rc::new(ts);
        let statistic = Statistic::new(&options.model);
        let activity = Activity::new(&ts);
        let frame = Frames::new(&ts);
        let lift = Lift::new(&ts);
        let mut res = Self {
            options,
            ts,
            activity,
            solvers: Vec::new(),
            lift,
            statistic,
            obligations: ProofObligationQueue::new(),
            frame,
        };
        res.extend();
        res
    }
}

impl Engine for IC3 {
    fn check(&mut self) -> Option<bool> {
        loop {
            let start = Instant::now();
            loop {
                match self.block() {
                    Some(false) => {
                        self.statistic.overall_block_time += start.elapsed();
                        self.statistic();
                        return Some(false);
                    }
                    None => {
                        self.statistic.overall_block_time += start.elapsed();
                        self.statistic();
                        self.verify();
                        return Some(true);
                    }
                    _ => (),
                }
                self.statistic.num_get_bad += 1;
                if let Some((bad, inputs)) = self.get_bad() {
                    let bad = Lemma::new(bad);
                    self.add_obligation(ProofObligation::new(self.level(), bad, inputs, 0, None))
                } else {
                    break;
                }
            }
            let blocked_time = start.elapsed();
            if self.options.verbose > 1 {
                self.frame.statistic();
                println!(
                    "[{}:{}] frame: {}, time: {:?}",
                    file!(),
                    line!(),
                    self.level(),
                    blocked_time,
                );
            }
            self.statistic.overall_block_time += blocked_time;
            self.extend();
            let start = Instant::now();
            let propagate = self.propagate();
            self.statistic.overall_propagate_time += start.elapsed();
            if propagate {
                self.statistic();
                self.verify();
                return Some(true);
            }
        }
    }

    fn certifaiger(&mut self, aig: &Aig) -> Aig {
        let invariants = self.frame.invariant();
        let invariants = invariants
            .iter()
            .map(|l| Cube::from_iter(l.iter().map(|l| self.ts.restore(*l))));
        let mut certifaiger = aig.clone();
        let mut certifaiger_dnf = vec![];
        for cube in invariants {
            certifaiger_dnf
                .push(certifaiger.new_ands_node(cube.into_iter().map(AigEdge::from_lit)));
        }
        let invariants = certifaiger.new_ors_node(certifaiger_dnf.into_iter());
        let constrains: Vec<AigEdge> = certifaiger.constraints.iter().map(|e| !*e).collect();
        let constrains = certifaiger.new_ors_node(constrains.into_iter());
        let invariants = certifaiger.new_or_node(invariants, constrains);
        certifaiger.bads.clear();
        certifaiger.outputs.clear();
        certifaiger.outputs.push(invariants);
        certifaiger
    }

    fn witness(&mut self, aig: &Aig) -> String {
        let mut res: Vec<Cube> = Vec::new();
        let b = self.obligations.peak().unwrap();
        res.push(b.lemma.iter().map(|l| self.ts.restore(*l)).collect());
        let mut b = Some(b);
        while let Some(bad) = b {
            res.push(bad.input.iter().map(|l| self.ts.restore(*l)).collect());
            b = bad.next.clone();
        }
        witness_encode(aig, &res)
    }
}
