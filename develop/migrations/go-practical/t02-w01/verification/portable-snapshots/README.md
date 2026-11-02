# Complete original snapshots in a relocated checkout

The snapshot restorer previously compared saved absolute paths with the current
checkout. The [unchanged original helper](original-driver.py) rejects a complete
copy in another directory before restoring any source; its [failure record](original-reproduction.json)
and exact stdout/stderr remain. Historical absolute paths identify the original
run and must not become restore destinations or require rewriting old receipts.

The [current helper](../snapshot-inputs/preserve-originals.py) resolves canonical
relative member paths under its current root. It verifies every original archive
chunk, member, source byte/hash and permission mode, rejects linked storage or
restore paths, and never overwrites an existing corrupt original. The complete
historical manifest and all nine encoded archive parts remain byte-identical.
The [first successful relocation](relocated/result.json) retains its exact
[consumed helper preimage](relocated/consumed-driver.py) alongside its original
input and output receipts.

The [final integration](relocated-rust/result.json) copies all current workspace
members and their complete consumed fixture/proof inputs to a new absolute path,
then recompiles the actual two Rust integration binaries there. No restored
snapshot file exists before launch. The ordinary test helper restores all77,162
original files,696,793,808 bytes, from the complete unchanged original storage.
Independent post-run comparison checks every relative path, byte and mode,
and both source workspaces remain frozen throughout execution.

All5 snapshot/capture roots and4 source-input roots pass. Both complete original
source units retain43 roots each, zero/70 error-case universes and all32 prior
certificate byte streams. Compilation, execution and final full-file inspection
take240.95 seconds on this host; this is not a native release resource result.
All15 [focused controls](controls.json) pass: one valid original relative member
and14 corruption/path/link rejections. Changed Python syntax checks pass.

Only the restoration helper changes; the Go/Rust production implementation and
its prior lint/format/native/checker results remain unchanged. The selected tests
exercise its actual consumers and complete original inputs. No unrelated suite
or intermediate whole-T gate is repeated. The [verification record](verification.json)
keeps the original failure distinct from successful checks.

Canonical capture/context/descriptor/registry and launcher authority, complete
module/build/compiler roles, source/IR/state/storage/frame/callee/program proofs,
all56 foundation families/all169 demands, native Linux release gates and the
remaining T02-T11 stay due. R01/R06/W01 remain open; the practical profile stays
inactive. The cumulative T02 gate remains final W04: `./scripts/check-fast.sh`.
Go and Rust remain the only supported source languages.
