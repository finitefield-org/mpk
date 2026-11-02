# Original module archive checkpoint

Original ZIP bytes use the final `auxiliary` source kind and retain their full
raw SHA256/source identity. Go h1 separately binds member names and contents;
compression, order, timestamps and ZIP attributes may change raw identity while
preserving h1. The unchanged `x/mod` v0.27.0 Hash1 source is added to the existing
vendor inventory after its module archive matches repository `go.sum`. The
existing BSD license is retained. No module version, go.mod or go.sum changes.

The memory-only parser checks canonical module/version prefixes and Go paths,
exact root manifest spelling, Unicode simple-fold collisions including ancestors,
file/directory conflicts and duplicate files. A component trie avoids recursive
parent checks and quadratic parent-string copies. It validates original member
sizes and regular-member CRCs under Go's ZIP reader, plus every directory header.
Zero-uncompressed directory metadata retains Go's compatibility behavior. The
Go500MiB content and16MiB manifest/license protocol bounds remain distinct from
the captured raw16MiB file limit. Valid empty ZIPs and opaque unselected assets
remain input data; they are not silently promoted to logical source candidates.

The source inventory retains every regular member's complete length and SHA256.
Captured file bytes can be checked against that inventory; returned metadata is
copied. This is content linkage, not independent package/module/root authority.
Archive checksum ownership requires an actual captured original `go.sum`. A
separate original-byte reconstruction is cached once, so mutating a caller's
parsed checksum map cannot authorize a new archive. Archive namespace observations
aggregate across capabilities, count implicit parents and repeated/root headers,
and do not double-charge repeated reads. Ignored failures prevent sealing.

The [results](verification.json) include the independent Go1.25 h1 vector from
T01-W02 and a fresh actual pinned offline `go mod verify`: original files pass,
a changed dependency source fails in both Go and MPK. Main `go.mod`/`go.sum`
remain unchanged. This is checksum verification on a prepared original fixture,
not original-package type checking, production loader materialization or a native
Linux isolation receipt. No generated application body or initializer executes.

The initial affected-boundary selection passes13 roots/46 subtests in2.251
seconds; the legacy development selection passes9 roots/28 subtests in2.040
seconds. Review then identifies a shadowed directory payload not opened by Go
HashZip's last-name lookup. The first [focused attempt](initial-shadowed-payload/result.json)
stops during invalid-directory fixture creation. Correcting the raw fixture
[reproduces the actual regression](shadowed-payload-regression/result.json).
All inputs/logs remain retained. The fix validates every original directory
header before hashing. All9 affected archive roots/25 subtests pass in1.642
seconds and six pure legacy roots/17 subtests pass in1.936 seconds. Current vet
and format pass. Unaffected old boundaries retain their earlier passing results.

Shared-budget tests use real small ZIP files and seeded remaining counters;
they do not claim full-size stress acceptance. No C# tests or intermediate full
gate run. W01 remains In progress: complete private context/foundation and
compiler inventory, versioned module metadata/root-manifest linkage, actual
selected graph and immutable source/export loader authority remain required.
The cumulative T02 gate remains deferred to W04 (`check-fast.sh --skip-csharp`).
