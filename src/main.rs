use clap::Parser;
use ic3::{Args, Ic3};
use std::time::Instant;

fn main() {
    let mut args = Args::parse();
    let aig = // Safe
    // 1000s vs 0.2s
    // "../MC-Benchmark/hwmcc20/aig/2019/beem/pgm_protocol.7.prop1-back-serstep.aag";
    // 31s vs 17s
    "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal143/cal143.aag";
    // 47s vs 23s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal118/cal118.aag";
    // 131s vs 47s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal102/cal102.aag";
    // 216s vs 73s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal112/cal112.aag";
    // 28s vs 11s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal140/cal140.aag";

    if args.model.is_none() {
        args.model = Some(aig.to_string());
    }

    let mut ic3 = Ic3::new(args);
    let start = Instant::now();
    dbg!(ic3.check(), start.elapsed());
}
