use aig::Aig;
use clap::Parser;
use rIC3::{
    bmc::BMC, general, kind::Kind, portfolio::Portfolio, transys::Transys,
    verify::check_certifaiger, Engine, Options, IC3,
};
use std::process::exit;

fn main() {
    procspawn::init();
    let option = Options::parse();
    let verbose = option.verbose;
    if verbose > 0 {
        println!("the model to be checked: {}", option.model);
    }
    let res = if option.portfolio {
        let mut portfolio = Portfolio::new(option);
        portfolio.check()
    } else {
        let aig = Aig::from_file(&option.model);
        let (mut ts, mut restore) = Transys::from_aig(&aig);
        let pre_lemmas = vec![];
        if option.preprocess.sec {
            panic!("sec not support");
        }
        (ts, restore) = ts.simplify(&[], option.ic3, !option.ic3, &restore);
        let mut engine: Box<dyn Engine> = if option.bmc {
            Box::new(BMC::new(option.clone(), ts))
        } else if option.kind {
            Box::new(Kind::new(option.clone(), ts, pre_lemmas))
        } else if option.gic3 {
            Box::new(general::IC3::new(option.clone(), ts))
        } else {
            Box::new(IC3::new(option.clone(), ts, pre_lemmas))
        };
        let res = engine.check();
        if option.certifaiger {
            let certifaiger = engine.certifaiger(&aig, &restore);
            check_certifaiger(&option.model, &certifaiger);
        }
        res
    };
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
