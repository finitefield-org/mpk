# Original module observations

The [original owner](../../../../../../go-tools/go2vir/practical_module_observation_inputs.go)
hands off its whole original resolver output/diagnostics and every selected
manifest identity under the same mutex as the source/export capture. There is
no caller graph, manifest, package, command or source callback. Returned buffers
own their original data independently of owner retirement. The unchanged export
entry point delegates to the same locked implementation.

The [Rust reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/module_observations.rs)
checks the complete selected-module set against that original observation,
the source snapshot and every original application package's module metadata.
The main module is bound to its original manifest. A source token/line-block
projection derives original module/Go/replacement spelling, including quoted
strings and exact-version-before-wildcard lookup. Ignored main-module replacement
directives and duplicate unused directives remain retained; selected duplicate
directives reject under the existing binding rule. This projection is not a
complete modfile validator.

Local replacements bind their normalized original target, longest matching
capability and exact manifest/source location. Versioned replacements retain
their original selected identity and effective cache spelling, including escaped
uppercase names. Every nonstandard package must retain the corresponding
manifest object, import-path-relative directory, selected version and replacement
identity. Native JSON duplicate fields reject before typed projection. Whole
native metadata, diagnostics, source/export buffers and unselected manifests
remain retained.

These are private observation/source consistency checks. They do not independently
establish MVS/version selection, complete module identity/directive semantics,
ZIP/go.sum correctness, authorized roots, an installed compiler/context or
source/program semantics. The existing native producer's module/archive checks
retain their own earlier evidence; those checks are not relabeled as independent
Rust verification. No practical profile is activated.

The actual local-replacement input retains all181 packages/180 exports/5,667
objects/164,737,170 raw bytes, including an unused HTTP body and unexecuted
initialization. The actual versioned-replacement input retains both packages/
exports, all5,490 objects/85,379,438 raw bytes, original go.sum and all four
original cached mod/info/ZIP/ziphash files. The existing HTTP/embed export input
retains all180 packages/179 exports/5,663 objects/164,865,655 raw bytes. None of
these original source or package universes is reduced to the selected Root.

All three actual Go roots and25 affected Rust roots pass. Twenty-four resealed
module/package metadata controls reject crossed manifests, paths, versions,
replacement targets, malformed/duplicate native fields and omitted graph rows
while retaining the same original capture and one Root contract. Existing21
export metadata controls and four coherent archive controls also pass. Both
43-root units, zero/70-error universes and all32 proof streams stay byte-exact.
Strict affected Clippy, Go vet and changed-file formats pass.

[Independent inspection](inspection.json) checks every original raw source buffer,
whole metadata stream, manifest/cache correspondence, all new controls, complete
primary Go sources and old proof bytes. All eleven source epochs preserve full
prelaunch paths/bytes/modes and exact outputs, distinguishing formatter mutations.
No source/root/type/error/proof scope, review rule/seal/exclusion, premise, math
rule or kernel budget changes. Host checks make no native Linux release claim.
See the [verification record](verification.json) for selection and open work.

Primary references are the complete pinned Go1.25.0 sources retained by the
native capture, the complete vendored modfile parser retained by each source
epoch, and the [Go module reference](https://go.dev/ref/mod#go-mod-file-replace).
The native comparison exercises real offline Go resolution, including exact and
wildcard local replacement directives and a versioned replacement with an
uppercase module path. It executes neither application initialization nor the
unused unsupported HTTP body.

The complete56-family foundation/all169 demands, independent module/compiler/
root/capture/context/descriptor/registry and launcher authority, source/IR/state/
storage/frame/callee/program proofs, native Linux gates and remaining T02–T11
stay due. R01/R04/R05/R06/W01 remain open. The cumulative T02 gate remains final
W04: `./scripts/check-fast.sh`. Go/Rust only.
