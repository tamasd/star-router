#!/bin/sh

cargo test --no-run
cargo test --bench

for i in $(ls $CARGO_TARGET_DIR/debug/deps/*_benchmark-* | grep -vE '\.(d|rmeta)$'); do
    sudo perf record --call-graph dwarf $i
done
