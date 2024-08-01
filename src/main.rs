use aig::Aig;
use clap::Parser;
use rIC3::{bmc::BMC, imc::IMC, kind::Kind, portfolio::Portfolio, transys::Transys, Options, IC3};
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
        let (ts, _) = Transys::from_aig(&aig, !option.ic3);
        if option.preprocess.sec {
            ts.sec();
            return;
        }
        if option.bmc {
            BMC::new(option, ts).check()
        } else if option.kind {
            Kind::new(option, ts).check()
        } else if option.imc {
            IMC::new(option, ts).check()
        } else {
            IC3::new(option, ts).check()
        }
    };
    if verbose > 0 {
        println!("result: {res}");
    }
    exit(if res { 20 } else { 10 });
}
