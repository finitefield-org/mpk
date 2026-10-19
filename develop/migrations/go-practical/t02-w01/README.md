# T02-W01 private contexts and immutable dependency capture

Status: In progress. Entry is T01 completion commit
`fe1b85cb70c7ea390c02469706d09cdc913f973f`. The
[requirements](requirements.json) retain the complete W01 scope: private
contexts, compiler/stdlib/analysis identity, immutable multi-capability capture,
exact module/replacement resolution, archive/Go sums, export metadata and
rejection of escape, stale inputs, network/VCS/generation/toolchain fallback.

The first implementation component is
`go-tools/go2vir/practical_source.go`. It constructs the final
`mpk.go.source_object.v1` complete domain-separated identity using `go_file`
and all seven other frozen source kinds. Original bytes are independently
owned; returned bytes and metadata cannot change the stored buffer. Raw binary
bytes, Unicode path spelling and zero-length buffers retain their exact
identity. Logical paths and the16MiB file/1MiB overlay bounds are checked.
Oversized inputs reject before ownership. Strict import rejects duplicate, extra or missing fields, non-integral
byte counts, Boolean/null confusion, unknown/old kinds and mismatched original
bytes or complete identities.

These source-object records supply no filesystem/capability authorization,
compiler identity, module resolution, selected-body acceptance or application
proof. Those executors remain due in W01 and later T02 work items. Existing
candidate graphs remain at their original epoch; no graph is relabeled or
promoted. The public frontend continues to reject the inactive practical
profile before source reads.

The [selected checks](verification.json) pass with the exact Go1.25.0 Darwin
toolchain: eight root tests and26 subtests, including frozen identity vectors,
arbitrary original bytes/caller alias mutations, metadata forgeries, old-kind
crossing, logical-path and byte boundaries, existing shared canonical/strict
JSON and closed public-profile arguments. `go vet` and changed-file `gofmt`
pass. The first shared-cache setup failure and the successful isolated-cache
configuration are recorded separately. The strict input review also identifies obsolete-kind literals in the initial
negative tests. The final tests directly consume the exact original negative
object referenced by the frozen catalog, with its full fixture hash checked.
The three exact failed diagnostics are preserved through the existing historical
seal policy. All127 rules/four scopes and all previous36 seals remain; twelve
altered-seal controls and all existing fixture checks pass. No production or
directory exemption is added. No historical graph changes. No dependency/toolchain download
or C# test is used. These are component checks, not Linux release acceptance.

No complete W01 or T02 claim is made. Targeted checks cover changed code and
affected consumers; the cumulative T02 gate is deferred to W04 and will use
`./scripts/check-fast.sh --skip-csharp` under the user's explicit instruction.
All169 Go theorem demands remain unelaborated and no practical profile activates.

The [rooted capture checkpoint](verification/rooted-capture/README.md) now adds
actual multi-capability filesystem reads and sealed owned snapshots. It checks
exact names, rooted/nonblocking opens, links/special files, observed namespace/
content changes, aggregate budgets and deterministic identities. The final
Go1.25 selection passes20 root tests/43 subtests; after the native-asset review
fix all8 affected rooted tests/17 subtests plus vet/format pass. The Go1.23
compatibility build separately passes3 root tests/11 subtests and rejects
unavailable rooted capture. Materializing the loader namespace and complete
context/module/compiler/source/export authority remain due; W01 stays open.

The [original module input checkpoint](verification/module-inputs/README.md)
adds exact manifest/version/checksum handling and captured local replacement
mapping. A fresh pinned offline Go module-graph comparison agrees with original
nested-module inputs. The12-root52-subtest selection, vet/format and separate
legacy build pass. Versioned archive/graph/export capture and checked private
context/foundation remain due; this does not complete W01.

The [ordinary Boolean checkpoint](verification/ordinary-boolean/README.md)
adds5 actual definitions and5 universal ordinary laws, exact component import,
and fresh current-checker evidence. Both checkers accept the same zero-axiom
certificate with identical hashes and reject an incorrect ordinary equality
claim. Three targeted Rust integration tests, strict Clippy and format pass;
no C# tests execute. This Boolean basis does not register the complete private
foundation/context or complete W01.

The [original archive checkpoint](verification/module-archives/README.md) now
verifies raw ZIP identity, Go h1, complete namespace/size/CRC rules and source
content linkage through actual captured checksum authority. Shared limits and
private-map reconstruction reject forged/crossed inputs; current offline Go
checksum observations agree on original and tampered files. The review fix and
all targeted checks are recorded. Complete module metadata/graph/context/loader
binding remain due; no W01 completion or profile activation is claimed.

The [versioned module checkpoint](verification/module-snapshots/README.md)
now binds original .mod/.info/.ziphash/.zip inputs and verifies original checksum,
version, root-manifest and legacy no-rewrite consistency. Its9-root34-subtest
selection, current vet/format and separate legacy build pass. Actual selected
module graph, complete context/compiler authority and immutable original-package
source/export loading remain due; W01 is still In progress.

The [actual Go module discovery checkpoint](verification/module-resolution/README.md)
now runs pinned offline Go version/replacement selection and binds preparation
observations to original snapshots, roots, languages and main checksum ownership.
An initial11-root31-subtest selection passes; after a reproduced checksum-owner
omission is fixed, all4 affected binder roots/10 subtests and current legacy,
vet/format checks pass. This initially reads the original mutable namespace;
complete consulted-input capture and immutable authoritative replay remain due,
alongside the complete private context/compiler/foundation. W01 stays In progress.

The [prepared compiler checkpoint](verification/compiler-image/README.md) now
checks a complete original Go1.25.0 image:5,577 files,9 exact command/tool binaries,
original specification/version witnesses and complete component ownership.
Independent original-SDK hashes agree with the retained fresh inventory. Native
image/reauthorized-change controls, the eleven-case build-info matrix, legacy
checks and current vet/format pass; the original fixture-mode failure is retained.
Complete analysis-library/frontend build identity and checked context/registry
binding remain due before immutable production loading; W01 stays In progress.
