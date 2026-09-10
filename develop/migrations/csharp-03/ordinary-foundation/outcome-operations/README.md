# Ordinary outcome operations (W09 unit 3)

The generator reconstructs every expanded option, lookup, result, validation and
boundary-field instance from the validated original VIR. It checks exact arm
names/tags, payload types, validation's role bound, dependencies and the complete
frozen operation signature/equation/error list. Uninvoked operations are included.
Imports regenerate exact metadata and canonical certificate bytes, with the same
16 MiB limits as the other ordinary operation programs.

Constructors use the existing ordinary sum storage definitions, zero new padding
and preserve all active payload bits. Predicates compare the entire u32 tag.
Payload getters have an explicit invalid-operation gate and return storage zero
on another tag; that storage value is not an accepted normal result. Option
value-or chooses the active payload or the supplied same-type fallback at Bool
leaves. Equality and eligible comparison use semantic child relations, including
non-reflexive IEEE NaN equality. An IEEE-containing instance has no comparison
operation regardless of its particular tag or value.

Validation's invalid constructor requires a nonempty error sequence of at most
256 elements. Its ordered failures are empty-errors and validation-bound.
Append takes two bounded sequences, preserves every left element followed by
every right element, and fails if their combined length exceeds 256. Empty
append inputs are permitted by this frozen operation; the separate invalid
constructor still rejects an empty resulting sequence. The ordinary count adder
is exact over valid input lengths (each at most 4096) and saturates malformed
large lengths instead of wrapping into acceptance. A concrete unsigned
subtraction circuit calculates right-hand offsets. Inactive output slots and
new count padding are zero.

These are operation definitions, not application proofs. Input representation
and public domains, all gates, source binding/projection correspondence and
application VC obligations remain mandatory. In particular, lookup of a nullable
value retains the distinction between missing-key and found(none); an inactive
getter's zero result cannot erase its failure gate.

The corpus contains 16 actual-source contexts, 17 concrete instances and 115
operations. It includes existing binding/nullable/string/domain sources and three
new captures in `../outcome-sources/`: Bool/product error sequences and nullable
lookup. Maximum certificate sizes are 7483 terms, 168 declarations and 8258
counted static transformers. All original 13 programs and metadata are byte
unchanged after adding the three extra sources.

Source semantics passed 907 ordinary-term observations across all five template
families, including constructors, active/inactive payloads, value-or, tags,
equality/order and nullable lookup. Every high tag bit 2..31 is also tested
against payload gates. Validation append tests cover empty sides, left/right
order, sums 256/257, malformed full-word lengths and Bool/product element shapes.
For nonempty valid inputs the existing outcome model is an independent oracle;
empty sequence append is checked against the frozen sequence operation contract.
Large output padding is sampled while checking every expected nonzero leaf and
neighboring addresses. These finite observations are not universal padding or
application proofs.

Targeted commands:

```sh
cargo test -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_outcomes_original_source
cargo test -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_validation_error_append_and_bounds
cargo test -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_validation_append_bool_and_product
cargo test -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_outcome_source_requests
```

`MPK_W09_OUTCOMES_OUT` selects a separate candidate regeneration directory;
normal tests require exact checked-in bytes. Request generation likewise uses
`MPK_W09_OUTCOME_REQUESTS_OUT` only for an explicit output file. The Go test
`TestCheckerAgreementWithRustCLIOutcomeOperations` sends the same decoded bytes
to both unchanged checkers, requires zero axioms and rejects hash corruptions.
Current evidence is recorded in `../unit-3-outcome-verification.json` and
`../unit-3-outcome-review.md`. Unit 3/W09 remain incomplete; maps/sets, source
state integration and original units 4-8 are still required. The full T06 gate
remains deferred to W12.
