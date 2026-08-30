#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' \
  'the standalone predecessor Rust gate was retired by CSHARP-02-T20; run scripts/check-csharp-frontend.sh for the sole Go/Rust/C# successor release' \
  >&2
exit 64
