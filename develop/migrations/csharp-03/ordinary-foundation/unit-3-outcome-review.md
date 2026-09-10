# W09 unit 3 outcome operation review

Scope: the new outcome generator, its sequence-offset circuit, exports,
actual-source captures, scoped tests and pinned certificates. This is not a
completion receipt for all pending unit 3 changes or W09. Evidence and exact
artifact/log hashes are recorded in `unit-3-outcome-verification.json`.

Direct review checked complete expanded-instance coverage, exact frozen
signatures/equations/error order, payload argument/result types, all tag bits,
source/context reconstruction, optional comparison, dependency closure,
validation nonempty/upper-bound conditions, unsigned offset selection,
left-then-right order, zero tail/count padding and exact importer regeneration.
Only ordinary definitions are emitted; no checker rule or application authority
is introduced. Normal-result bodies cannot substitute for input/public domains,
failure gates, source binding correctness or application proofs.

The first 13 original-source contexts passed 675 observations and both checkers.
Review expanded the corpus to Bool and source-product error sequences and
lookup of an optional value. The resulting 16-source corpus passes 907 ordinary
observations. The new lookup fixture exercises both missing-key and found(none)
without collapsing them. Bool/product append cases make the difference between
left/right order observable against the reference outcome model. The original
13 certificate bytes and their metadata are unchanged.

The i32 validation boundary test passed in 176.20 seconds, including empty
sides, exact 256, rejecting 257/4096, and arbitrary large lengths that would
wrap under unguarded u32 addition. Its normal-body read uses the right offset
only when the index is at least the left length. Saturated count addition is
exact for every admitted pair of input lengths. Empty append inputs remain
valid sequences, while an empty invalid outcome is rejected by its own gate.

The new source-request helper initially used a nonexistent arm-mapping type
and private type-ID helpers. It now uses the public semantic-arm mapping and
closed-instance identity API. An ambiguous integer type in the supplemental
test closure was explicitly made usize. These were test compilation failures;
the production operation terms were unchanged. All three new frontend captures
were accepted by the existing deterministic two-run control-emission harness.

No additional actionable implementation finding remains in this component's
direct review. See the verification record for completed versus pending checker
and replay commands. Large padding observation is explicitly sampled and does
not discharge universal input/source obligations. The ongoing recursive-domain
suite, remaining map/set operations, construction source-state integration and
the original units 4-8 remain required. No unit 3/W09 completion or commit is
claimed by this component review; the full T06 gate remains at W12.
