# T06-W09 internal unit 5: normal source execution checkpoint

The new execution record closes the normal transport gap left by the local
source-step checkpoint. For each complete step and each non-exception outgoing
edge it conjoins the selected source node entry, exact local step, outgoing
guard, edge join, optional phi join, and target node entry. It also requires the
chosen target edge selector to be true and every other incoming selector to be
false.

State bridges compare assignedness exactly and compare payload storage whenever
the slot is assigned. An unassigned payload remains irrelevant, matching the
existing slot semantics. Shared bridge and selector definitions avoid duplicate
large terms. If the complete conjunction would exceed the 256-binder limit, all
indexed components remain authoritative and the optional compact definition is
omitted.

The full argument/component maps are reconstructed in memory from retained
entry, step and edge records. Repeating them for every edge made the largest
canonical JSON exceed the frozen 16 MiB importer limit. Canonical metadata now
stores the exact argument and component counts plus a SHA-256 commitment to the
complete map. The importer regenerates the whole program from validated VIR and
requires byte-identical metadata and certificate bytes. Tests mutate all ten
serialized execution fields and remove each owning collection; every mutation
rejects.

The 18 original contexts contain 625 complete normal edge executions. Tests
independently reconstruct the expected `(source step, edge)` set, verify all
identities, definitions, argument indices and component roles, and execute the
shared bridge and edge-selector truth tables. Three separately generated
36-file candidates and the promoted fixture are byte-identical. The previous
source-step fixture is archived under `before-source-executions`.

All 15 distinct certificate byte sequences pass the unchanged Go and Rust
checkers. Their module, declaration, axiom and report hashes agree, axiom count
is zero, and both checkers reject each one-bit hash corruption. The previous
579-step metadata and declaration-preservation tests, targeted clippy, format,
and all five inventory tests also pass.

This checkpoint establishes normal entry selection and inter-step transport for
complete local steps. It does not establish function entry initialization,
exceptional or alias execution, reachability of the ten unreachable nodes, the
remaining non-data call, or complete application proof assembly. Unit 5 and W09
remain incomplete, W10 remains blocked, and the whole T06 gate stays deferred
to W12.
