# Original module input checkpoint

Original `go.mod` and optional `go.sum` are captured through the bound-root
reader. The parser retains every byte and original require/exclude/retract/
replacement directive. It checks full compiler patch requirements before
normalizing language minors. Main proof languages are go1.22–go1.25;
type-check-only dependencies may predate that table. Dependency toolchain
suggestions cannot select the main toolchain. Unmodeled directive families
remain rejected, and root workspaces/vendor inputs remain excluded.

The checksum parser retains original bytes, requires canonical module versions
and SHA256 h1 payloads, permits identical duplicate lines and rejects conflicting
ones. `go.mod` verification uses Go's one-member Hash1 summary. Its independent
known vector is the actual Go1.25 observation retained in T01-W02. Empty sums,
archive-only sums, wrong module/version and even an added newline cannot satisfy
the required manifest identity. ZIP/archive checks remain due.

Each original local replacement directive maps into the most specific explicitly
bound root. Component-prefix escape, unbound roots, links, ambiguous equally
specific roots and detached owner/directive metadata reject. Reparse captured
original bytes before following a replacement, so mutating a returned parser
tree cannot redirect reads. Broader duplicate roots cannot affect a more
specific binding through map iteration order. Go still owns version selection.

The [recorded results](verification.json) pass12 root tests/52 subtests on pinned
Go1.25.0 in2.217 seconds, vet in0.461 seconds and format in0.006 seconds. An
actual offline `go list -mod=readonly -m -json all` compares original nested-main
and local-dependency manifests, version, replacement path and effective module
versions. All4 original buffers, including empty `go.sum` and the complete Go
file with an unused unsupported function, survive sealing unchanged. This is
a module-resolution differential; it does not load/type-check packages or
materialize a production immutable loader namespace.

The separate Go1.23 development build passes7 root tests/40 subtests in2.181
seconds, covering the four pure parser/checksum tests, closed public arguments
and unavailable rooted capture. No supported practical compiler/target is added.
Both routes use vendored inputs and isolated caches, with downloads and
fallback disabled. No C# tests or intermediate full gate execute.

W01 stays In progress. The checked private context/foundation, compiler/library
inventory, versioned offline archives/content, complete selected Go graph,
immutable loader namespace and source/export metadata remain required. The
cumulative T02 gate remains deferred to W04 (`check-fast.sh --skip-csharp`).
