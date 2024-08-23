use aig::Aig;
use clap::Parser;
use rIC3::{
    bmc::BMC,
    frontend::aig::aig_preprocess,
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
    let options = Options::parse();
    let verbose = options.verbose;
    if verbose > 0 {
        println!("the model to be checked: {}", options.model);
    }
    let aig = Aig::from_file(&options.model);
    if aig.bads.len() + aig.outputs.len() == 0 {
        panic!("no property to be checked");
    }
    let mut engine: Box<dyn Engine> = if options.portfolio {
        Box::new(Portfolio::new(options.clone()))
    } else {
        let (aig, restore) = aig_preprocess(&aig, &options);
        let mut ts = Transys::from_aig(&aig, &restore);
        let pre_lemmas = vec![];
        if options.preprocess.sec {
            panic!("sec not support");
        }
        ts = ts.simplify(&[], options.ic3, !options.ic3);
        if options.bmc {
            Box::new(BMC::new(options.clone(), ts))
        } else if options.kind {
            Box::new(Kind::new(options.clone(), ts, pre_lemmas))
        } else if options.gic3 {
            Box::new(general::IC3::new(options.clone(), ts))
        } else {
            Box::new(IC3::new(options.clone(), ts, pre_lemmas))
        }
    };
    let res = engine.check();
    match res {
        Some(true) => check_certifaiger(&mut engine, &aig, &options),
        Some(false) => check_witness(&mut engine, &aig, &options),
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
