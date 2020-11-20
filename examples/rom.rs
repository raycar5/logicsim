use std::convert::TryInto;
use wires::*;
fn main() {
    let mut g = GateGraph::new();

    let input1: Vec<_> = constant(1u8);

    let output = rom(&mut g, &input1, &[4u8, 5u8, 6u8]);

    g.init();

    let t = std::time::Instant::now();
    let ticks = g.run_until_stable(1000).unwrap();
    let out = g.collect_u8(&output.clone().try_into().unwrap());
    let d = t.elapsed().as_micros();

    println!(
        "Result: {}, ticks:{}, duration: {}us, {:.2}us/t",
        out,
        ticks,
        d,
        d as f64 / ticks as f64
    );
}
