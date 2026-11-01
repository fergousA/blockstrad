#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/engine/target}"
cargo build --locked --release --target wasm32-unknown-unknown --manifest-path engine/Cargo.toml
install -m 644 "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/blockstrad_wasm.wasm" libs/blockstrad.wasm
sha256sum libs/blockstrad.wasm
