# Go analysis vendor manifest: in progress

The independent Rust parser derives the Go analysis package set directly from
the complete owned `metadata` source buffer. It checks all three exact module
versions, section and annotation structure, unique package ownership, the
pinned Go import-path rules and the existing 32,768-package limit. It retains
the original source object and bytes. The resulting facts do not authenticate
the source capability, module archive contents, compiled frontend, practical
context or any source/program theorem.

The exact original `vendor/modules.txt` contains 29 packages. The pinned Go
[oracle](native/result.json) and [Rust tests](vendor-selected/result.json) agree
on all 36 cases: ten accepted and 26 rejected. They retain the complete original
manifest while testing Unicode whitespace, duplicate/crossed sections, exact
versions, invalid UTF-8 and import-path edge cases. A valid supplemental
manifest preserves every original byte and reaches 32,768 unique packages;
adding one more rejects. The package names in that boundary case are manifest
inputs, not manufactured application packages or a claim that those files exist.
The pinned Go implementation accepts an import element containing `a..b`; the
comparison preserves that observed behavior instead of imposing a new rule.

The [independent inspection](inspection.json) verifies all 35 distinct input byte
files, all 36 source IDs, the exact original 29-package set, and both full
boundary inputs. The prelaunch [archive](prelaunch/result.json) retains 424
source/environment files and verifies the complete 5,577-file registered SDK.
The new Rust test and all eight affected existing source/type-key tests pass.
Scoped Rust Clippy, changed-file formatting and the affected Go package's vet
also pass; their selection reasons and raw outputs are retained separately.

The practical Go profile remains inactive and T02-W01 stays open. Complete
analysis source/archive correspondence, source-to-binary build authority,
private context, application capture and source/program proofs remain due.
These local emulated Go observations do not constitute native Linux release
acceptance. The whole T02 gate `./scripts/check-fast.sh` is deferred to the final
T02-W04, as required for an intermediate W.
