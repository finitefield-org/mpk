# Multi-package candidate handoff

The private Go producer now carries the candidate's complete request hash and
all distinct requested package paths through source, contract, selection,
capture, export, compiler and module-graph inputs. It resolves every requested
function against the original typed packages and rejects a partial or replaced
physical-root set. The complete source/export closure and unselected original
functions remain present.

The candidate contract and export records use separately domain-tagged v2
schemas with a sorted unique package list and the original request hash. The
Rust reader checks the records against one another, the complete native Go
metadata and source snapshots. It requires physical roots to cover exactly the
recorded packages; the existing request binding then independently resolves
every original function and requires both the request hash and physical roots
to match. Legacy v1 records retain their old wire bytes and domains. The Rust
`requested_package()` accessor now returns `Option<&str>`: it is `Some` for a
v1 record and `None` for a v2 candidate; `requested_packages()` exposes the
complete set for either version.

The cross-language test used a freshly captured original HTTP application,
all dependencies and the registered 5,577-file Go SDK. An additional function
and independent source package made four selected functions in three packages,
including one original dependency. All 183 packages, 5,517 source objects and
182 compiler export objects were retained; the independent Rust reader bound
the four roots and the complete original module graph and derived their four
source contract candidates. Missing or replaced roots, extra or omitted
packages, duplicate or unsorted packages, changed request hashes, relabeled
native selected/dependency roles and crossed snapshots were rejected. This
does not establish an installed launcher, source execution or program proof.

The full registered-SDK single-package owner test, original replacement and
sidecar tests, and both Go and Rust 8,192-function selection tests passed.
Existing Rust contract and export input suites passed, including original
metadata and archive controls. The v2 byte-boundary tests preserve the existing
stdout, diagnostics and JSON limits and use separate hash domains. Rust also
matched all 40 freshly generated Go byte-boundary cases and the earlier
30-case original corpus, including the maximum-sized byte streams.
Go vet, Rust Clippy, changed-file formatting and Go 1.23 compile-only
compatibility passed. These tests were selected because they exercise the
changed source, selection, native export and independent Rust reader paths.
Go ran under local Linux amd64 emulation; Rust ran locally on macOS arm64.
Neither constitutes the native Linux release gate.

The live cross-language integration test is ignored by default because it
requires the full fresh Go capture. Run
`TestPracticalCandidateOriginalPackagesOwnWholeImage` with an empty absolute
`MPK_GO_PRACTICAL_SYNTAX_EVIDENCE` directory and the complete registered
Linux/amd64 SDK, then set `MPK_GO_CANDIDATE_MULTI_INPUTS` to its
`versioned-owned` directory and run the ignored Rust test
`complete_go_multi_package_capture_binds_independently_in_rust`.

The practical profile remains inactive and W01 remains incomplete. Exact
handling for Go command-pattern package names, native argument-size limits,
the 8,192-root boundary through the new multi-package handoff, launcher
authority and source/program proofs remain open. The T02-wide
`./scripts/check-fast.sh` gate is deferred to final T02-W04. No validation
archive is committed.
