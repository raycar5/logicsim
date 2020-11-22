use wires::*;
fn main() {
    let g = &mut GateGraph::new();
    let bits = 128;
    let a = 0u128;
    let b = -10000i128;

    let input1 = WordInput::new(g, bits);
    let input2 = WordInput::new(g, bits);

    let output = adder(g, OFF, input1.bits(), input2.bits());
    let out = g.output(&output, "out");

    g.init();
    input1.set(g, a);
    input2.set(g, b);

    let t = std::time::Instant::now();
    let mut ticks = 0;
    let mut res = 0;

    for i in 0..10000 {
        input1.set(g, a + i);
        ticks = ticks + 1 + g.run_until_stable(10).unwrap();

        res = out.i128(g);
    }

    let d = t.elapsed().as_micros();
    println!(
        "Result: {}, ticks:{}, duration: {}us, {:.2}us/t",
        res,
        ticks,
        d,
        d as f64 / ticks as f64
    );
}
