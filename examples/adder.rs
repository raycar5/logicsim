use std::convert::TryInto;
use wires::*;
fn main() {
    let mut g = BaseNodeGraph::new();
    let bits = 128;
    let a = 0u128;
    let b = -10000i128;

    let input1 = WordInput::new(&mut g, bits);
    let input2 = WordInput::new(&mut g, bits);

    let output = adder(&mut g, OFF, input1.bits(), input2.bits());

    g.init();
    input1.set(&mut g, a);
    input2.set(&mut g, b);

    let t = std::time::Instant::now();
    let mut out = 0;
    let mut ticks = 0;

    for i in 0..10000 {
        input1.set(&mut g, a + i);
        ticks = ticks + 1 + g.run_until_stable(10).unwrap();

        out = g.collect_u128(&output.clone().try_into().unwrap());
    }

    let d = t.elapsed().as_micros();
    println!(
        "Result: {}, ticks:{}, duration: {}us, {:.2}us/t",
        out as i128,
        ticks,
        d,
        d as f64 / ticks as f64
    );
}
