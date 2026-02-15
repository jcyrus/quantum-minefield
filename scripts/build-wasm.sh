#!/usr/bin/env bash

set -euo pipefail

MODE="${1:-release}"
PROFILE_FLAG="--release"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

if [[ "$MODE" == "dev" ]]; then
  PROFILE_FLAG="--dev"
fi

wasm-pack build crates/qmf-wasm \
  --target web \
  --out-dir ../../apps/web/public/wasm \
  "$PROFILE_FLAG"
