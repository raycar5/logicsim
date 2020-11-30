/*!
Create and simulate digital circuits with Rust abstractions!

In logicsim you use a [GateGraphBuilder][GateGraphBuilder] to create and connect logic gates,
conceptually the logic gates are represented as nodes in a graph with dependency edges to other nodes.

Inputs are represented by constants([ON][ON], [OFF][OFF]) and [levers][lever].

Outputs are represented by [OutputHandles][OutputHandle] which allow you to query the state of gates and
are created by calling [GateGraphBuilder::output][output].

Once the graph is initialized, it transforms into an [InitializedGateGraph][InitializedGateGraph] which cannot be modified.
The initialization process optimizes the gate graph so that expressive abstractions
that potentially generate lots of constants or useless gates can be used without fear.
All constants and dead gates will be optimized away and the remaining graph simplified very aggressively.

**Zero overhead abstractions!**

# Examples
Simple gates.
```
# use logicsim::graph::{GateGraphBuilder,ON,OFF};
let mut g = GateGraphBuilder::new();

// Providing each gate with a string name allows for great debugging.
// If you don't want them affecting performance, you can disable
// feature "debug_gates" and all of the strings will be optimized away.
let or = g.or2(ON, OFF, "or");
let or_output = g.output1(or, "or_output");

let and = g.and2(ON, OFF, "and");
let and_output = g.output1(and, "and_output");

let ig = &g.init();

// `b0()` accesses the 0th bit of the output.
// Outputs can have as many bits as you want
// and be accessed with methods like `u8()`, `char()` or `i128()`.
assert_eq!(or_output.b0(ig), true);
assert_eq!(and_output.b0(ig), false);
```

Levers!
```
# use logicsim::graph::{GateGraphBuilder,ON,OFF};
# let mut g = GateGraphBuilder::new();
let l1 = g.lever("l1");
let l2 = g.lever("l2");

let or = g.or2(l1.bit(), l2.bit(), "or");
let or_output = g.output1(or, "or_output");

let and = g.and2(l1.bit(), l2.bit(), "and");
let and_output = g.output1(and, "and_output");

let ig = &mut g.init();

assert_eq!(or_output.b0(ig), false);
assert_eq!(and_output.b0(ig), false);

// `_stable` means that the graph will run until gate states
//  have stopped changing. This might not be what you want
// if you have a circuit that never stabilizes like 3 not gates
// connected in a loop!
// See [InitializedGateGraph::run_until_stable].
ig.flip_lever_stable(l1);
assert_eq!(or_output.b0(ig), true);
assert_eq!(and_output.b0(ig), false);

ig.flip_lever_stable(l2);
assert_eq!(or_output.b0(ig), true);
assert_eq!(and_output.b0(ig), true);
```

[SR Latch!](https://en.wikipedia.org/wiki/Flip-flop_(electronics)#SR_NOR_latch)
```
# use logicsim::graph::{GateGraphBuilder,ON,OFF};
# let mut g = GateGraphBuilder::new();
let r = g.lever("l1");
let s = g.lever("l2");

let q = g.nor2(r.bit(), OFF, "q");
let nq = g.nor2(s.bit(), q, "nq");

let q_output = g.output1(q, "q");
let nq_output = g.output1(nq, "nq");

// `d1()` replaces the dependency at index 1 with nq.
// We used OFF as a placeholder above.
g.d1(q, nq);

let ig = &mut g.init();
// With latches, the initial state should be treated as undefined,
// so remember to always reset your latches at the beginning
// of the simulation.
ig.pulse_lever_stable(r);
assert_eq!(q_output.b0(ig), false);
assert_eq!(nq_output.b0(ig), true);

ig.pulse_lever_stable(s);
assert_eq!(q_output.b0(ig), true);
assert_eq!(nq_output.b0(ig), false);

ig.pulse_lever_stable(r);
assert_eq!(q_output.b0(ig), false);
assert_eq!(nq_output.b0(ig), true);
```

# The 8 bit computer

In the examples folder you'll find a very simple 8 bit computer, it's a great showcase of what you can achieve by using Rust's constructs
to create modular circuit abstractions.

You can play with it in only 3 shell commands! ([Assuming you have cargo installed](https://rustup.rs/)).
```sh
git clone https://github.com/raycar5/logicsim
cd logicsim
cargo run --release --example computer greeter
```

# Built in circuits

The `circuits` module features a lot of useful pre-built generic components like:

- [WordInput][WordInput]
- [Bus][Bus]
- [Wire][Wire]
- [d_flip_flop][d_flip_flop]
- [rom][rom]

[and many more!][circuits]

# Debugging

Currently there are 2 debugging tools:

## Probes

Calling [GateGraphBuilder::probe][probe] allows you to create probes, which will print the value of all of the bits provided
along with their name whenever any of the bits change state within a [tick][tick].

## Example:
```
# use logicsim::graph::{GateGraphBuilder,ON,OFF};
let mut g = GateGraphBuilder::new();

let l1 = g.lever("l1");
let l2 = g.lever("l2");


let or = g.xor2(l1.bit(), l2.bit(), "or");
let xor = g.xor2(l1.bit(), l2.bit(), "xor");
g.probe(&[or,xor],"or_xor");
let xor_output = g.output1(xor, "xor_output");


let ig = &mut g.init();
assert_eq!(xor_output.b0(ig), false);

ig.set_lever_stable(l1);
assert_eq!(xor_output.b0(ig), true);

ig.set_lever_stable(l2);
assert_eq!(xor_output.b0(ig), false);

ig.reset_lever_stable(l1);
assert_eq!(xor_output.b0(ig), true);

ig.reset_lever_stable(l2);
assert_eq!(xor_output.b0(ig), false);
```
In the terminal you'll see:
```sh
or_xor: 3
or_xor: 1
or_xor: 3
or_xor: 0
```

## .dot files

Using the method [InitializedGateGraph::dump_dot][dump_dot] you can generate [.dot](https://en.wikipedia.org/wiki/DOT_(graph_description_language))
files which can be viewed in many different graph viewers. I recommend [gephi](https://gephi.org/), many others can't handle the size of the graphs
generated by logicsim.

For example here is the graph representation of the [8 bit computer](#the-8-bit-computer):

<img src="https://i.imgur.com/kOiiAKa.png" width="400px" height="271px">

If we zoom in a bit we can see each node is labeled with its name which can help debug really weird bugs.

<img src="https://i.imgur.com/4Y5SOx0.png" width="400px" height="271px">

# Next steps

- Better debugging: I want a gui where I can see many outputs at once with logic-analyzer-like features, probably web based.
- More thorough optimization testing and documentation: I have documented and tested a lot of the public API surface but the optimizations folder
needs some love.
- RISC-V: I want to test out the limits of logicsim by implementing a RISC-V core and running Rust programs in it!
- Compiling: Right now logicsim is just an interpreter, I might try making it compile circuits to either Rust or x86_64 directly.
- Synthesizing: I have a nice fpga dev kit next to me and it would be pretty cool if I could synthesize circuits built in logicsim into it.

[GateGraphBuilder]: https://docs.rs/logicsim/0.1.5/logicsim/graph/struct.GateGraphBuilder.html
[ON]: https://docs.rs/logicsim/0.1.5/logicsim/graph/constant.ON.html
[OFF]: https://docs.rs/logicsim/0.1.5/logicsim/graph/constant.OFF.html
[lever]: https://docs.rs/logicsim/0.1.5/logicsim/graph/struct.GateGraphBuilder.html#method.lever
[OutputHandle]: https://docs.rs/logicsim/0.1.5/logicsim/graph/struct.OutputHandle.html
[output]: https://docs.rs/logicsim/0.1.5/logicsim/graph/struct.GateGraphBuilder.html#method.output
[InitializedGateGraph]: https://docs.rs/logicsim/0.1.5/logicsim/graph/struct.InitializedGateGraph.html
[WordInput]: https://docs.rs/logicsim/0.1.5/logicsim/circuits/struct.WordInput.html
[Bus]: https://docs.rs/logicsim/0.1.5/logicsim/circuits/struct.Bus.html
[Wire]: https://docs.rs/logicsim/0.1.5/logicsim/circuits/struct.Wire.html
[d_flip_flop]: https://docs.rs/logicsim/0.1.5/logicsim/circuits/fn.d_flip_flop.html
[rom]: https://docs.rs/logicsim/0.1.5/logicsim/circuits/fn.rom.html
[circuits]: https://docs.rs/logicsim/0.1.5/logicsim/circuits/index.html
[probe]: https://docs.rs/logicsim/0.1.5/logicsim/graph/struct.GateGraphBuilder.html#method.probe
[tick]: https://docs.rs/logicsim/0.1.5/logicsim/graph/struct.InitializedGateGraph.html#method.tick
[dump_dot]: https://docs.rs/logicsim/0.1.5/logicsim/graph/struct.InitializedGateGraph.html#method.dump_dot
*/
#[macro_use]
pub mod graph;
pub mod data_structures;
extern crate concat_idents;
pub mod circuits;
pub use circuits::*;
pub use graph::*;
