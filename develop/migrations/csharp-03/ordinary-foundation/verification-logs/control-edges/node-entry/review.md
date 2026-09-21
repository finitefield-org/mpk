# Node-entry relation review

Each original native block receives a separate entry relation. Its arguments
contain the current execution's entry slots/phis and, for each original incoming
W04 edge, a selection flag, the exact guard inputs and an edge-specific target
snapshot. The ordinary body requires exactly one selected edge, requires that
edge's actual guard, and compares only its snapshot with the fresh entry.
It does not simultaneously equate states from different loop iterations or
require two alternative predecessors to agree. No selection or multiple
selection is false. Unknown incoming guards leave the whole node definition
pending rather than dropping that predecessor.

Assignedness is compared separately from values. Unassigned slot payloads are
irrelevant; assigned slots retain their full physical representation, including
nullable presence and array snapshots. Native phi values always compare in
full and keep their exact validated IDs/types. Function entry initialization,
source-exit production, native execution, reachability and the actual choice of
incoming edge remain separate obligations. This is an open relation, not a
proof that an execution follows any chosen edge.

The 17 original source contexts contain 617 node entries and pass 13,576 new
observations. Tests reconstruct node/edge/slot/phi bindings from original VIR,
exercise each incoming edge with independently evaluated guards, accept nonzero
selected states while other predecessors contain different values, reject
missing/double selection and individual assignedness/payload/phi corruption,
and distinguish unassigned slot contents from native phis. Metadata mutations
reject changing node/edge identity, argument offsets, rules, definitions or
removing entry records. All contexts have at least one incoming edge per node;
no general reachability claim is inferred from this corpus.

Five historical preservation tests verify unchanged existing definitions; the
immediate predecessor's entire term/declaration tables remain exact prefixes.
The old state-rule metadata changes only from merge-pending to merge-separate.
All nine slot certificates replay unchanged because entry relations are appended
in the public edge generator, after the private builder used by slots. Four
other source modules only re-export the new metadata types, with reversible
source-hash checks. No new consumer path or checker/core rule is introduced.

Review additionally bounded slot/phi and incoming-argument expansion before
cloning metadata. The bound correction regenerates all 17 candidate programs
exactly. Clippy, formatting and the affected preservation/dependency checks
pass. The 14 distinct byte sequences covering these 17 contexts pass both
unchanged checkers with matching reports, zero axioms and hash-corruption
rejection. Equivalent-context evidence is reused only after exact byte identity.
The previous 38,950 edge observations remain supported by exact declaration
preservation; adding entry observations gives 52,526 scoped observations.

W04 logical observation mapping, non-transfer memory effects, source-exit
snapshots, exception entry/search/finally and complete native/application proof
composition remain unfinished. Unit 5 and W09 remain incomplete. The full gate
is deferred to T06-W12; no component-only commit is made.
