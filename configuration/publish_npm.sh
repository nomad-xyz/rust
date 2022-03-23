#!/usr/bin/env bash
set -e

cargo test

./package_it.sh
cd ts
npm publish