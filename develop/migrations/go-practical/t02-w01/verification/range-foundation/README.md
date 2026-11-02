# Actual ordinary half-open logical folds

This W01 component adds eight ordinary definitions and31 checked laws to the
fresh Bool/Nat/Int foundation. Its zero-axiom certificate has748 declarations,
12,738 terms and169,034 bytes. The complete certificate hash is
`ddb538b929f4ef0e04d6ac74f119d561e2df132cdba409aa61b0259b99179a70`.
The complete preceding709-declaration integer component retains its exact bytes
and hash. No prototype certificate/graph/context or evidence receipt is promoted.

The [production recipes](../../../../../../crates/mpk-vc/src/go_practical_ordinary/range.rs)
define FoldForall, FoldExists, FoldNatSum and FoldIntSum by genuine unbounded
Nat induction. Each receives lower, count and a total mathematical body; its
successor step visits `lower + index`. The Forall/Exists/NatSum/IntSum interval
wrappers use `count = truncated Sub upper lower`, giving exactly `[lower,upper)`.
The result is true, false, Nat zero or Int zero for equal/reversed bounds.
Neither a machine-integer bound nor finite loop unrolling proves these laws.

Three Boolean projection/reflection laws are ordinary case proofs. Each fold
has seven ordinary laws: zero, successor step, same bounds, conditional empty,
pointwise preservation, bounded preservation and interval preservation. The
bounded theorem's premise is the actual FoldForall of element equality over
the visited prefix. The interval theorem uses that actual half-open Forall.
No equality outside the visited interval is assumed. Integer sums use semantic
cross-sum equivalence; function equality of their balanced representatives is
not substituted for it. Total mathematical bodies carry no source-evaluation,
short-circuit definedness, side-effect, permission or application-proof claim.

The exact importer reconstructs all complete ordinary bytes. The tests reject
honestly different valid components as well as all altered/rehashed candidates.
No kernel/checker code, reduction, fuel, opcode, axiom category or function
extensionality rule changes. Standard inputs and every255 frozen W04 design
member remain unchanged in the [preservation observation](frozen-design-preservation.json).

All five corrected [integration roots](reviewed/rust-tests/result.json) pass.
Twenty-five closed interval observations cover true/false Forall/Exists and a
unit Nat sum over zero, nonempty, equal and reversed bounds; one more checks
semantic zero for a nonempty integer sum. The bounded-only case uses constant
true and IsZero bodies, which agree at the visited index0 and provably differ
at unvisited index1. Actual bounded Fold/interval premises and conclusions are
proved, with no globally pointwise assumption.

The [focused additional root](final/rust-control/result.json) strengthens the
integer-fold control after review. A three-argument constant-zero fold is
independently accepted as a well-typed value; replacing the actual FoldIntSum
with it fails the required step law. Its acceptance as a value proves no fold
correctness and exact component import still rejects it. Only this affected
root reruns; unchanged positive/other-negative proof bytes are retained.

The [paired current-checker observations](paired-reviewed/result.json) agree on
all17 original byte streams: six zero-axiom acceptances with identical complete
certificate/export/axiom hashes and eleven core type mismatches. Controls alter
fold identities/body/offset/bounds, replace actual premises with unrelated or
forged proofs, or demand the stronger integer function equality. They rebuild
all ordinary metadata and are not counted as encoding/hash/resource failures.
The original Go binary and every checker input are rehashed against its prior
build. Strict [Clippy](final/clippy/result.json) and [format](final/format/result.json)
pass; unchanged core/Go/legacy unit matrices are not repeated. No C# tests run.

The [initial execution](initial/rust-tests/result.json) accepts all general
foundation laws but rejects two concrete observation recipes. Reflexivity/beta
conversion under computed recursor majors does not provide the missing
congruence transports. The corrected proofs explicitly compose checked fold
steps, body equalities, congruence, and core-equality-to-integer-equivalence
transport. Both exact initial candidates, source inputs and raw diagnostics
remain. Removing one unused test field changes no proof bytes, as shown by the
[all16-candidate byte comparison](review/formatter-cleanup-byte-equality.json).

The observer initially allows only app/app, then app/pi term-kind diagnostics.
Both checkers already reject the same candidates with matching actual kinds;
the observer is too restrictive. Both original observers, partial outputs and
raw failure reproductions remain in the
[first](initial/paired-observer-failure/result.json) and
[second](initial/paired-second-observer-failure/result.json) records. The final
observer matches the actual detailed Rust kinds against the exact Go message
for all seven existing term arms, while still requiring a core type mismatch.
The second review also finds that the original integer-fold mutation traverses
the exposed result-carrier Pi as an extra argument. That weaker typed control
remains preserved and is replaced in final evidence by the separate well-typed
value plus genuine step-law rejection described above.

The [verification record](verification.json) retains exact sources, commands,
times, selection reasons and scope. The changed-input review retains all raw
diagnostics and four exact formatter EOF observations, with no source/JSON/
directory or blanket whitespace exclusion. The cumulative T02 gate remains
deferred to W04: `check-fast.sh --skip-csharp`.

Bounded source/body definedness, full word/carrier/source/operation/storage/
lifetime/program laws and the complete56-family foundation/descriptor/private
registry/context still remain due. W01 stays In progress; all169 application
theorem demands remain unelaborated. No public profile, source-program proof or
native Linux acceptance is claimed.
