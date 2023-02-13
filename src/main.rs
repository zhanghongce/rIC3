#![feature(assert_matches)]

#[allow(dead_code)]
mod pdr;
#[allow(dead_code)]
mod preimage;
#[allow(dead_code)]
mod utils;

use std::time::Instant;

fn main() {
    let start = Instant::now();
    let aig = aig::Aig::from_file("../MC-Benchmark/examples/counter/3bit/counter.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/crafted/paper_v3/paper_v3.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc17/single/ringp0.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc19/single/aig/goel/opensource/vis_arrays_buf_bug/vis_arrays_buf_bug.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc08/visbakery.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc08/pdtvishuffman7.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/opensource/h_TreeArb/h_TreeArb.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc08/srg5ptimo.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/goel/opensource/vcegar_QF_BV_itc99_b13_p10/vcegar_QF_BV_itc99_b13_p10.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2020/mann/simple_alu.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/beem/anderson.3.prop1-back-serstep.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc20/aig/2019/beem/at.6.prop1-back-serstep.aag").unwrap();
    // let aig = aig::Aig::from_file("../MC-Benchmark/hwmcc17/single/intel007.aag").unwrap();
    println!("{}", aig);
    // dbg!(preimage::circuit_sat::solve(aig.clone()));
    // dbg!(pdr::pdr::solve(aig.clone()));
    dbg!(pdr::postimage::solve(aig.clone()));
    println!("{:?}", start.elapsed());
}
