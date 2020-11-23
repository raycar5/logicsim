perf record -g target/release/examples/computer
perf script | stackcollapse-perf | rust-unmangle | flamegraph >flame.svg
