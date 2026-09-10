# W09 unit 3 integrated-program review checkpoint

This review covers shared storage/emission, the integrated ordinary program
and actual-source/composed tests. It is not a unit-3/W09 completion receipt.

Standalone programs each owned a Builder and common iteration/storage
declarations. Combining them would duplicate globals and multiply transformer
cost. The integrated generator now uses one Builder and explicit caches.
Duplicate-name rejection remains unchanged; each cached storage shape must
equal the requested concrete carrier. All nine component generation/import/
pinned-replay tests confirm existing standalone emission bytes are preserved.

Review checked carrier reconstruction, internal construction state, source fields
and active sum payloads, finite dependencies, cache ownership, expanded-operation
IDs, deferred Money/Transition operations, construction ownership requirements,
canonical import equality and limits. All twelve template families occur in
the current 42-source corpus. Source occurrences remain distinct even when contexts
generate identical carrier bytes. Every metadata definition resolves within its
certificate. The nested construction/sequence/domain regression detects the
original aggregate transformer-budget problem.

The initial composed test omitted the mandatory Option arm and failed to compile;
it now explicitly constructs None. The next run passed five source contexts and
nineteen observations. Scoped lint found an unnecessary Box replacement in the
source-field mutation; replacing the boxed value in place fixed it, and scoped
lint passed. The final post-fix semantic run also passed all five sources and nineteen
observations (9.56 seconds).

The component coverage audit found one missing original source,
nullable-string-default. It is now the 42nd integrated source, and all original
41 byte files and metadata rows remain unchanged. All 42 passed generation,
metadata/certificate import mutations, exact pinned replay and same-byte dual
checking with zero axioms and hash-corruption rejection.

The additional structural comparison covers all 112 standalone source-component
pairs across eight families. It checks 1,132 root definitions and 10,500
transitive declaration occurrences. Numerical IDs and term DAG sharing may
differ; declaration kinds/reducibility, bodies/types, global names, variable
indices, application order and universe levels must agree. Root and dependency
body mutations reject. This establishes preservation of the checked component
definitions within the combined program without treating observed values as
universal proofs. Scoped lint and all five inventory tests also passed.

No further actionable issue was found in this integration inspection. This does not
substitute observations or helper type checking for universal application proof
obligations. Pending deep-domain/collection checks and full unit review still
prevent closure. See `unit-3-structural-foundation-progress.json`.
