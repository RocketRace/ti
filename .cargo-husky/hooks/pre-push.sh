#!/bin/bash

set -ex

cargo fmt --check --all
cargo clippy --all-features --all-targets --workspace -- --deny warnings
cargo test --all-features --all-targets --workspace