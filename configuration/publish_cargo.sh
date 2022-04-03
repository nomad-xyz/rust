#!/usr/bin/env bash
set -e

cargo clippy
cargo test

cargo publish