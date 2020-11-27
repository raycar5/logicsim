use super::Wire;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("BUS:{}", name)
}

#[derive(Debug, Clone)]
pub struct Bus {
    bits: Vec<GateIndex>,
}
impl Bus {
    pub fn new<S: Into<String>>(g: &mut GateGraphBuilder, n: usize, name: S) -> Self {
        let name = mkname(name.into());
        Self {
            bits: (0..n).map(|_| g.or(name.clone())).collect(),
        }
    }
    pub fn connect(&self, g: &mut GateGraphBuilder, other: &[GateIndex]) {
        assert_eq!(
            self.bits.len(),
            other.len(),
            "Use connect_some to connect to a bus of a different width"
        );
        self.connect_some(g, other);
    }
    pub fn connect_some(&self, g: &mut GateGraphBuilder, other: &[GateIndex]) {
        for (or, bit) in self.bits.iter().zip(other) {
            g.dpush(*or, *bit);
        }
    }
    // The signature is very intentional, one does not simply merge buses.
    pub fn merge(&self, g: &mut GateGraphBuilder, other: Bus) -> Bus {
        self.connect(g, other.bits());
        self.clone()
    }
    pub fn len(&self) -> usize {
        self.bits.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bits.len() == 0
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
    pub fn split_logicsim(&self, g: &mut GateGraphBuilder, other: &mut [Wire]) {
        assert_eq!(self.len(), other.len());
        for (bit, wire) in self.bits.iter().zip(other) {
            wire.connect(g, *bit)
        }
    }
}
impl Into<Vec<GateIndex>> for Bus {
    fn into(self) -> Vec<GateIndex> {
        self.bits
    }
}
