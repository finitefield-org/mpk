# Source entry and transfer integration — scoped review

The public ordinary control certificate now contains each source function's
entry initialization and successful load/store/pattern-bind relations. The
standalone slot generator and the public generator share `emit_function`, which
reconstructs represented slot types and checks original source-anchor endpoints
and result identities. The public generator passes its exact memory bindings,
checks that the original flow and nullable overrides agree, and appends the
relations after the existing native definitions. The private edge builder does
not append these relations; standalone source-slot pins remain stable.

Entry assigns exactly the original parameter slots and connects their values to
original SSA inputs. Other slots remain unassigned without an invented zero
payload. A store/pattern bind assigns its target to the exact SSA value and
preserves other assigned values and flags. A load requires assignedness and
agreement with the source SSA value. Nullable payload imports preserve Some
presence. Array transfers use the current native memory SSA and the existing
source snapshot relation, rather than equating private and public storage.
These are successful source transfers; exception execution remains separate.

The eighteen-source test compares complete slot metadata with standalone
generation and compares every standalone declaration and dependency modulo
canonical term/global numbering. It executes each integrated relation with a
consistent input, checks every assignedness argument, tests constrained versus
unassigned payload changes, and rejects altered SSA results (array logical length
rather than irrelevant inactive private storage). It covers 18 entry relations
and 207 transfers: 114 loads, 88 stores, five pattern binds. All 6,342 new runtime
observations pass. Existing nullable and nonzero scalar slot observations and
array boundary/current-memory tests replay through the standalone generator;
its nine pinned contexts remain unchanged.

Production review checked that the shared emitter retains the prior argument
order, definition identities and code paths; the existing relation body did not
change. Original flow and source metadata are preserved, and appended relations
retain exact entry/exit anchors. The added source graph and memory membership
checks fail closed. Prior public certificate declarations and metadata must
remain exact except for the new `slot_relations` field. The metadata alone is
not trusted: importer reconstruction and certificate bytes still must match.

The first lint run found duplicated alternatives in the preservation test's
historical-folder list. They were removed; the corrected lint and rebuilt tests
pass. The rebuilt generator reproduces all eighteen candidate pairs exactly.
The obsolete filter `control_slot_nullable_transport` matched no test in the
combined command; nullable coverage was actually executed by
`control_slot_source_relations`, including the type fixture. The actual test
count is three, with the ninth count/fill pin checked in the preservation run.

Fresh same-byte Go/Rust checks are required for all fifteen distinct certificate
byte sequences before promotion. Three duplicate contexts may reuse evidence
only by exact byte equality. Earlier execution observations remain applicable
only after the term/declaration and metadata preservation tests pass. The
whole gate stays at T06-W12. This change provides entry and transfer relations
in the same owning certificate; actual execution/selection witnesses, literal
values, exceptional flow, alias frames, loop invariants and native/application
proof composition remain open. No unit completion or component-only commit is
claimed.
