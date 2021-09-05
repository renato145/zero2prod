#!/usr/bin/env bash
cargo check
cargo clippy
cargo fmt --all -- --check
cargo sqlx prepare --check -- --lib
