#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' \
  'the standalone predecessor Rust gate was retired; run sudo ./scripts/check-go-rust-frontend.sh for the sole Go/Rust successor release' \
  >&2
exit 64
