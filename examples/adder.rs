use logicsim::*;
fn main() {
    let mut graph = GateGraphBuilder::new();
    let g = &mut graph;
    let bits = 128;
    let a = 0u128;
    let b = -10000i128;

    let input1 = WordInput::new(g, bits, "input");
    let input2 = WordInput::new(g, bits, "input");

    let output = adder(g, OFF, &input1.bits(), &input2.bits(), "adder");
    let out = g.output(&output, "out");

    let g = &mut graph.init();
    input1.set_to(g, a);
    input2.set_to(g, b);

    let t = std::time::Instant::now();
    let mut ticks = 0;
    let mut res = 0;

    for i in 0..10000 {
        input1.set_to(g, a + i);
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
