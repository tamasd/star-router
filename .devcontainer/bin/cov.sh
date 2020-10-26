#!/bin/sh

cargo tarpaulin --exclude-files /usr/local/rustup -o Html -o Xml --fail-under 80 --no-fail-fast
