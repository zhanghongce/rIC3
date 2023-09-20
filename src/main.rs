use clap::Parser;
use ic3::{Args, Ic3};
use std::time::Instant;

fn main() {
    let mut args = Args::parse();
    let aig = // Safe
    // 1000s vs 0.2s
    // "../MC-Benchmark/hwmcc20/aig/2019/beem/pgm_protocol.7.prop1-back-serstep.aag";
    // 31s vs 17s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal143/cal143.aag";
    // 47s vs 23s
    "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal118/cal118.aag";
    // 131s vs 47s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal102/cal102.aag";
    // 216s vs 73s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal112/cal112.aag";
    // 28s vs 11s
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal140/cal140.aag";
    // 21s vs 19s
    // "../MC-Benchmark/hwmcc17/single/intel007.aag";
    // ? vs 141s
    // "../MC-Benchmark/hwmcc17/single/6s0.aag";
    // ? vs 216s
    // "../MC-Benchmark/hwmcc17/single/6s269r.aag";
    // ? vs
    // "../MC-Benchmark/hwmcc17/single/6s281b35.aag";
    // 110s vs 170s
    // "../MC-Benchmark/hwmcc17/single/6s404rb4.aag";
    // 225s vs 260s
    // "../MC-Benchmark/hwmcc17/single/6s109.aag";
    // 61s vs 43s
    // "../MC-Benchmark/hwmcc17/single/bob05.aag";
    // 3s vs 3s
    // "../MC-Benchmark/hwmcc17/single/neclaftp4002.aag";
    //
    // "../MC-Benchmark/hwmcc17/single/nusmvreactorp5.aag";
    // ?
    // "../MC-Benchmark/hwmcc17/single/6s343b08.aag";
    // ?
    // "../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal227/cal227.aag";
    // ?
    // "../MC-Benchmark/hwmcc20/aig/2019/beem/brp2.6.prop3-back-serstep.aag";
    // ?
    // "../MC-Benchmark/hwmcc20/aig/2019/beem/at.6.prop1-back-serstep.aag";

    if args.model.is_none() {
        args.model = Some(aig.to_string());
    }

    let mut ic3 = Ic3::new(args, None);
    let start = Instant::now();
    dbg!(ic3.check(), start.elapsed());
}
