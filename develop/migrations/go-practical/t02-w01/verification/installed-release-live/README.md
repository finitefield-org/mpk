# Live roots and a staged installed release

This intermediate T02-W01 checkpoint combines the complete, independently held
source and export roots with the existing installed Go compiler reader. The
local observer is an x86-64 Linux **test executable** placed at `bin/mpk` in a
staged release. It uses the actual installed-root loader and pinned registries,
but it is not the production CLI or launcher and establishes no program proof.

The previous temporary Rust bundle volume no longer existed. The frozen Rust
distribution, LLVM archive, crate archives, original source epoch and pinned
Linux images were recovered or reproduced. All 362 registered Rust files and
5,578 Go frontend/SDK files matched their complete inventories. Cargo created
hardlinks for the two Rust frontend outputs, so the recorded first finalization
rejected them; separate byte-identical copies passed the unchanged validator.
The first Go staging attempt rejected writable original directory modes; a
separate complete copy with registered modes passed without changing originals.
The initial download attempt failed because the local Python certificate store
could not verify HTTPS; the system Python succeeded with certificate checks and
the original complete archive hashes. Each failure and the corrected receipts
are retained beside the successful results.

The [live run](live-original/result.json) passed with all four registered Go and
Rust bundles staged, capturing the original Go application and 182 packages.
It retained both root sets through the binding and matched 5,516 source objects,
181 export objects and all 5,577 registered Go SDK files (166,354,960 bytes).
The original source inputs, SDK, Go cache and staged executable remained
unchanged. Four capture-time paths were checked absent after the run. All 16,880
original evidence files were stored with their exact bytes, paths and modes;
the separate [restoration](live-original/restore-check.json) verifies them.
The observer executable was also restored from its retained chunks.

Verification was intentionally scoped to this changed observer and its direct
consumers: the actual Linux x86-64 build, strict Clippy for the same target,
format check, full original Go capture with installed-root binding, independent
evidence restoration and review. The first Linux image did not contain Clippy;
the Linux-target host Clippy run passed, and that first diagnostic remains.
The local QEMU execution on Linux arm64 is **not native Linux release
acceptance**. It proves neither launcher authorization nor Go source semantics
or an application theorem. The practical Go profile is still inactive, and
T02-W01 remains in progress. `./scripts/check-fast.sh` is deferred to the final
T02-W04 under the repository's T-level gate rule.
