#![feature(bindings_after_at)]
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
#[macro_use]
mod graph;
mod slab;
mod state;
use graph::*;
use state::State;

fn main() {
    let mut g = BaseNodeGraph::new();

    let bits = 128;
    let mut a: i128 = 8;
    let mut b: i128 = -3;

    // Adder levers
    let adder_cin = g.lever("cin");
    let adder_levers: Vec<NodeIndex> = (0..bits * 2)
        .step_by(1)
        .map(|i| g.lever(format!("lever{}", i)))
        .collect();

    let clock = g.lever("clock");
    let mut qs = Vec::new();
    // D flip flop register
    let nand_flops = false;
    for i in 0..bits * 2 {
        if nand_flops {
            let bot_flop_nand = g.nand2(adder_levers[i], LATER, format!("bot_flop_and{}", i));

            let bot_flip_and = g.and2(bot_flop_nand, clock, format!("bot_flip_and1{}", i));
            let bot_flip_nand = g.nand2(bot_flip_and, LATER, format!("bot_flip_and2{}", i));
            g.d1(bot_flop_nand, bot_flip_nand);

            let top_flop_nand = g.nand2(clock, LATER, "");
            g.d1(bot_flip_nand, top_flop_nand);

            let top_flip_nand = g.nand2(bot_flop_nand, top_flop_nand, "");
            g.d1(top_flop_nand, top_flip_nand);

            let nq = g.nand2(bot_flip_nand, LATER, "");
            let q = g.nand2(top_flop_nand, nq, "");
            g.d1(nq, q);
            qs.push(q);
        } else {
            let input = adder_levers[i];
            let ninput = g.not1(input, "");

            let flip_and = g.and2(ninput, clock, "");
            let flop_and = g.and2(input, clock, "");

            let q = g.nor2(flip_and, LATER, "");

            let nq = g.nor2(flop_and, q, "");
            g.d1(q, nq);
            qs.push(q)
        }
    }

    // Adder
    let mut outputs = Vec::new();

    let mut cin = adder_cin;
    for i in 0..bits {
        let x = g.xor2(qs[i * 2], qs[i * 2 + 1], "x");
        let output = g.xor2(x, cin, format!("output{}", i));
        let a = g.and2(qs[i * 2], qs[i * 2 + 1], "a");
        let a2 = g.and2(x, cin, "a2");
        cin = g.or2(a2, a, format!("carry{}", i));
        outputs.push(output)
    }

    let mut state = State::new(g.len());
    for lever in &adder_levers {
        if lever.idx % 2 == 0 {
            state.set(*lever, a & 1 != 0);
            a = a >> 1;
        } else {
            state.set(*lever, b & 1 != 0);
            b = b >> 1;
        }
    }
    let drive = |g: &mut BaseNodeGraph, state: &mut State| {
        let mut out = a * 0;
        for (i, output) in outputs.iter().enumerate() {
            let mask = 1 << i;
            let s = g.value(*output, state);
            if s {
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
    g.init(&mut state);
    state.set(clock, true);
    let t = std::time::Instant::now();
    //while hash != new_hash {
    for _ in 0..1000000 {
        ticks += 1;
        if ticks == 2 {
            state.set(clock, false);
        }

        hash = new_hash;
        out = drive(&mut g, &mut state);
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
