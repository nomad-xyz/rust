#!/usr/bin/env bash
set -e

cargo clippy --target wasm32-unknown-unknown
cargo test

./package_it.sh
cd ts
npm publish