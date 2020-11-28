use crate::graph::*;

#[macro_export]
/// Creates a wire with the same variable name and gate name.
macro_rules! wire {
    ($g:expr,$name:ident) => {
        #[allow(unused_mut)]
        let mut $name: Wire = Wire::new($g, stringify!($name));
    };
}

/// Data structure that helps with connecting wires to many different components.
///
/// This is basically syntactic sugar for an or gate.
// TODO example.
/// # Example
/// ```
/// # use logicsim::{GateGraphBuilder,counter,Wire,ON,OFF,zeros};
/// # let mut g = GateGraphBuilder::new();
/// let mut reset = Wire::new(&mut g, "reset");
/// let reset_lever = reset.make_lever(&mut g);
/// let clock = g.lever("clock");
///
/// let counter_output = counter(
///     &mut g,
///     clock.bit(),
///     ON,  // enable
///     OFF, // write
///     ON,  // read
///     reset.bit(),
///     &zeros(4), // input
///     "counter"
/// );
/// // Notice I connect the third (index 2) bit of the output to reset.
/// // so as soon as the counter reaches 4 (0b100) it will reset.
/// reset.connect(&mut g, counter_output[2]);
///
/// let output = g.output(&counter_output, "result");
///
/// let ig = &mut g.init();
/// ig.pulse_lever_stable(reset_lever);
///
/// assert_eq!(output.u8(ig), 0);
///
/// ig.pulse_lever_stable(clock);
/// assert_eq!(output.u8(ig), 1);
///
/// ig.pulse_lever_stable(clock);
/// assert_eq!(output.u8(ig), 2);
///
/// ig.pulse_lever_stable(clock);
/// assert_eq!(output.u8(ig), 3);
///
/// ig.pulse_lever_stable(clock);
/// assert_eq!(output.u8(ig), 0);
/// ```
#[derive(Debug, Clone)]
pub struct Wire {
    bit: GateIndex,
    lever: Option<LeverHandle>,
    pub name: String,
}
impl Wire {
    /// Returns a new [Wire] with name `name`.
    pub fn new<S: Into<String>>(g: &mut GateGraphBuilder, name: S) -> Self {
        let name = name.into();
        Self {
            bit: g.or(format!("WIRE:{}", name)),
            lever: None,
            name,
        }
    }

    /// Makes a new lever for the wire, stores it for easy access later and returns
    /// it's [LeverHandle].
    pub fn make_lever(&mut self, g: &mut GateGraphBuilder) -> LeverHandle {
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
    /// Returns Some(LeverHandle) if [make_lever](Self::make_lever) has been called before.
    /// None otherwise.
    pub fn lever(&self) -> Option<LeverHandle> {
        self.lever
    }

    /// Connects a new [GateIndex] to the wire.
    pub fn connect(&self, g: &mut GateGraphBuilder, other: GateIndex) {
        g.dpush(self.bit, other);
    }

    /// Returns the [GateIndex] of the wire.
    pub fn bit(&self) -> GateIndex {
        self.bit
    }
}
