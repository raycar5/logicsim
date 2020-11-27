use super::Wire;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("BUS:{}", name)
}

/// Data structure that helps with managing buses, it allows you to connect &[[GateIndex]] to it as well as providing
/// a &[[GateIndex]] to connect to other components.
///
/// This is basically syntactic sugar for a set of or gates.
///
/// # Example
/// ```
/// # use logicsim::{GateGraphBuilder,constant,Bus,ON};
/// # let mut g = GateGraphBuilder::new();
/// let input1 = constant(0x01u8);
/// let input2 = constant(0x10u8);
///
/// let bus = Bus::new(&mut g, 8, "bus");
/// bus.connect(&mut g, &input1);
/// bus.connect(&mut g, &input2);
///
/// let output = g.output(bus.bits(), "result");
///
/// let ig = &g.init();
/// assert_eq!(output.u8(ig), 0x11);
/// ```
#[derive(Debug, Clone)]
pub struct Bus {
    bits: Vec<GateIndex>,
}
impl Bus {
    /// Returns a new [Bus] of width `width` with name `name`.
    pub fn new<S: Into<String>>(g: &mut GateGraphBuilder, width: usize, name: S) -> Self {
        let name = mkname(name.into());
        Self {
            bits: (0..width).map(|_| g.or(name.clone())).collect(),
        }
    }

    /// Connects a &[[GateIndex]] to the bus, each bit of the output of the bus will be set to the or
    /// of every corresponding bit in the inputs.
    ///
    /// # Panics
    ///
    /// Will panic if `other.len()` != `self.len()`. Use [connect_some](Bus::connect_some)
    /// if this is not your desired behavior.
    pub fn connect(&self, g: &mut GateGraphBuilder, other: &[GateIndex]) {
        assert_eq!(
            self.bits.len(),
            other.len(),
            "Use connect_some() to connect to a bus of a different width"
        );
        self.connect_some(g, other);
    }

    /// Connects a &[[GateIndex]] to the bus, each bit of the output of the bus will be set to the or
    /// of every corresponding bit in the inputs.
    ///
    /// If there are excess bits in `other`, they won't get connected to the bus.
    /// If there are missing bits in `other` only other.len() will be connected to the bus.
    pub fn connect_some(&self, g: &mut GateGraphBuilder, other: &[GateIndex]) {
        for (or, bit) in self.bits.iter().zip(other) {
            g.dpush(*or, *bit);
        }
    }

    /// Connects the bits of `other` to `self` and returns a clone of `self`.
    // The signature is very intentional, one does not simply merge buses.
    pub fn merge(&self, g: &mut GateGraphBuilder, other: Bus) -> Bus {
        self.connect(g, other.bits());
        self.clone()
    }

    /// Returns the width of the bus.
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// Returns true if `self.len()` == 0.
    pub fn is_empty(&self) -> bool {
        self.bits.len() == 0
    }

    /// Returns a &[[GateIndex]] to connect to other components.
    pub fn bits(&self) -> &[GateIndex] {
        &self.bits
    }

    /// Returns the [GateIndex] of the `n`th bit in the bus.
    ///
    /// # Panics
    ///
    /// Will panic if `n` >=  `self.len()`.
    pub fn bx(&self, n: usize) -> GateIndex {
        self.bits[n]
    }

    /// Returns the [GateIndex] of the 0th bit in the bus.
    ///
    /// # Panics
    ///
    /// Will panic if `self.is_empty()`.
    pub fn b0(&self) -> GateIndex {
        self.bits[0]
    }

    /// Connects the bus to a series of [Wires](Wire).
    ///
    /// # Panics
    ///
    /// Will panic if `self.len()` != `other.len()`.
    pub fn split_wires(&self, g: &mut GateGraphBuilder, other: &mut [Wire]) {
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
