# Original versioned module snapshot checkpoint

One already selected module identity binds its original `.mod`, `.info`,
`.ziphash` and `.zip` bytes beneath an explicit download-root capability.
Official module path/version escaping preserves uppercase names, major suffixes
and legacy versions. Noncanonical root-directory aliases reject. Go retains
responsibility for actual version selection/replacements; this is no MVS
implementation and does not authorize a caller's claimed graph.

Both manifest and ZIP h1 checks use a separately reconstructed original captured
`go.sum` index. A caller's mutable parsed map cannot change that authority.
The shared reconstruction is factored for archive and full snapshot consumers.
Go info receives strict JSON/duplicate checks and Go-compatible Version/Time/
Origin field decoding. Unknown Go cache fields retain opaque original bytes;
none grants MPK artifact or VCS authority. ZIP hash metadata is compared after
Go-compatible whitespace trimming while its exact original bytes remain stored.

A ZIP root `go.mod`, when present, must equal the separately captured/checksummed
`.mod` bytes. Individually valid but inconsistent manifest/ZIP checksums reject.
Legacy ZIPs without `go.mod` retain their existing external `.mod`; no source or
archive is rewritten to manufacture a manifest. Missing files, stale checksums,
wrong info versions, unavailable compiler patches and changed hash metadata fail
and prevent sealing. Verification never fills missing snapshots or contacts a
proxy/VCS/generation service.

The [results](verification.json) pass9 selected roots/34 subtests on exact
Go1.25.0 in2.267 seconds, vet in0.415 seconds and format in0.006 seconds. They
cover all new metadata capture, escaped paths, legacy preservation and meaningful
missing/stale/inconsistent inputs, plus both affected existing archive-capture
consumers and the independent manifest-h1 vector. Five legacy development roots/
20 subtests pass in2.187 seconds; they add no supported practical compiler.
Unchanged pure ZIP/native-Go checksum tests retain their preceding evidence and
are not repeated. No C# test or intermediate full gate executes.

W01 stays In progress. Actual selected-graph/replacement ownership, every
consulted manifest/metadata and original package source/export/build closure,
complete checked private context/foundation/compiler inventories and immutable
production loader materialization remain required. No source/program proof,
public profile activation or native Linux acceptance is claimed. The cumulative
T02 gate remains deferred to W04 (`check-fast.sh --skip-csharp`).
