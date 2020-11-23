rm flame.svg
perf record --call-graph=dwarf -g target/release/examples/computer
perf script | stackcollapse-perf | rust-unmangle | flamegraph >flame.svg
