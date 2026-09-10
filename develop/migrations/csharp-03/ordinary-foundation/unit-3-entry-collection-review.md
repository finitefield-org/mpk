# W09 unit 3 entry/map/set review checkpoint

This is an in-progress component review, not a clean unit 3/W09 completion
receipt. The reviewed scope is the entry and collection generators, exports,
source captures, test additions and exact generated metadata/certificates.
Verification status and hashes are in `unit-3-entry-collection-progress.json`.

Review found and corrected four collection implementation issues:

1. Direct dependency IDs in VIR are sorted and deduplicated. Treating their
   positions as template dependency indices rejected a real integer-map source.
   Each required dependency is now matched by exact template and arguments.
   The original source generation regression failed before this correction.
2. Two maps with different keys can share a lookup result type. Re-emitting
   its storage declarations would reject a valid closure as duplicate globals.
   Storage definitions now share a per-generation cache keyed by exact type ID.
   The additional original source with two maps exercises this path.
3. Search was inside the output's selector group, making each observed bit
   recompute lower bound. The writer now receives shared search state; lookup
   uses the ordinary pointwise cube mux with shared arguments. There is no
   generated-operation shortcut in the evaluator or either checker.
4. The first shared writer had four value binders, which would exceed the
   256-binder ceiling for a C253 result. Position and length now use the existing
   C6 state. Each generated writer has three value binders, checked directly
   by the actual-source certificate test. The lookup mux helper is generated
   for its concrete depth when not already present, including nullable payloads.

The current nine-source generation/mutation subtest passed and pins 75 operations
across ten instances. The three supplemental source contexts passed 243 ordinary
observations across four instances, including NaN non-reflexivity, absent compare,
found(none), missing lookup and shared lookup definitions. These observations
use independent source/value reconstruction and the existing ordered-collection
model. Exact decimal-cohort key retention is tested separately because replacing
only a value must retain the stored key's bits; an equality-only oracle would
not detect changing its decimal scale/sign representation.

Entry source/certificate replay and same-byte dual checking passed for six
certificates with zero axioms and rejection of hash corruptions. The complete
entry semantic test subsequently passed all six contexts and 248 observations,
including decimal and non-reflexive NaN cases. No actionable entry-component
finding remains from this review. All nine current collection programs passed
exact pinned replay and mutations.
All nine collection certificates subsequently passed same-byte dual checking
with zero axioms and rejection of their hash corruptions (686.922 seconds).
The current maximum-update suite subsequently passed all three source contexts:
first/middle/last insertions into 4095-entry maps/sets, full-map replacement and
exact decimal 1.00 key retention (4,165.23 seconds). Integer map/set domain and
capacity verification subsequently passed both 4096-slot cases (4,341.47 seconds).
That run used the corrected thunk lifetime before iterative EnvRef destruction;
it does not verify the later observer version. Complete collection semantics
subsequently passed all 669 observations across nine sources/ten instances
(12,673.09 seconds), using the v4 writer before observer lifetime/destruction
corrections. The later 116-pair declaration-closure comparison preserves the
production definitions; the semantic run itself does not verify the new observer. The original capacity job used unchanged
domain/gate definitions with the previous writer. After the observer lifetime
correction passed scoped tests, that job was explicitly replaced by the full
same-scope suite using the corrected observer and current writer. The separate
maximum-update test supplies the completed current-writer boundary evidence. Superseded runs and
their concrete replacement reasons are retained in the progress record.

Code inspection checked complete expanded-operation coverage, exact source/type
links, dependency closure, operand/result depths, full u32 bounds, lower-bound
sentinel conversion, first/middle/last shift formulas, padding/tail zeros, failure
order, conditional comparison and non-null/nullable lookup separation. The
remaining recursive-domain commands must finish and any findings must be
resolved before a final unit review can be claimed. Public/source obligations and application
VC proofs remain separate; no checker rule, axiom or public route is added.

Internal unit 3 and original units 4-8 remain open. No new commit or W09 completion
is claimed. The full T06 gate remains deferred to W12.
