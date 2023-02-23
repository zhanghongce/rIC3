#![feature(assert_matches, is_sorted)]

#[allow(dead_code)]
mod pdr;
#[allow(dead_code)]
mod preimage;
#[allow(dead_code)]
mod utils;

use std::time::Instant;

fn main() {
    let start = Instant::now();
    // let aig = aig::Aig::from_file("../MC-Benchmark/examples/counter/10bit/counter.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/crafted/paper_v3/paper_v3.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc17/single/ringp0.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc19/single/aig/goel/opensource/vis_arrays_buf_bug/vis_arrays_buf_bug.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc08/visbakery.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc08/pdtvishuffman7.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/opensource/h_TreeArb/h_TreeArb.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc08/srg5ptimo.aag").unwrap();

    // Safe

    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2020/mann/rast-p00.aag").unwrap(); //  vs 17s

    // let aig = aig::Aig::from_file(
    //     "../MC-Benchmark/hwmcc20/aig/2019/beem/pgm_protocol.7.prop1-back-serstep.aag",
    // )
    // .unwrap(); // 911s vs 600s

    // let aig =
    //     aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal143/cal143.aag")
    //         .unwrap(); // 30s vs 10s
    // let aig =
    //     aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal102/cal102.aag")
    //         .unwrap(); // 100s vs 88s
    // let aig =
    //     aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal140/cal140.aag")
    //         .unwrap(); // 30s vs 10s
    let aig =
        aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/industry/cal112/cal112.aag")
            .unwrap(); // 167s vs 158s

    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc17/single/intel007.aag").unwrap();

    // Unsafe
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/beem/at.6.prop1-back-serstep.aag").unwrap(); // 21s

    // println!("{}", aig);
    dbg!(pdr::pdr::solve(aig));
    println!("{:?}", start.elapsed());
}
