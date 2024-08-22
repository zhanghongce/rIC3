use aig::Aig;
use clap::Parser;
use rIC3::{
    bmc::BMC,
    general,
    kind::Kind,
    portfolio::Portfolio,
    transys::Transys,
    verify::{check_certifaiger, check_witness},
    Engine, Options, IC3,
};
use std::{mem, process::exit};

fn main() {
    procspawn::init();
    let option = Options::parse();
    let verbose = option.verbose;
    if verbose > 0 {
        println!("the model to be checked: {}", option.model);
    }
    let aig = Aig::from_file(&option.model);
    let mut engine: Box<dyn Engine> = if option.portfolio {
        Box::new(Portfolio::new(option.clone()))
    } else {
        if aig.bads.len() + aig.outputs.len() == 0 {
            panic!("no property to be checked");
        }
        let mut ts = Transys::from_aig(&aig);
        let pre_lemmas = vec![];
        if option.preprocess.sec {
            panic!("sec not support");
        }
        ts = ts.simplify(&[], option.ic3, !option.ic3);
        if option.bmc {
            Box::new(BMC::new(option.clone(), ts))
        } else if option.kind {
            Box::new(Kind::new(option.clone(), ts, pre_lemmas))
        } else if option.gic3 {
            Box::new(general::IC3::new(option.clone(), ts))
        } else {
            Box::new(IC3::new(option.clone(), ts, pre_lemmas))
        }
    };
    let res = engine.check();
    match res {
        Some(true) => check_certifaiger(&mut engine, &aig, &option),
        Some(false) => check_witness(&mut engine, &aig, &option),
        _ => (),
    }
    mem::forget(engine);
    if let Some(res) = res {
        if verbose > 0 {
            println!("result: {res}");
        }
        exit(if res { 20 } else { 10 });
    } else {
        if verbose > 0 {
            println!("result: unknown");
        }
        exit(0)
    }
}
