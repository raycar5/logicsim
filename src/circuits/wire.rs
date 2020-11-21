use crate::graph::*;

pub const WIRE: &str = "BUS";

#[macro_export]
macro_rules! wire {
    ($g:expr,$name:ident) => {
        let mut $name: Wire = Wire::new($g, stringify!($name));
    };
}
#[derive(Debug, Clone)]
pub struct Wire {
    bit: GateIndex,
    lever: Option<GateIndex>,
    pub name: String,
}
impl Wire {
    pub fn new<S: Into<String>>(g: &mut GateGraph, name: S) -> Self {
        Self {
            bit: g.or(WIRE),
            lever: None,
            name: name.into(),
        }
    }
    pub fn lever(&mut self, g: &mut GateGraph) -> GateIndex {
        match self.lever {
            Some(lever) => lever,
            None => {
                let lever = g.lever(&self.name);
                self.connect(g, lever);
                self.lever = Some(lever);
                lever
            }
        }
    }
    pub fn connect(&mut self, g: &mut GateGraph, other: GateIndex) {
        g.dpush(self.bit, other);
    }
    pub fn bit(&self) -> GateIndex {
        self.bit
    }
}