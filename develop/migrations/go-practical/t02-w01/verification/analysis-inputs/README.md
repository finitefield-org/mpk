# Original analysis inputs and reproduced frontend builds

The private analysis capture binds the frontend's original Go1.23.0 module and
existing exact dependencies: x/mod v0.27.0, x/sync v0.16.0 and x/tools v0.36.0.
Changed requirements/checksums/indirect roles, local replacements, crossed vendor
package ownership and unmodeled compiler inputs reject. All three original
versioned snapshots are read through explicit rooted capabilities and checked
against the original sums and module ZIP provenance.

The [separate provisioning record](provision/result.json) downloads only those
fixed versions through the official Go proxy using a copy of the existing
original go.sum; neither copied manifest/sum changes. The exact original
[offline fixtures](../../../../../../go-tools/go2vir/testdata/practical-analysis-cache/PROVENANCE.json)
retain their raw hashes/provenance. Capture and all later builds run offline,
with no VCS/workspace/toolchain fallback or source generation.

All126 vendored Go sources and six LICENSE/PATENTS files match their original
archive members. The [initial comparison](provision/vendor-comparison-initial.json)
finds one existing uncompiled SSA TODO with one omitted final LF. Actual bytes
remain unchanged and both original/actual hashes and lengths enter the explicit
auxiliary-difference inventory. No input is normalized or rewritten. Unknown
compiler assets and embedded/assembler inputs reject; source directive checking
parses comments beyond imports and preserves ordinary directive-like strings.

The [owned input inventory](final/selected/captured-inputs.json) retains169 build
inputs:33 frontend Go files,133 vendor files and the module/sum/vendor manifest.
The complete sealed snapshot has181 original source objects including module
provenance. The [two fresh builds](final/selected/reproduced-builds.json) use only
those owned buffers, different physical roots, independent empty compiler caches
and the exact prepared compiler image. Full binary bytes agree, and linked
module/build metadata matches the pinned analysis libraries. Compiler and
frontend consumers share the rooted regular/nonblocking, streamed hash and
same-file binary reader.

The [independent observer](independent-build/result.json) recomputes all source
object IDs, every module/manifest h1 and original vendor-member correspondence.
It copies the exact5,577-file prepared compiler inventory and rebuilds with a
third empty cache. All three8,592,898-byte binaries have the same full SHA256 in
the results. This proves the observed development source/build correspondence;
metadata alone is insufficient and no registered practical frontend is claimed.

The initial selection passes10 roots/67 subtests but stops the reproduction root
after its first successful offline build: its private output path includes the
macOS /var alias and root authorization correctly rejects it. The original
[failed result](selected/result.json), raw logs and exact inputs remain. The
test-only fix selects the normalized physical output and retains rejection of
the original alias. The full affected reproduction root passes in38.872 seconds;
seven legacy roots/41 subtests and the ten unaffected initial roots/67 subtests
retain passing evidence without repetition. Current vet/format pass. The
independent third build/observation passes in17.828 seconds. Existing compiler
consumers and their11-case matrix cover the shared reader refactor.

The [changed-input review](input-review/run-review.py) scans every change, raw
failure and portable fixture with the unchanged strict scanner/historical seal,
whitespace and local-link policy. No source/JSON/directory exception is added. The original Go build-info CLI
output retains four terminal tabs for empty checksums. Its exact645-byte hash
and four literal lines are recorded as an original formatter observation; the
raw file and first failed input review remain unchanged. No blanket whitespace
exclusion or scanner change is made.
No C# test or intermediate full gate executes.

Complete actual ordinary foundation and checked private registry/context binding,
application all-consulted dependency/source/export/build closure and authoritative
immutable replay remain due. No T01 recipe fixture is promoted or relabeled.
W01 stays In progress; all169 Go application demands remain unelaborated. No
application proof, public activation or native Linux acceptance is claimed. The
cumulative T02 gate remains deferred to W04 (`check-fast.sh --skip-csharp`).
