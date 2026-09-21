# Native construction storage and ownership-use review

Six captured original sources reconstruct 21 invoked definitions and 26 SSA use
points. Frozen signatures/check order, original subjects, guards, successors,
exception values and source identities are retained. The complete observation
uses its exact Data/SequenceOwnership signature. Helpers still emit every frozen
construction operation, and five predecessor certificates remain byte-identical.

At the initial storage checkpoint, ownership had no storage predicate. Its
failure entry was None, and every dependent formula was retained exactly in
pending_predicates. Nine ownership records and 183 formula roles were unresolved. Removing those records/formulas
or changing metadata/certificate bytes rejects on independent regeneration.
The first coverage assertion correctly rejected a source set without complete;
a real captured repeated-string-assignment context closed that test gap.

Physical result equality covers every bit. Fill/rewrite use recursive index-path
comparison; each untouched sibling is compared in full, the selected leaf equals
the replacement, and the initialization bit is true. Length is unchanged. All
result length/bitmap padding and the fourth product role are zero. Nonzero index
bits 14..31 preserve original cells/bitmap rather than wrapping. Exact zero
predicates are an ordinary Boolean optimization, not a sparse-data assumption.
No evaluator or checker acceptance rule changes.

Runtime testing passed 198 executed observations across two versions (178 before
the final freeze optimization, then 20 targeted final freeze cases), including nonzero/wrong results, full
word failure arguments, 16384/16385 construction bounds, publication bounds,
uninitialized/repeated/incomplete writes, index paths 0/1/8192/16383, high/negative
indices, corrupted untouched cells, selected-cell bits, role/field padding, and
wide string-child physical patterns. Representation/public domains are separate
obligations; arbitrary physical-pattern probes do not assert valid strings.

Only three frozen result relation bodies change in the final optimization.
A term/global-ID-independent comparison proves all other preexisting definition
types and bodies unchanged, permitting reuse of unaffected runtime results.
The first 4096 physical cells are preserved, high private indices are excluded,
and all published length-field padding remains zero.

All six final certificates pass identical-byte Rust/Go checking with zero axioms
and corrupted-hash rejection. Affected lint/format and inventory pass. The first
300-second runtime timeout is retained as a non-verdict diagnostic; it is not
counted as verification. Direct review found no remaining scoped storage-adapter
defect. Ownership, native CFG/application proofs and W09 completion remain open.


The source-ownership extension now combines storage definitions and symbolic
ownership equations/proofs in one Builder. It shares the existing C5 helpers,
retains the complete frozen equality foundation, and obeys the existing limits.
All fourteen standalone ownership proof candidates stayed byte-identical during
that builder extraction. Later many-origin proof optimizations preserve all six
construction certificates, as verified by exact regeneration.

Each of the eighteen original ownership checks requires a matching function,
node, first operand/receiver and symbolic invocation state. The scoped failure
keeps the original operation signature and subject order. Its theorem quantifies
over those data arguments and retains the matching checked flow theorem in a
Let before returning the exact receiver theorem. Its generic failure constant
is registered only while compiling that source use and removed immediately
afterward; the generic definition's failure entry stays None. All 183 formerly
pending formula roles are now defined at their original sites.

Import tests reject changed node, receiver, flow theorem or scoped-definition
bindings, removed ownership proof metadata, changed application-pending status,
and changed certificate bytes. Current runtime checks pass 220 observations,
including false ownership failures and true static goals for all eighteen
bindings. Every one of the six changed certificates passes both unchanged
checkers with matching reports, zero axioms and corrupted-hash rejection.
Affected lint, format and inventory pass. The registered Std consumer count
increases by exactly this one source file (133 to 134).

No remaining defect was found in this scoped binding change. Nine original
ownership records, native execution/control and application proof assembly are
still pending. The explicit application_scope_pending flag stays true; unit 5
and W09 stay open. Evidence is in
`verification-logs/construction-data/source-ownership/verification.json`.

## Symbolic ownership records

Reviewed record reconstruction against original function/node/actions and the
validated symbolic trace. The selected conjunction includes local actor checks,
invocation transition, applicable terminal/cleanup coverage, and every incoming
edge transition, cleanup, phi and join, including delayed backedges. Normal
admission is included only where the source flow requires it. A missing
selected equation rejects generation. The proof normalizer produces ordinary
equality terms and retains the complete checked flow theorem in a typed let.
Concrete before/after states are not silently interpreted as symbolic states.

Exact certificate-prefix comparison confirms the previous 220 semantic
observations still describe unchanged definitions. Nine added record predicates
and 36 repeated use checks pass. Five changed data certificates and three added loop/finally certificates now
pass both unchanged checkers, report agreement and hash-corruption rejection;
the unchanged return-array certificate retains its accepted bytes. The added
control sources execute eight record predicates, including loop and finally
flows. Exact regeneration and mutation checks pass for all nine contexts. No
remaining defect was found within this scoped record linkage. No unit completion or
component-only commit is claimed.
