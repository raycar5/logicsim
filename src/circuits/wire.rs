use crate::graph::*;

#[macro_export]
macro_rules! wire {
    ($g:expr,$name:ident) => {
        let mut $name: Wire = Wire::new($g, stringify!($name));
    };
}
#[derive(Debug, Clone)]
pub struct Wire {
    bit: GateIndex,
    lever: Option<LeverHandle>,
    pub name: String,
}
impl Wire {
    pub fn new<S: Into<String>>(g: &mut GateGraphBuilder, name: S) -> Self {
        let name = name.into();
        Self {
            bit: g.or(format!("WIRE:{}", name)),
            lever: None,
            name,
        }
    }
    pub fn lever(&mut self, g: &mut GateGraphBuilder) -> LeverHandle {
        match self.lever {
            Some(lever) => lever,
            None => {
                let lever = g.lever(&self.name);
                self.connect(g, lever.bit());
                self.lever = Some(lever);
                lever
            }
        }
    }
    pub fn connect(&mut self, g: &mut GateGraphBuilder, other: GateIndex) {
        g.dpush(self.bit, other);
    }
    pub fn bit(&self) -> GateIndex {
        self.bit
    }
}
