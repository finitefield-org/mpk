# GO-PRAC-T01-W03 proof feasibility checkpoint

Status: In progress. W04 has not started and T01 is not complete. This
checkpoint retains actual ordinary symbolic lemma certificates and local
resource observations. It establishes no public Go profile or Go function
theorem.

The [private producer](../../../probes/go-practical/certificates/src/main.rs)
uses the zero-axiom `fixtures/core-bool-cases/right-identity.hex` foundation
and a reused untrusted certificate-term encoder. The producer is not a
checker. Every candidate is passed as identical canonical bytes to
`mpk_kernel::verify_certificate_bytes_json_output` and the separately built
Go `mpk-checker-ref verify` command. No production checker source is changed.

## Observed scope

[Corrected evidence](pilot-corrected/observations.json) pins the exact source
inputs, both checker binaries, Rust/Cargo versions, Go 1.25.0 host compiler
inventory and host/architecture. Its [index](pilot-corrected/index.json)
records each theorem, quantifier count, term count, bytes and intended result.
The report contains actual construction/checker wall time, CPU time, peak
resident bytes and raw checker JSON for each case. The local host is
Darwin 27.0.0 arm64 with 10 reported CPUs. Model and physical RAM are unknown
because their sysctl access is unavailable in the workspace sandbox.

Seventeen claims are accepted by both checkers with zero axioms. Ten
deliberately false projections or writes reject at core type checking in both
checkers. A malformed transport, timeout or source rejection does not count
as a negative proof result. Accepted export/certificate/axiom-report hashes
agree across checkers. Two fresh generations are byte-identical, and isolated
per-case construction matches the same canonical output.

The pilot quantifies complete 64-bit payloads, complete stores, arbitrary
state transformers and every observed payload-bit address. It proves a
nominal tag distinction, pair projection, closed success/failure tag behavior,
return frame, continue/post behavior, a one-bit full-adder carry relation,
read-after-write and whole-cell updates. String capacities are 0/1/32/256
eight-bit cells; slice capacities are 0/1/8/32/128 64-bit cells. Empty-length
and nil-tag properties are separate observations. This does not yet establish
every descriptor validity or Go source binding rule.

The largest positive string candidate contains 13,392 terms and 113,603
bytes; the 128-cell slice candidate contains 7,377 terms and 61,442 bytes.
These are feasible **cell-update lemmas**, not measured complete string,
slice, `Quote`, or filter proofs. Volatile time/RSS values remain in the
machine-readable report; no frozen profile budget is inferred from this
partial workload.

## Exact local natural-number conservation

The subsequent [four-candidate arithmetic measurement](natural-conservation/observations.json)
and its [index](natural-conservation/index.json) use the
[natural-number producer](../../../probes/go-practical/certificates/src/natural.rs).
It copies the existing zero-axiom `Std.Nat` constructor/recursor interfaces
into the self-contained certificate, with remapped indices. Both checkers
validate all 23 declarations, including the new theorem. No imported premise
or completion claim for another language is involved.

For Boolean inputs `x`, `y` and incoming carry `c`, define
`s = x xor y xor c` and
`k = (x and y) or (c and (x xor y))`. If `inc_b(n)` means `succ(n)` when
`b` is true and `n` otherwise, the first accepted theorem proves, for every
natural `n` and all three flags:

```text
inc_x(inc_y(inc_c(n))) = inc_s(if k then succ(succ(n)) else n)
```

The second accepted theorem generalizes `succ` to every function
`step : Std.Nat -> Std.Nat`, applying that same step twice for the carried
place. This permits later composition at a weighted place without treating
the local carry flag itself as the mathematical-value theorem. The two
mutations discard the carried place and reject at core checking in both
checkers. Positive certificates contain 753/754 terms and 8,024/7,972 bytes.
They require no natural-number recursor application or new checker rule.

Each claim universally quantifies its initial natural and input flags. The
proof splits only the three flags, then supplies explicit equality steps.
This is exact local mathematical-value conservation, not the complete
signed 64-bit, overflow, multiword or processed-prefix sum bridge. Those
requirements remain open below. The runner measured construction and both
checkers for all four cases, produced identical bytes twice, and checked
isolated generation against the complete generation.

## Retained initial finding and correction

[The initial ten-case run](pilot-initial/observations.json) includes its
[producer snapshot](pilot-initial/initial-main.rs), original certificate
bytes and raw rejection observations. Four intended positives failed:
nominal tag, closed error, carry and backing read-after-write. The producer's
normalizer had assumed that conversion would reduce a nested recursor major.
Both current checkers fire iota only on a constructor-headed major.

The corrected producer retains the original theorem goals and constructs
`Std.Eq.congr`, `Std.Eq.trans` and `Std.Eq.symm` witnesses for the intermediate
steps. It does not replace the theorem by a normal form, add an assumption,
or modify conversion. It also handles the reference checker's omitted
zero `axiom_count` field and the Rust checker's `error_code` rejection field.
The report preserves each checker's original JSON representation.

BSD `time -l` could not measure resources because its `kern.clockrate`
sysctl is forbidden in the sandbox. The runner instead uses `os.wait4` for
its own child processes. This changes only the measurement implementation.

## Requirements still open

- Complete the source/model bindings and validity invariants for the proposed
  nominal, result/error, byte and slice representations.
- Establish the index/length/capacity, remaining operation, alias and abrupt
  control laws needed by the actual application proof shapes.
- Complete the signed bitvector to exact mathematical-integer/sum bridge.
  The one-bit carry lemma is a building block and discharges none of that
  full requirement by itself.
- Measure the full aggregate interactions and limit-minus-one/limit/limit-
  plus-one cases before W04 freezes any numeric/resource bounds.

The next work must retain these requirements and the design's actual proof
scope. A feasibility failure requires a recorded correction; it cannot be
hidden by a ground-only or definition-only substitute.

## Targeted validation

Replay and report-integrity commands are in the
[probe instructions](../../../probes/go-practical/README.md). Checks target
the changed private producer, resource runner and retained evidence. Their
adversarial tests reject lost coverage, changed certificates, missing checker
results, axioms, checker hash disagreement, stale source identities,
non-core negative failures and invented completion. Offline Clippy checks
the private Rust crate. Unchanged production source/old Go fixture identity
is checked with the W01 audit rather than rerunning unrelated suites.

The same runner remeasured the 27 representation candidates after adding the
natural-number producer so both reports pin the current private source. The
previous representation certificate bytes are unchanged; source/checker
binary identities and resource observations are recorded for the new build.

`scripts/check-fast.sh` remains deferred to the last W of T01, W05. No
native Linux release or public-profile acceptance check is claimed here.

## Compositional unsigned-word diagnostic

The [word model](word-conservation-model.md) and
[retained diagnostic](word-initial/README.md) extend the one-bit ingredient
to all 128 operand bits of an unsigned 64-bit addition. Both checkers accept
its exact natural-value theorem, including final carry, with zero axioms and
identical logical hashes. The certificate has 81,514 terms and 1,043,626 bytes.
This remains a partial arithmetic bridge, not signed interpretation, an exact
processed-prefix sum, or a source-bound Go theorem.

The initial matrix stops at the 32-bit dropped-carry candidate because both
checkers exhaust conversion fuel. Its raw result is retained as a resource
failure and is not added to the semantic-negative count. The separate 64-bit
positive diagnostic passes at about 198 s in Rust and 476 s in Go; it is not
a complete replay/matrix or a frozen application resource budget. The next
candidate will use the existing Bool-addressed cube and balanced half-word
composition to reduce the work without reducing the input domain. W03 and
its existing remaining requirements stay open.

## Balanced word and signed bridge measurements

The [balanced model and actual 48-case evidence](balanced-words/README.md)
now replace the long operand telescope with full Bool-addressed operands.
Both checkers accept all 16 unsigned/conditional-signed positive claims with
zero axioms and identical logical hashes. Twenty-five incorrect candidates
fail at core type checking; seven offset-removal candidates reach conversion
fuel and are explicitly not counted as semantic negatives.

The unsigned64 theorem checks in about 0.126/0.266 s and the signed64 theorem
under its actual no-overflow guard in 0.186/0.353 s. Both generations are
identical for every width and family. The signed proof ties its guard to the
actual incoming/outgoing sign-stage carries and proves equality of natural
cross sums for two's-complement values. This is a conditional addition bridge,
not all integer operations, a completed prefix-sum bridge or a Go application
proof. Internal width128 probes do not admit Go 128-bit source integers.

The first balanced positive failures at 16 and 32 bits are retained in
`tree-initial/`. Explicit checked projection and carry equalities resolve
those positive-proof fuel failures without changing either checker. Complete
prefix-sum composition, Go source/operation binding and guard discharge,
collection/control laws and aggregate boundary/resource freeze remain open.
W03 stays In progress and the full T01 gate stays deferred to W05.

## Exact unsigned prefix-sum ingredients

The [prefix model and actual 55-case evidence](prefix-sums/README.md) now
compose the full 64-bit adder through a real left-to-right accumulator.
Both checkers accept 18 zero-axiom claims with matching hashes: exact sums
under the actual step-carry premise, unconditional sums with actual carry
corrections, and valid empty-prefix controls. Twenty-three altered claims
fail at core type checking; fourteen offset/last-summand removals reach fuel
and are recorded separately as resource outcomes.

Counts `0/1/3/8/32/127/128/129` preserve all input/padding bits. Each group
has two identical fresh generations. Count129 probes a boundary and does
not change Go admission. The initial empty-prefix fuel failure and original
sources remain in `prefix-initial/`; explicit checked identity/composition
steps resolve that positive failure without changing a checker rule.

This is unsigned arithmetic evidence. Signed/nonnegative prefix interpretation,
the `Quote` guard/order/subtraction bridge, complete Go operation/source
binding, collection/control closure and aggregate/resource freeze stay open.
W03 remains In progress and W04 has not started. The full T01 gate remains
deferred to W05.

## Conditional signed prefix composition

The [signed prefix model and 32-case evidence](signed-prefix-sums/README.md)
compose the unsigned carry-correction law with a checked balance between
actual intermediate carries and actual operand/result signs. Both checkers
accept nine zero-axiom claims, including the valid empty guard control.
Fifteen altered claims fail at core type checking; eight dropped-result-sign
candidates reach fuel and remain separately classified resource findings.

All negative two's-complement patterns remain in the quantified domain.
The only arithmetic premise is each actual update's sign-stage carry
agreement, with checked projection to the same intermediate fold states.
Counts `0/1/3/8/32/127/128/129` have two identical fresh generations. The
128-element positive has 17,736 terms and 214,747 bytes. This closes a
conditional signed prefix-composition ingredient; the actual Go source
guard/order/subtraction discharge and nonnegative `Quote` invariant remain
open. Collection/control/operation bindings and aggregate resource closure
stay due. W03 remains In progress and the full T01 gate stays deferred to W05.

## Source-shaped comparison/subtraction guard discharge

The [order/guard model and 40-case evidence](order-guards/README.md) now
derive the actual addition guard from full-word signed comparison,
nonnegative loop-state conditions and the wrapping remaining-amount
subtraction. A six-state relation between four carry chains composes local
bit facts through complete operands. The resulting step has no unsigned
wrap, a nonnegative bounded result, and exact `Std.Nat` addition. It does
not take the final addition guard as an unrelated premise.

Carry composition covers widths 1–128; source-shaped path and mathematical
claims cover actual signed Go widths 8/16/32/64. Both checkers accept 16
zero-axiom claims and reject 24 substitutions at core type checking. Two
fresh generations are identical for each group. A pinned Go observer also
matches the independent bit-loop model on 3,428 boundary/sign/full-range
cases, including noncontinuing and wrapping inputs. Those empirical
observations remain distinct from proof evidence.

Production Go branch/operator binding, whole-loop/prefix/return composition,
collection/control/byte law closure and aggregate resource freeze remain
open. W03 stays In progress, W04 has not started, and the whole T01 gate
remains deferred to W05.

## Continuing-prefix composition and literal zero entry

The [continuing-prefix model and 72-case evidence](continuing-prefixes/README.md)
now derive every intermediate accumulator bound and carry from the actual
item-sign/remaining-amount checks and the initial invariant. No per-step
invariant or carry safety is supplied as a caller oracle. Checked zero-word
sign/comparison/value lemmas also discharge the source-shaped literal-zero
initializer, giving bounded zero-entry results and exact natural sums.

Counts 0/1/3/8/32/127/128/129 retain all input/padding values. The four
families have two identical fresh generations: 36 zero-axiom claims pass
both checkers and 36 substitutions reject at core type checking. The
empty-prefix check replacements remain accepted controls. The initial
zero-complement candidate failure is retained with its original source and
corrected by explicit checked congruence.

An ordinary-Go `Quote` fixture also matches an independent bit-loop model
on 769 cases, covering 169 successes, 456 bad-input and 144 limit-exceeded
results, input preservation and nil/non-nil empties. Those are empirical
observations, not a formal source/loop/error/frame proof. Complete source
binding, actual loop/return/error/descriptor composition, remaining
byte/collection/control laws and aggregate resources stay open. W03 remains
In progress; W04 has not started and the full T01 gate stays due in W05.
