# Actual pinned offline Go module discovery checkpoint

The preparation discovery command runs the actual selected Go1.25.0 compiler:
`go list -mod=readonly -m -json all`. Its exact executable hash is checked before
and after execution. The existing closed loader environment pins Linux/amd64,
disables proxy/sumdb/VCS/workspace/toolchain fallback and gives private scratch,
home and build caches. Bounded stdout/stderr cancel on overflow, with a120-second
deadline. The original module-cache path is passed through a value-copy environment;
private cleanup never changes permissions on that original cache.

The strict streamed result binds one original main module and every selected
versioned dependency to captured original manifests, checksums and complete
versioned snapshots. Local and versioned replacements must agree with original
main directives and authorized roots. Go selects versions; directive matching
only checks ownership with exact-version precedence over path-wide replacements.
Original output bytes remain owned independently. Errors, ambiguous or altered
module identities/paths/languages/checksums, chained replacements and unknown
result fields reject.

The [results](verification.json) record an initial11-root31-subtest selection in
3.819 seconds. Actual Go selects the transitive c@v1.1.0 upgrade, agrees with
local exact-before-wildcard and versioned replacements, and fails on unavailable
offline inputs without acquiring source. Original application bytes include an
unused map function and remain unchanged; this command makes no package typing
or selected-body acceptance claim. Initial legacy7-root21-subtest checks, vet
and format also pass.

Review reproduces a missing checksum-owner check: another captured input could
supply valid checksums while the original main go.sum was empty. The original
[failed regression](review/main-checksum-owner/before-fix/result.json), raw logs
and exact source inputs remain retained. The fix requires the exact original
main-module go.sum object before checksum reconstruction. All4 affected binder
consumers/10 subtests pass in2.629 seconds; legacy7 roots/21 subtests, vet and
format pass. Unchanged stream/output/offline-failure/public-boundary tests retain
the initial evidence. The [changed-input review](input-review/run-review.py)
inventories all changes and applies the unchanged strict scanner and whitespace
rules.

This result is a preparation observation from the original mutable namespace.
It grants no immutable replay, complete consulted-version/metadata closure,
registered practical context/compiler/foundation or source-program proof.
The existing test toolchain fixture is not the complete practical standard-library
and analysis inventory. W01 stays In progress; original source/export loading,
sealed replay and complete authority remain due. No public profile or native
Linux acceptance is claimed. No C# test or intermediate full gate executes;
the cumulative T02 gate remains deferred to W04 (`check-fast.sh --skip-csharp`).
