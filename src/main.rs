use clap::Parser;
use rIC3::{bmc::BMC, Args, IC3};

fn main() {
    let args = Args::parse();
    if args.bmc {
        let mut bmc = BMC::new(args);
        println!("bmc result: {}", !bmc.check_no_incremental());
    } else {
        let mut ic3 = IC3::new(args);
        println!("ic3 result: {}", ic3.check_with_int_hanlder());
    }
}
