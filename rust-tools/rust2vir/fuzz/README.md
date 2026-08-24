# Isolated rust2vir fuzzing

`scripts/check-fuzz-smoke.sh` is the deterministic gate. It runs both targets
through the pinned hermetic launcher for exactly 256 iterations with seed 1.
The launcher copies the closed seed inventory into a fresh private corpus and
writes artifacts only to a separate private directory.

For unbounded developer diagnostics only (never an acceptance gate), prepare
a disposable local `/mpk` test mount and invoke cargo-fuzz directly—not
`run-rust2vir-toolchain.sh`, which deliberately accepts only the frozen
256-run command:

```sh
cd /mpk/frontend
cargo fuzz run --fuzz-dir /mpk/work/fuzz-project driver_protocol -- -runs=0
cargo fuzz run --fuzz-dir /mpk/work/fuzz-project rust_contract -- -runs=0
```

Elapsed fuzzing time and locally discovered artifacts never affect artifact
acceptance. Add a minimized regression as an enumerated seed and deterministic
test before relying on it.
