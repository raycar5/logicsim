use wires::*;
fn main() {
    let mut graph = GateGraphBuilder::new();
    let g = &mut graph;

    let input1: Vec<_> = constant(1u8);
    let rom_out = g.lever("rom_out");

    let output = rom(g, rom_out.bit(), &input1, &[4u8, 5u8, 6u8], "rom");
    let out = g.output(&output, "out");

    let g = &mut graph.init();
    g.set_lever(rom_out);

    let t = std::time::Instant::now();
    let ticks = g.run_until_stable(1000).unwrap();
    let d = t.elapsed().as_micros();

    println!(
        "Result: {}, ticks:{}, duration: {}us, {:.2}us/t",
        out.u8(g),
        ticks,
        d,
        d as f64 / ticks as f64
    );
}
