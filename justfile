_default:
  @just --choose

dev:
  RUST_LOG=debug cargo run | bunyan

checks:
  ./scripts/checks.sh

tests:
  TEST_LOG=enabled cargo test | bunyan

run:
  cargo run --release | bunyan
