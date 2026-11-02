# Original module directives

The [independent reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/module_observations/manifest.rs)
derives module identity, the exact Go requirement, language version, toolchain
suggestion, ordered require/exclude rows, replacements and retractions from each
owned original go.mod. The whole original source object and byte buffer remain
owned. Caller graphs, parsed directives and source adapters are not accepted.
The [existing module/source/export boundary](../../../../../../crates/mpk-vc/src/go_practical_type_keys/module_observations.rs)
uses this reader and exposes all selected original manifests for later graph
reconstruction. This does not perform version selection.

Byte-based syntax preserves original comments, line/block structure, invalid
UTF-8 comments and arbitrary path bytes. The pinned Go Unicode15 graphic table
supplies identifier classification. Quoted literal invalid UTF-8 follows native
Unquote replacement per original byte; escaped arbitrary bytes remain exact.
Paths and original toolchain suggestions use byte vectors so parsing cannot
silently replace non-UTF8 path facts or compiler-name suffixes. Toolchain
validation reads the exact version prefix while retaining its entire original
custom suffix. It does not select or invoke another compiler.
The original source bytes are never rewritten.

Requirement/exclusion and replacement versions use complete decimal strings,
short-version normalization, prerelease validation, build-metadata normalization
and path-major rules. Host integer width imposes no new version limit. Original
duplicates and directive order remain intact. The pinned parser intentionally
retains retraction version strings without semver normalization; these facts
grant no selected-version authority. Main/dependency Go and toolchain policies
match the existing Go producer, including complete patch requirements and
dependency toolchain suggestions. Nonempty tool/godebug/ignore directives keep
the producer's rejection; empty permitted blocks remain source syntax.

The [native oracle](../../../../../../go-tools/go2vir/practical_module_manifest_oracle_test.go)
compares the unchanged practical Go parser for both main and dependency modes.
It retains every whole original source/object, verdict and derived fact row.
Cases cover all directive families, duplicates, block and quoted syntax, suffix
comments/indirect flags, version/path-major combinations, large decimals, exact
compiler requirements, invalid bytes and malformed or unavailable inputs.
Actual local/versioned owner snapshots retain every original source/package,
unused HTTP dependency, cache/archive and selected manifest.

Primary references are the complete pinned Go1.25 version sources in primary/,
the unchanged vendored x/mod module/modfile/semver sources retained before each
consumer, and the earlier complete source-owner records. No language profile,
source kind, rule, premise, axiom or kernel limit is changed. Scope remains Go
and Rust only.

The final comparison retains all656 complete original main/dependency cases:
162 accept and494 reject, with exact agreement on every derived fact. The
first652 source buffers and verdicts remain exact; four additional cases retain
non-UTF8 original toolchain suffixes in both modes. All ten affected Rust roots
and three native Go roots pass; the updated native oracle is additionally
executed after byte-preservation changes. Current strict Clippy/vet/formats pass.
The original unused-import and redundant test-match failures remain preserved
with complete inputs and exact outputs. No functional comparison case is dropped.

[Independent inspection](inspection.json) compares all verdicts/facts and original
source-object hashes, verifies the earlier652 raw preimages, checks every original
raw buffer in the actual local/versioned owners and retains the unchanged32
proof streams and old43-root/zero-or70-error universes. Those unchanged proof
consumers are not claimed newly replayed. Every consumed source epoch retains
complete inputs/outputs and formatter states. Explicit-path formatting retains
all actual formatter inputs; full functional consumers keep their entire source
and package universes. The [verification record](verification.json) distinguishes
this scoped result from remaining graph/context/program and release obligations.

General MVS and pruned/unpruned graph reconstruction, complete original graph
inputs, installed compiler/root/capture/context/descriptor/registry/launcher
authority, source/IR/state/storage/frame/callee/program proofs, all56 foundation
families/all169 demands, native Linux release gates and remaining T02–T11 stay
due. R01/R04/R05/R06/W01 remain open; the practical profile stays inactive.
The cumulative T02 gate remains final W04: `./scripts/check-fast.sh`.
