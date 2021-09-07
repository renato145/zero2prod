#!/usr/bin/env bash
set -x
cargo check
cargo check --tests
cargo clippy
cargo fmt --all -- --check
cargo sqlx prepare --check -- --lib
