# Go analysis frontend build information: in progress

The private installed compiler owner now independently reads the selected Go
frontend executable and checks its Go version, host, main module, complete
analysis dependency set and build settings before binding the SDK. The Rust
validator follows the existing native Go rules, including duplicate-setting and
dependency rejection, exact versions and optional exact module sums. This
checks build metadata, not the relationship between a source tree and binary;
it does not register a practical context or prove source/program semantics.

The exact registered 9,004,378-byte frontend and all original prelaunch inputs
are [retained](prelaunch/result.json). A full [source-generation comparison](generation-diff.json)
compares all 242 files of the registered source epoch with all 318 files in the
current prelaunch tree: 233 have equal bytes, nine differ and 76 are added.
Twenty-three changed or added files are non-test Go files. This comparison does
not assert which files were compiled. Matching build metadata alone must not be
used to attribute the old registered binary to the current source generation.

The pinned-Go [native observations](native/result.json) include the actual
registered binary and all 38 complete controls: four accepted and 34 rejected.
The affected Rust [tests](vc-selected/result.json) independently compare all
observations and run the existing compiler build-info, ELF and complete original
SDK input tests; all ten test roots pass. The full SDK owner, all 5,577 files,
182 packages, 5,697 original source objects, one Root selection and zero
contract overlays remain. The CLI's [default full-snapshot check](whole-snapshot-host/result.json),
both affected Clippy checks and the changed-code format and Go vet checks pass.
Their selection reasons and original inputs are recorded in their receipts.

The first direct version probe used a Rust image without Go; its available tool
transcript is [retained](initial-version-probes.json). The initial host-format
attempt could not spawn the missing temporary Darwin tool; the visible failure
and its limits are recorded. A second formatter attempt failed because its
single-file mount made the temporary sibling read-only. The [corrected attempt](go-format-linux-directory/result.json)
uses the registered SDK and confirms every other package file is unchanged.

The changed Linux test executable also passes the actual four-bundle
[installed-root checks](../installed-compiler/installed-qemu-analysis-inspection.json)
under QEMU user-mode emulation. The original SDK/source owner binds, and the
wrong-host, crossed-owner, changed-SDK, missing-Rust-member and independently
valid alternate-registry controls reject. All original inputs and the installed
bundles remain unchanged after the controls. Local emulation does not provide
native Linux release acceptance. W01, its remaining context/capture/foundation
and source/program requirements, and the practical profile remain open. The
whole T02 gate `./scripts/check-fast.sh` is deferred to final T02-W04.
