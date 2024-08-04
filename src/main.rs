use aig::Aig;
use clap::Parser;
use rIC3::{
    bmc::BMC, general, imc::IMC, kind::Kind, portfolio::Portfolio, transys::Transys, Options, IC3,
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
        let (mut ts, _) = Transys::from_aig(&aig, option.ic3);
        let pre_lemmas = vec![];
        if option.preprocess.sec {
            assert!(!option.ic3);
            let sec = ts.sec();
            if option.verbose > 0 {
                println!("sec find {} equivalent latchs", sec.len());
            }
            ts.simplify_eq_latchs(&sec, option.ic3);
        }
        if !option.ic3 {
            ts.simplify(&[], false, true);
        }
        if option.bmc {
            BMC::new(option, ts).check()
        } else if option.kind {
            Kind::new(option, ts, pre_lemmas).check()
        } else if option.imc {
            IMC::new(option, ts).check()
        } else {
            if option.ic3_options.bwd {
                IC3::new(option, ts, pre_lemmas).check_with_int_hanlder()
            } else {
                general::IC3::new(option, ts).check()
            }
        }
    };
    if verbose > 0 {
        println!("result: {res}");
    }
    exit(if res { 20 } else { 10 });
}
