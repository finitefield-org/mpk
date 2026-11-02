# Ordinary unsigned-byte string order

Status: Scoped checks passed; strict input review is recorded separately below. Entry is
`8814646e76c9dd75d2666e4610e8c091ea44fc8b`. Source language scope remains Go/Rust.
This checkpoint does not complete R01, R06, W01 or the practical Go profile.

The [producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/string/order.rs)
and [string bindings](../../../../../../crates/mpk-vc/src/go_practical_ordinary/string/order/string_values.rs)
produce 1,706 ordinary declarations / 52,523 terms / 679,227 bytes. They add
26 definitions and 190 laws, with no axioms, imports, function extensionality,
checker changes or increased limits. All 1,215 prior StringEquality identities
remain exact. The old full StringEquality and WordOrder producers regenerate
their original golden bytes. The closed private composer keeps the entire old
string component and the actual dependency closure of four compiled-in byte
comparison/numeric roots; callers cannot choose roots or supply source authority.

The [Go comparison specification](https://go.dev/ref/spec#Comparison_operators)
defines string order byte-wise. Rank0 represents end; an in-prefix byte has
rank `unsigned_value + 1`. Its guard uses the full unsigned64 logical size.
Thus an actual NUL byte differs from end, and length256 differs from length0.
At a shared present position, a checked rank comparison equals the original
Word8 unsigned comparison circuit. Actual left-end/right-present and the
opposite case are proved, as is equality when both positions are outside.
This reference does not update the frozen compiler or design inputs.

For fixed address depths0..8, ordinary Nat-vector Equal and Less have explicit
step laws. Equal reflects/completes actual pointwise Nat equalities. Less true
eliminates to a real address, equality at every smaller address and a strict
rank difference there; those actual witnesses conversely prove true Less.
Low/high index comparisons and crossing bounds supply the address argument.
Proper Bool case analysis and unbounded Nat induction supply the proof, without
an operand table. The elimination result remains Sort0. Small Bool algebra is
proved through explicit congruence and transitivity.

Four operations expose Less, Greater, LessEqual and GreaterEqual, with checked
reversal/complement, first-difference reflection/completion for strict order,
reflexivity and a total-order equality **of the rank vectors**. All four respect
both operands' actual full-size/pointwise-bit content relation and normalization.
No equality of whole functions is assumed.

The remaining universal bridge from bounded rank equality to the existing
full-size String.Equality is not claimed. Raw lengths beyond256 are not admitted
source strings. Source type, original evaluation and nominal/signed-index domain
binding, source validity, prefix/suffix/substring/concat preservation and program
proofs remain separate obligations. The private descriptor/registry/context,
complete ordinary foundation, native kernel, general namespace/selected closure,
all169 requirements and T02–T11 remain due.

The six [affected Rust roots](../../../../../../crates/mpk-vc/tests/go_practical_string_order.rs)
cover old identities/golden bytes, strict owning import, 190 altered conclusions,
122 replaced typed witness domains, 25 typed wrong definitions and actual finite
execution. Empty depth0 prefixes are vacuous; their mutation checks the exact
typed contract, not logical independence. Every changed definition has a fully
well-typed definition prefix and an actual semantic counterexample with canonical
padding. The complete altered certificate is rejected; partial acceptance is
never claimed. Controls cover constant results, missing reversal/complement,
missing end sentinel, ignored end, low8 length, high-first traversal, omitted
high halves and ignored earlier-equality guards at every depth1..8.

The unchanged definition interpreter observes all65,536 unsigned byte pairs in
113,660,694 AST steps and75 full string cases in147,304,477 steps. All four
operations run in both operand orders on raw and normalized carriers. Complete
arrays, full64 lengths, all byte bits and every available partial output remain.
Cases include18 boundary lengths, empty/full values, dirty tails, first/high-bit
and last/low-bit differences, equal prefixes, embedded/trailing NUL, invalid UTF8,
multibyte/distinct Unicode bytes and0 versus256 zero bytes. Only core constructors/
eliminators and complete native input data are primitive; no arithmetic/string
definition callback or theorem body supplies a result. Existing50M-step/2M-value/
2M-environment ceilings are unchanged. Finite observations supply neither the
universal proof nor native Go source/runtime evidence.

[Actual executions](epochs/) retain complete source epochs and original preimages,
prelaunch drivers, candidates saved before checking and available complete outputs.
The first borrow/import compile failures, two nested-Bool reflexivity failures,
and the strict Clippy argument-count failure remain. The [original diagnostic](epochs/original-boolean-diagnosis/diagnostic/)
executes the failed16 Bool endpoints; it does not execute theorem bodies or
reconstruct missing original outputs. Explicit congruence repairs proof conversion
without changing the core or fuel. The [original two failures](failed-paired/result.json)
are independently rejected by the unchanged Go core too.

[Same-byte core comparisons](paired/result.json), [verification](verification.json)
and [strict input review](input-review.json) record final outcomes. Strict current target Clippy and all six changed-file format
checks pass. The receiver-only Clippy repair regenerates the identical full
certificate, so the earlier six-root semantic/proof checks are retained against
those same bytes; observer and consumer source bytes remain exact. Both cores
agree on339 complete current inputs:two zero-axiom acceptances and337 genuine
type mismatches. The ten full source epochs/3,779 original preimages and14 actual
executions retain all five original failures separately from final passing checks.

All255 frozen inputs,186 Go frontend build inputs and127 matchers/four scopes/
39 seals remain exact, without a new source/JSON/directory/seal exception.
Unchanged Go/frontend/native capture/core/ref/protocol and unrelated suites are
retained without repeat. All source-consuming handles must terminate before
staging. The cumulative T02 gate remains at W04: `./scripts/check-fast.sh`.

The [first metadata review](input-review.initial.json) reported only the link to
its own not-yet-created output. Its exact original result and available tool
output remain; the review passes after that output exists. This metadata finding
is separate from the five retained source/test/lint failures above. Original
Cargo output EOF blank lines remain byte-exact; the [whitespace record](git-whitespace-review.json)
pins only those exact output files and checks all other staged files. It adds no
obsolete-IR scanner exception.
