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

The [analysis/source build checkpoint](verification/analysis-inputs/README.md)
now binds the exact three original analysis module snapshots and vendored code,
retains169 sealed frontend/vendor build inputs and checks linked binary metadata.
Two independent empty-cache builds and a separately reconstructed third build
produce identical complete binaries. The existing uncompiled TODO byte difference
and initial private-output-alias failure remain explicit. Targeted current/legacy,
compiler-consumer, vet/format and independent original-hash checks pass. Complete
actual ordinary foundation/context/registry binding is the next critical work;
application closure/immutable replay still remain due and W01 stays In progress.

The [actual core/basis checkpoint](verification/core-interfaces/README.md) stages
the two exact frozen core interfaces as an explicit W01 foundation prerequisite;
W03 retains common migrations. All255 frozen design inputs remain unchanged.
The fresh35-declaration zero-axiom ordinary basis proves an arbitrary-Nat legacy
recursion identity and conditional Bool false-elimination. Both current checkers
agree on all18 same-byte cases and exact positive hashes. Core/current/legacy,
strict lint/format and targeted importer tests pass. Full ordinary foundation/
descriptor/context and application closure/replay remain due; W01 stays In progress.

The [actual unbounded Nat checkpoint](verification/natural-foundation/README.md)
adds eight ordinary operations and45 checked laws, with88 total declarations
and zero axioms. Both current independent checkers agree on all15 original byte
streams: five acceptances with exact hashes and ten core type mismatches.
The original insufficient reflexivity-only computation recipe is reproduced;
explicit checked computation laws prove all20 correct observations without a
kernel change. Five scoped Rust integration roots, strict Clippy and format
pass. All255 frozen inputs and the preceding core basis remain unchanged.
Balanced integers/ranges, complete representations/operations/source laws and
full descriptor/private context binding remain due; W01 stays In progress.

The [actual balanced integer checkpoint](verification/integer-foundation/README.md)
adds13 ordinary definitions,42 library laws and566 checked Nat-equality
checkpoints. Both current checkers agree on15 same-byte cases with zero-axiom
positive hashes and eleven core type rejections. Integer equality is the
cross-sum relation on full unbounded Nat pairs, and arithmetic/observations
respect redundant representatives. Four targeted integration roots, strict
Clippy and format pass. The initial insufficient embedding-injection proof and
every original input/diagnostic remain; its initial negative passes are not
counted. Half-open folds, full representations/operations/source foundation and
descriptor/private context binding remain due; W01 stays In progress.

The [actual half-open fold checkpoint](verification/range-foundation/README.md)
adds eight ordinary definitions and31 laws. Bounded/interval preservation needs
only equality over the actual visited prefix, including semantic equivalence
for integer sums. Both current checkers agree on17 same-byte cases. Five scoped
integration roots and a focused stronger typed-control root pass, as do strict
Clippy and format. Original insufficient observation recipes, observer failures
and the replaced weaker control remain recorded. Full carrier/source/operation/
definedness foundation and descriptor/private context binding remain due; W01
stays In progress and the T02 gate stays deferred to W04.

The [actual full-width word checkpoint](verification/word-foundation/README.md)
adds1000 ordinary circuit definitions and144 basic universal laws for all four
widths. Exact current checkers agree on20 final candidates with zero-axiom
positive hashes and17 core type rejections. Actual decoded-certificate terms,
independent finite integer arithmetic and fresh pinned native Go agree on all1640
inputs, including forty runtime-domain failures. Corrected scoped Rust tests,
strict Clippy and format pass; initial helper-name/native-preparation failures
remain. Full Nat/Int interpretation/conservation, source-operation/nominal/state/
program binding and descriptor/private context remain due; W01 stays In progress.

The [actual word/numeric checkpoint](verification/word-numeric/README.md)
adds balanced unbounded Nat values/shared weights, pointwise congruence, actual
ripple addition conservation and conditional balanced-Int addition for all four
widths. Both current checkers agree on26 unchanged candidates: three zero-axiom
acceptances and23 core type rejections, including all four dropped-final-carry
claims. Four scoped Rust roots, strict Clippy and format pass. Actual Boolean
circuits, exact integer arithmetic and fresh pinned native Go agree on288
numeric queries, with210 signed no-overflow cases and78 overflow cases. The
original comparison-consumer subtype assertion and focused reproduction remain;
no proof/checker bytes or limits change to repair it. Complete preceding word
bytes and all255 frozen inputs remain unchanged. Remaining numeric/source/
carrier/program laws and descriptor/private context binding remain due; W01
stays In progress and the cumulative T02 gate stays deferred to W04.

The [actual fixed-shift/multiplication checkpoint](verification/word-arithmetic/README.md)
adds genuine unbounded PowerTwo laws, actual discarded-bit/fixed-left-shift
conservation, multiplier-prefix interpretation and complete actual Mul0..width
conservation with a computed Nat witness. Both unchanged checkers agree on26
final streams, with three zero-axiom acceptances and23 core type rejections.
Actual48-input Boolean-circuit evaluation, fresh pinned native Go and exact
integer arithmetic agree on1440 stages and107 carries. Original construction/
conversion/diagnostic/control failures remain; the focused five-case correction
retains eighteen unchanged typed rejections and all passing production/model
inputs. Scoped current Clippy and format pass. Every frozen artifact bound and
all255 design inputs remain unchanged. Remaining numeric/source/carrier/program
laws and descriptor/private context binding remain due; W01 stays In progress
and the cumulative T02 gate stays deferred to W04.

The [actual word bounds/order/guard checkpoint](verification/word-order/README.md)
adds proved complement/value bounds, actual subtraction conservation, unsigned
comparison and equality correspondence, zero values and actual divisor-guard
positivity for all four widths. Both unchanged checkers agree on31 exact
streams, with three zero-axiom acceptances and28 core type rejections. Four
targeted roots, strict Clippy and format pass. Actual Bool circuits, fresh
pinned native Go and independent integers agree on272 original queries;
additional actual carry/zero/guard bits agree with host meanings. The original
large-tree conversion limit and helper compilation error remain with complete
sources/raw diagnostics; explicit small block folds fix conversion without
changing checker fuel or artifact bounds. Prior numeric/arithmetic bytes and
all255 frozen inputs remain exact. Actual division/remainder and the remaining
foundation/source/context/application work remain due; W01 stays In progress
and the cumulative T02 gate stays deferred to W04.
