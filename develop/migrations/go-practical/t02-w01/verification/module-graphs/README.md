# Original Go module graph inputs

The [controlled graph collector](../../../../../../go-tools/go2vir/practical_module_graph.go)
runs only the fixed Go1.25 `mod graph` command under the existing offline
environment. It binds complete stdout/stderr to the actual original module-list
observation, compiler, distribution, main manifest and cache/root locations.
Caller graphs, selectors, compiler commands and source adapters are not inputs.
`mod graph` does not accept `-mod`; original input checksums and namespace/byte
revalidation remain required.

Original go.mod inputs are retained for both the initial main requirements and
the final native graph. Dependency summaries use Go's lax parsing disposition,
preserving ignored future directives in unselected old versions. The traversal
tracks identity and incoming pruning state, so old dependencies can unprune
modern dependencies and cycles terminate. It keeps original zero-requirement
leaves even when the native output has no row for them. Unselected graph-only
versions need their original checksummed .mod, without invented ZIP/info/source
trees. Pruned targets keep their native incoming edges without requiring a .mod
that Go does not read. Exact main replacements and exclusions remain bound.
Virtual go/toolchain nodes, including original default/custom suggestions, have
no fabricated source object.

The collector reconstructs the complete native breadth-first output, retaining
edge order and duplicates, and checks its maximum real-module versions against
the original module list. Final roots still come from native Go. This is a
preparation consistency check; independent Rust ParseLax, root expansion and MVS
remain due. The initial/final inventories are not a complete filesystem-access
trace of every intermediate native root-expansion attempt.

Immutable image consumption reconstructs these summaries from owned original
bytes. Mutable preparation parser views cannot replace them. The
[graph handoff](../../../../../../go-tools/go2vir/practical_module_graph_inputs.go)
retains both inventories, whole original diagnostics and an acyclic hash link
to the existing original module/source/export packet under one owner lock.
Returned buffers remain independent after original directories, the image and
the owner retire. The complete HTTP owner retains181 packages/180 exports and
5,667 original objects, including unused unsupported bodies and unexecuted init.

All five new native test roots and15 affected existing roots pass. They cover
unselected old/empty manifests, cycles, duplicate requirements, exclusions,
pruning, exact-before-wildcard local replacements, versioned cache escaping,
complete owner retirement, command selectors, changed graph observations and
four coherently changed edge streams. Default/exact/custom virtual toolchain
cases retain every original suggestion. Final vet and changed-file format checks
pass; the existing Go1.23.2 compiler also compiles the retained frontend. Rust
runtime/test sources are unchanged and are not claimed newly replayed.

The initial graph test epoch remains failed: an observation filename collision
prevented one replacement record, and an inconsistent original root set was
incorrectly expected to pass the existing readonly module-list command. The
corrected test retains that same case as an original Go rejection and records
the full native diagnostics and the separate native expanded graph; it never
acquires an accepted owner. The initial failed child diagnostics/generated tree
were not separately serialized before that failure; its complete prelaunch
generator/source epoch and exact parent output remain. Corrected epochs preserve
every generated original file before either command. The successfully checked
full HTTP owner is retained without an unnecessary rerun. Subsequent virtual-
toolchain changes are limited to explicit toolchain roots absent from that owner.

All17 source epochs, formatter preimages and exact outputs remain. Complete
Go1.25 SDK originals (14,496 files/202,722,007 bytes) precede their consumers;
the final legacy compile additionally retains its actual complete Go1.23.2 SDK
(13,260 files/236,047,459 bytes). The earlier legacy compile retained its tool
identity and repository inputs with the Go1.25 SDK; it is superseded by that
complete final legacy epoch. [Independent inspection](inspection.json) checks
whole source-object identities, transport hashes, complete owner membership,
all native controls and unchanged32 proof byte streams. The latter are inspected
from previous evidence and are not claimed newly replayed. The
[verification record](verification.json) distinguishes these scoped results.

General independent graph/MVS and installed compiler/root/capture/context/
descriptor/registry/launcher authority, source/IR/state/storage/frame/callee/
program proofs, all56 foundation families/all169 demands, native Linux release
gates and remaining T02–T11 stay due. R01/R04/R05/R06/W01 remain open; the practical
profile remains inactive. The cumulative T02 gate is deferred to final W04:
`./scripts/check-fast.sh`. Scope remains Go/Rust only; no source/type/error/proof
universe, review rule, premise, math rule or kernel cap is changed.
