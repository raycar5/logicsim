use super::constant;
use super::decoder::decoder;
use crate::graph::*;

pub const ROM: &str = "rom";
// Will fill missing addresses with zeros
pub fn rom<T: Copy>(g: &mut GateGraph, address: &[GateIndex], data: &[T]) -> Vec<GateIndex> {
    let word_length = std::mem::size_of::<T>() * 8;
    assert!(word_length <= 64);

    let decoded = decoder(g, address);
    let out: Vec<GateIndex> = (0..word_length).step_by(1).map(|_| g.or(ROM)).collect();

    for (word, d) in data.iter().zip(decoded.into_iter()) {
        for (or, node) in out.iter().zip(constant(*word).into_iter()) {
            let and = g.and2(d, node, ROM);
            g.dpush(*or, and);
        }
    }

    out
}
