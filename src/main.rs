#![feature(assert_matches)]

use std::time::Instant;

mod preimage;
mod utils;

fn main() {
    let start = Instant::now();
    // let aig = aig::Aig::from_file("../MC-Benchmark/examples/xor/xor.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/examples/counter/10bit/counter.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/crafted/paper_v3/paper_v3.aag").unwrap();
    // let aig = aig::Aig::from_file("../or-or.aag").unwrap();
    // let aig = aig::Aig::from_file("../cex2.aag").unwrap();
    let aig = aig::Aig::from_file(
        "../MC-Benchmark/hwmcc20/aig/2019/goel/opensource/h_TreeArb/h_TreeArb.aag",
    )
    .unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc17/single/ringp0.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc19/single/aig/goel/opensource/vis_arrays_buf_bug/vis_arrays_buf_bug.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc08/visbakery.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc08/pdtvishuffman7.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc08/srg5ptimo.aag").unwrap();
    preimage::cav11::solve(aig.clone());
    dbg!(preimage::circuit_sat::solve(aig));
    println!("{:?}", start.elapsed());
}
