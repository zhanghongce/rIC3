use std::{sync::mpsc::channel, thread::spawn};

use crate::{bmc::BMC, Args, IC3};

pub struct Portfolio {
    args: Args,
}

impl Portfolio {
    pub fn new(args: Args) -> Self {
        Self { args }
    }

    pub fn check(&mut self) -> bool {
        println!("{}", self.args.model);
        let (tx, rx) = channel::<(bool, String)>();
        let t = tx.clone();
        let args = self.args.clone();
        spawn(move || {
            let mut bmc = BMC::new(args);
            let _ = t.send((!bmc.check_no_incremental(), "bmc".to_string()));
        });
        let t = tx.clone();
        let args = self.args.clone();
        spawn(move || {
            let mut ic3 = IC3::new(args);
            let _ = t.send((ic3.check(), "ic3".to_string()));
        });
        let (res, engine) = rx.recv().unwrap();
        println!("best configuration: {engine}");
        res
    }
}
