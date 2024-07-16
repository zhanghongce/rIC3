use clap::Parser;
use rIC3::{bmc::BMC, imc::IMC, kind::Kind, portfolio::Portfolio, Args, IC3};

fn main() {
    let args = Args::parse();
    if args.portfolio {
        let mut portfolio = Portfolio::new(args);
        println!("bmc result: {}", portfolio.check());
    } else if args.bmc {
        let mut bmc = BMC::new(args);
        println!("bmc result: {}", !bmc.check());
    } else if args.kind {
        let mut kind = Kind::new(args);
        println!("kind result: {}", kind.check(10));
    } else if args.imc {
        let mut imc = IMC::new(args);
        println!("imc result: {}", imc.check());
    } else {
        let mut ic3 = IC3::new(args);
        println!("ic3 result: {}", ic3.check_with_int_hanlder());
    }
}
