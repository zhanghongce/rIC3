use clap::Parser;
use rIC3::{bmc::BMC, imc::IMC, kind::Kind, portfolio::Portfolio, Options, IC3};
use std::process::exit;

fn main() {
    let args = Options::parse();
    let verbose = args.verbose;
    if verbose > 0 {
        println!("the model to be checked: {}", args.model);
    }
    let res = if args.portfolio {
        let mut portfolio = Portfolio::new(args);
        portfolio.check()
    } else if args.bmc {
        let mut bmc = BMC::new(args);
        bmc.check()
    } else if args.kind {
        let mut kind = Kind::new(args);
        kind.check(10)
    } else if args.imc {
        let mut imc = IMC::new(args);
        imc.check()
    } else {
        let mut ic3 = IC3::new(args);
        ic3.check_with_int_hanlder()
    };
    if verbose > 0 {
        println!("result: {res}");
    }
    if res {
        exit(20);
    } else {
        exit(10);
    }
}
