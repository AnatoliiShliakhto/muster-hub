#!/usr/bin/env bash
set -euo pipefail

rustup component add clippy rustfmt

mkdir -p "${CARGO_HOME:?}"

ensure_tool() {
  local cmd="$1"
  local install="$2"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Installing $cmd..."
    eval "$install"
  fi
}

ensure_tool "cargo-audit" "cargo install cargo-audit --locked"
ensure_tool "cargo-generate" "cargo install cargo-generate --locked"
ensure_tool "cargo-nextest" "cargo install cargo-nextest --locked"
ensure_tool "dioxus" "cargo install dioxus-cli --locked"

cargo fetch

cargo warm
