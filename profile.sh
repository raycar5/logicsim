perf record -g target/release/wires
perf script | stackcollapse-perf | rust-unmangle | flamegraph >flame.svg
