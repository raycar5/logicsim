use super::Wire;
use crate::graph::*;

pub const BUS: &str = "BUS";

#[derive(Debug, Clone)]
pub struct Bus {
    bits: Vec<GateIndex>,
}
impl Bus {
    pub fn new(g: &mut GateGraph, n: usize) -> Self {
        Self {
            bits: (0..n).map(|_| g.or(BUS)).collect(),
        }
    }
    pub fn connect(&mut self, g: &mut GateGraph, other: &[GateIndex]) {
        assert_eq!(
            self.bits.len(),
            other.len(),
            "Use connect_some to connect to a bus of a different width"
        );
        self.connect_some(g, other);
    }
    pub fn connect_some(&mut self, g: &mut GateGraph, other: &[GateIndex]) {
        for (or, bit) in self.bits.iter().zip(other) {
            g.dpush(*or, *bit);
        }
    }
    pub fn len(&self) -> usize {
        self.bits.len()
    }
    pub fn bits(&self) -> &[GateIndex] {
        &self.bits
    }
    pub fn bx(&self, n: usize) -> GateIndex {
        self.bits[n]
    }
    pub fn b0(&self) -> GateIndex {
        self.bits[0]
    }
    pub fn split_wires(&self, g: &mut GateGraph, other: &mut [Wire]) {
        assert_eq!(self.len(), other.len());
        for (bit, wire) in self.bits.iter().zip(other) {
            wire.connect(g, *bit)
        }
    }
}
