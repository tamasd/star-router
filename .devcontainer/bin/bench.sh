#!/bin/sh

cargo bench
rm -rf ./bench/
cp -R $CARGO_TARGET_DIR/criterion ./bench/
