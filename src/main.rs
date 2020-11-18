use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
mod graph;
mod slab;
mod state;
use graph::*;
use state::State;

fn main() {
    let mut g = BaseNodeGraph::new();

    let bits = 128;
    let adder_cin = g.lever().unwrap();
    let adder_levers: Vec<usize> = (0..bits * 2)
        .step_by(1)
        .map(|_| g.lever().unwrap())
        .collect();

    let mut outputs = Vec::new();

    let mut cin = adder_cin;
    for i in 0..bits {
        let x = g
            .xor2(adder_levers[i * 2], adder_levers[i * 2 + 1])
            .unwrap();
        let output = g.xor2(x, cin).unwrap();
        let a = g
            .and2(adder_levers[i * 2], adder_levers[i * 2 + 1])
            .unwrap();
        let a2 = g.and2(x, cin).unwrap();
        cin = g.or2(a2, a).unwrap();
        outputs.push(output)
    }

    let mut state = State::new(g.len());
    let mut a: i128 = 8;
    let mut b: i128 = -5;
    for lever in &adder_levers {
        if lever % 2 == 0 {
            state.set(*lever, a & 1 != 0);
            a = a >> 1;
        } else {
            state.set(*lever, b & 1 != 0);
            b = b >> 1;
        }
    }
    let drive = |state: &mut State| {
        let mut out: i128 = 0;
        for (i, output) in outputs.iter().enumerate() {
            let mask = 1 << i;
            if (g.value(*output, state)).unwrap() {
                out = out | mask
            } else {
                out = out & !mask
            }
        }
        out
    };

    let mut hash = 0;
    let mut new_hash = 1;
    let mut ticks = 0;
    let mut out = 0;
    let t = std::time::Instant::now();
    //while hash != new_hash {
    for _ in 0..1000000 {
        ticks += 1;
        hash = new_hash;
        out = drive(&mut state);
        state.tick();
        let mut hasher = DefaultHasher::new();
        state.hash(&mut hasher);
        new_hash = hasher.finish();
    }
    let d = t.elapsed().as_micros();
    println!(
        "Result: {}, ticks: {}, duration: {}us, {:.2}us/t",
        out,
        ticks,
        d,
        d as f64 / ticks as f64
    );
}
