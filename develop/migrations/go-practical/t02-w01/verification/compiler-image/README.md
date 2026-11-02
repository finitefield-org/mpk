# Prepared Go compiler and standard-library image checkpoint

The private validator checks a complete supplied prepared image and its original
root/component inventories, partition and content digests. It requires all nine
Go1.25.0 command/tool binaries and one target-content component, the four original
registered specification/version witnesses and original Linux-amd64 runtime and
standard-library sources. The exact prepared identity must still be bound to a
checked private practical context before production use.

Each tool is opened through the rooted regular/nonblocking reader with original
ancestor checks. Streamed hashes and same-file observations bind its decoded
build information: exact Go1.25.0, actual cmd/tool identity, allowed host, gc/exe,
cgo disabled, no experimental build and AMD64-v1 for Linux host binaries. The
retained image owns every root/component inventory and identity projection, and
can revalidate the original image before later consumption.

The [results](verification.json) cover the actual Go1.25.0 Darwin distribution
prepared using the existing release assembler pruning/mode rules:5,577 files,
163,457,770 bytes,10 components,9 binaries and715 original Go source directories.
Original implementation/support bytes remain unchanged; Go's own tests/testdata
are outside this prepared compiler closure and generation scripts are inert
content. The [complete fresh inventory](captured/selected/prepared-image.json)
is emitted exclusively before caller-mutation controls. An
[independent observer](original-observation/result.json) agrees on every original
SDK file length/hash, root/component canonical identity, partition and all four
registered source witnesses.

Self-consistent newer release labels, mixed hosts, rehashed changed version
sources and a fully rehashed gofmt binary substituted for compile reject. Caller
array mutations leave owned inventories valid. The previous partial SDK fixture
rejects as incomplete. An eleven-case pure build-info matrix checks crossed
release/tool/host/tuning/cgo/compiler/buildmode/experiment and duplicate settings;
these metadata controls make no Linux execution claim.

The initial native attempt stops during fixture preparation because the fixture
sealed directories while leaving file modes writable. Exact initial inputs and
raw failure remain in [the original result](selected/result.json). The test-only
fix passes the entire native root/six controls in7.770 seconds. Review adds the
explicit AMD64-v1 requirement: both affected roots/17 subtests pass in8.141
seconds, with the new metadata matrix passing on the separate legacy compiler.
Final exclusive inventory capture passes its native root/six controls in7.594
seconds; current vet and format pass. Five unaffected initial roots/11 subtests,
initial legacy5 roots/11 subtests and reviewed legacy1 root/11 subtests retain
their passing evidence. No unaffected suite is repeated.

The [changed-input review](input-review/run-review.py) applies the unchanged
strict scanner, exact historical policy, whitespace and local-link rules to all
changes, including the full inventory and original failure. No C# test or
intermediate whole-project gate executes.

This is prepared compiler-image validation. Complete analysis-library/frontend
build-source identity, actual checked foundation/registry/context, all consulted
module/source/export/build inputs and immutable production loading remain due.
The existing generic image inventory is not relabeled as the W03 common artifact.
W01 stays In progress; no application proof, public activation or native Linux
release receipt is claimed. The cumulative T02 gate stays deferred to W04
(`check-fast.sh --skip-csharp`).
