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

## Halting returns and actual-success arithmetic

The [ordinary halting Quote model and 108-case evidence](halting-returns/README.md)
extend the previous continuing-prefix ingredient with actual return status,
distinct closed error tags and whole-frame stop absorption. Actual root
success derives its entry/item checks, nonnegative bounded result and exact
natural sum from literal-zero entry; actual failure yields the model's zero
decision. A failed low prefix survives the later prefix unchanged. Arbitrary
input frames retain all 128 payload/metadata bits, and backing/padding cells
remain quantified at the eight workload counts.

Both checkers accept 54 zero-axiom claims with matching hashes and reject 54
altered claims at core type checking. Each group has two identical fresh
generations and no fuel failure. The same original-Go observer remains
empirical, separate from these ordinary model certificates. Complete source,
string/slice/operator/loop/return/error/descriptor bindings, remaining byte/
collection/control laws and aggregate resource closure remain open. W03 is
In progress; W04 has not started and the full T01 gate remains due at W05.

## Canonical index values and unobservable finite string tails

The [finite-byte model and 46-case evidence](byte-tails/README.md) prove
value-based 64-bit index coherence, full finite backing reads, tail-mask
idempotence and preservation of defined full-byte reads/equality/bytewise
order. The finite backing retains every byte and padding bit at capacities
0/1/32/256; comparisons quantify both backings. No function-extensionality
axiom, record eta, operand-value enumeration or smaller source word is used.
The source-defined-index guard remains separate from total model projections.
There is no empty defined-index positive.

Both checkers accept 25 zero-axiom claims with matching hashes and reject 21
altered claims at core type checking. Two fresh generations are identical,
and a separate replay passes all 46 candidates and exact Go observation
bytes. All 2,791 original-Go byte observations match the independent bit-loop
model, including invalid UTF-8/NUL, equal prefixes, logical versus padded
comparison and negative/extreme/boundary indexing. They remain empirical,
distinct from captured-source application proofs. Complete Go source/carrier/
operator binding, substring/concatenation/prefix/suffix and remaining length/
content laws, slice/descriptor/control composition and aggregate resource
closure stay open. W03 remains In progress; the whole T01 gate stays due in W05.

## Full-width bounded-index coverage and signed guards

The [index-bound model and 27-case evidence](index-bounds/README.md) derive
length nonnegativity and agreement of the actual source-shaped signed index
condition with the byte model guard from the full-width length bound. A
valid index then equals its finite embedding as a complete canonical 64-bit
value. This closes an ordinary prerequisite for the earlier finite byte
lift. There is no caller fit oracle or modulo-only equality. Capacity0
proves absence of valid indexes; capacity1 proves that a valid index is zero.

The four capacities 0/1/32/256 remain private workloads. The matrix contains
12 zero-axiom claims and 15 altered-result/erased-bound/erased-guard
mutations. Actual-Go/bit-loop counterexamples identify false claims
separately from core type rejection. Production Go source/operator/carrier
bindings, remaining string operations, slice/descriptor/control composition
and aggregate resource closure remain due. W03 remains In progress; W04
has not started and the full T01 gate remains deferred to W05.

## Descriptor/store sharing, array snapshots and observable slice tails

The [slice-store model and 90-case evidence](slice-stores/README.md) now
preserve explicit nil/region/offset/length/capacity observations, whole
128-bit cells, view assignment sharing, distinct-region frames and array
value-copy snapshots. Full re-slicing retains the descriptor's capacity and
exposes existing tail data. It does not apply string tail normalization to
slice storage. All source numeric fields remain 64-bit words; region names
are two symbolic identities rather than source pointer bits.

The private workloads are 0/1/8/32/127/128/129 cells. Both checkers accept
45 zero-axiom candidates and reject 45 altered-result/region/tail candidates
at core type checking. Actual-Go/independent-bit-loop observations match
all 1,596 cases, and every false mutation has an empirical witness distinct
from type failure. Original positive type/fuel failures remain preserved in
[the diagnostic archive](slice-initial/README.md).

These are ordinary store/descriptor ingredients. The following checkpoint
discharges ordinary absolute-address coverage and computed validity/access
composition. Source/ownership bindings, general slicing, overlapping `copy`,
bounded `append`, remaining byte/control laws and aggregate resources remain
due. W03 stays In progress; W04 has not started and the full T01 gate remains
deferred to W05.

## Computed validity, actual guards and absolute address coverage

The [slice-access model and 104-case matrix](slice-access/README.md) connect
the earlier descriptor/store algebra to computed validity. Non-nil validity
computes live-region, nonnegative/order and actual offset-plus-capacity
extent conditions. Its 65-bit extent is the actual carry plus 64-bit sum,
not a free safety flag or a new Go source integer type. Nil validity keeps
zero length/capacity. Descriptor copying and full re-slicing preserve it.

Computed validity plus the actual local signed index guard derives a live
region and a nonnegative, nonwrapping absolute address below storage length.
The separate semantic storage bound then yields complete canonical 64-bit
address equality with its zero-extended low-bit reconstruction, followed by
whole 128-bit finite reads. Both backings/padding remain quantified. The
semantic 127 and 129 bounds stay distinct from padding128 and padding256.
Zero storage excludes access, avoiding a vacuous indexed-read positive.

Original-Go construction and index outcomes are recorded independently of
raw-model validity. Invalid length/capacity metadata, a forged dead binding,
and legal Go reads outside the private bound have distinct false-claim
witness domains. A total modulo-heap self-read alone cannot establish full
numeric address coverage. Empirical observer results remain separate from
ordinary certificates and captured-source application proofs.

This closes ordinary validity/access ingredients, leaving production Go
capture/model/ownership/operation bindings, general slicing, overlapping
`copy`, bounded construction/`append`, remaining byte/control laws and
aggregate resource closure pending. W03 remains In progress; W04 has not
started and the complete T01 gate stays deferred to W05.

## General signed two-/three-index slicing

The [general reslice model and 100-case matrix](reslicing/README.md) now
derive descriptor validity and exact capacity from the actual signed
low/high/maximum guards. Every numeric input remains 64 bits. Checked
inverse/associative carry composition and non-strict bounds preserve nil,
non-nil empty and equal-endpoint cases. Two-index high bounds use capacity,
including values above the old logical length; three-index capacity uses
the explicit maximum.

Defined new indexes map to defined indexes in the old view expanded to its
existing capacity. Full-word address/key equality then preserves complete
128-bit reads in the original backing, with both stores/padding quantified.
The semantic bounds remain 0/1/8/32/127/128/129, separately from their
power-of-two padding. Zero emits no indexed-read positive.

All 4,261 actual-Go/bit-loop observations match, including capacity changes,
nil/empty results, tail exposure, alias-visible updates, source range
panics, index panics and out-of-profile reads. The
[original failed-positive records](reslice-initial/README.md) remain
separate from false-claim witnesses. Actual source guards cannot be
replaced by the computed result's validity alone. Observer-only writes and
recovery remain outside the proof closure.

This closes ordinary general-reslice ingredients. Production Go capture/
model/ownership/operation bindings, overlapping `copy`, bounded construction/
`append`, remaining byte/control laws and aggregate resources stay pending.
W03 remains In progress; W04 has not started and the full T01 gate remains
deferred to W05.

## Overlapping min-length copy

The [bulk-copy checkpoint](bulk-copy/README.md) adds an explicit immutable
pre-state heap relation, full64 relative-index cancellation, computed
min-length bounds, both actual source/destination guards, whole-cell copied
reads and destination/other-region frames. Its 84 candidates retain private
semantic storage bounds independently of padding. Nil/empty count claims
remain nonvacuous, and the source-after-copy mutation is emitted only where
an overlapping bounded-source counterexample exists.

Actual Go `copy` agrees with 341 independent bit-loop/snapshot observations.
Right and left overlaps, unequal lengths, nil/empty operations, full frames,
construction/index failures and out-of-profile reads remain distinct. Twelve
observations refute a naive forward-loop lowering and 17 refute substituting
the post-state source for its pre-state values. Observer-only allocation,
mutation and recovery prove no production capture or borrowed-write ownership.

Both checkers accept 42 zero-axiom claims and reject 42 false mutations at
core type checking. A separate replay reproduces all 84 canonical candidates
and exact native Go bytes. The 753 predecessor certificates, two standalone
indexes and 13 complete case indexes/outcomes remain unchanged. The selected
403 tests have passing applicable results after one negative test fixture is
corrected and its single-test rerun passes. W03 stays In progress. Production Go capture/model/ownership/
operation bindings, bounded construction/capacity-preserving `append`,
remaining byte/control laws and aggregate resource closure stay due. W04 has
not started; the complete T01 gate stays deferred to W05.

## Fresh construction, append and preallocation composition

The [construction checkpoint](construction/README.md) initializes complete
capacity, retains non-nil empty construction and derives separation from the
computed old live mask. Non-reallocating append computes full64 extent/carry
and capacity conditions, preserves metadata, proves actual result/saved-full
index guards and reads immutable pre-append source values. Old prefixes,
capacity tails, other regions and queries outside the absolute write interval
are framed. Composed allocation/append retains borrowed input values from
before both operations and preserves all formerly live regions.

The matrix has 276 candidates and explicit typed witnesses for its 138 false
mutations. Actual Go agrees with 2,806 independent bit-loop/snapshot cases.
Successful source reallocation stays outside the proved no-reallocation
condition and is never relabeled as a panic. Its exact growth policy remains
unmodeled. Invalid live/allocation bindings and failed-parent construction
observations stay distinct from actual source index attempts. Observer-only
allocation, seeding, I/O, recovery and value-based alias probes do not grant
production source or borrowed-write ownership admission.

Both checkers accept 138 zero-axiom candidates and reject 138 false mutations
at core type checking. A separate replay reproduces all 276 canonical cases
and exact native bytes. All 473 selected tests pass. The 837 predecessor
certificates, two standalone indexes and 14 complete case indexes/outcomes
remain unchanged. Three capacity groups are measured concurrently;
resources remain per-child observations without an isolated-machine or
profile-budget claim. W03 stays In progress, W04 has not started and the
complete T01 gate remains deferred to W05. Production bindings, remaining
byte/control and application proofs, and aggregate resource closure stay due.

## Guarded substring and bounded concatenation

The [string-operation checkpoint](stringop/README.md) derives substring length
bounds and actual old-source indexes from the signed source checks. Concat
computes full64 length addition, derives carry absence from the input lengths,
requires an explicit result bound and preserves both source byte ranges.
Defined-result reads retain all eight byte bits and preserve canonicalization
of input tails. Semantic bounds 0/1/31/32/33/255/256/257 derive full index
reconstruction independently of finite padding 1/1/32/32/64/256/256/512.

Actual Go and the independent bit-loop reference retain arbitrary bytes,
empty/NUL/invalid-UTF8/multibyte boundaries, extreme endpoints, range/index
panics and successful concatenation above the private bound. False witnesses
distinguish source panics from profile-domain failure. Native recovery and I/O
remain observer-only. These ingredients do not complete production source/
effect/operation bindings, checked prefix/suffix or full applications. W03
remains In progress and the complete T01 gate remains assigned to W05.

The initial 248-candidate matrix and independent final replay both pass: 124
zero-axiom acceptances and 124 core type mismatches, with exact canonical
certificate and native-byte reproduction. All 537 selected tests pass.
The 1,113 predecessor certificates, two standalone indexes and 15 complete
case indexes/outcomes remain unchanged; source pins and adjacent hardware
identity are refreshed. No current incomplete attempt remains, and the full
T01 gate is still deferred to W05.

## Byte prefix/suffix and selected source-body correspondence

The [prefix/suffix checkpoint](affix/README.md) retains full64 lengths/indexes
and all eight byte bits. The actual length comparison derives substring
safety and exact pattern length. Closed length/slice/equality formulas
correspond to both internal relations; true results derive actual source
indexes and full-byte equality. Empty/longer patterns, input-tail masking and
complete finite-index coverage remain distinct from semantic padding.

Original Go 1.25.0 library calls and literal bodies match 5,395 independent
observations. An AST receipt identifies exactly the two `strings` wrappers
and their two `internal/stringslite` callees. This does not install an adapter
or approve the whole package. Archived type/fuel failures stay separate from
semantic-negative controls. A deterministic standalone Rust extension checks
46 generated input identities and preserves all predecessor producer pins,
certificates and observations. Unchanged kernel measurements are not repeated.
Production capture/model/effect/operation binding, registered adapters,
remaining control/aggregate and full applications remain due. W03 stays
In progress and the full T01 gate remains deferred to W05.


## Coupled struct, descriptor-address and byte bounds

The [aggregate checkpoint](aggregate/README.md) combines complete value cells,
64-bit physical descriptor addresses and prefix/suffix byte observations in
one theorem. Separate semantic byte/storage bounds remain distinct from
shared padding. Nine B/S pairs, through 256/128 and boundary 257/129, retain
all cell and heap bits. Both checkers accept 49 zero-axiom claims and reject
49 deliberate false mutations at core typing. The independent final replay,
732 original-Go observations and all 37 new targeted tests pass. Existing
1,665 candidates remain unchanged; no production source or registry changes.

The [W03 exit audit](feasibility-exit-audit.md) separates feasibility from
T02–T06's production integration. General loop induction requires an additive
dependent interface because legacy Std.Nat.rec is Nat-to-Nat only. That
private prototype and the isolated resource-envelope review remain due before
W04. The full T01 gate remains deferred to W05.


## Private dependent induction and abrupt edges

The [proposed Nat.induct checkpoint](control-induction/README.md) adds a
canonical monomorphic Sort0 dependent interface only in mirrored Rust/Go
checkers. It preserves production files and the old Nat.rec/Boolean rules.
Six arbitrary-Nat invariant/stop and single-edge claims pass with zero axioms;
six false variants reject at core typing. Fifteen separate interface cases
admit two controls and reject thirteen forgeries. A separate replay and all
576 original-Go observations pass. Targeted checks include twelve new
interface tests, eleven Boolean regressions and 37 evidence/receipt tests.

The [two original-checker diagnostics](control-initial/README.md) remain
explicit. The new rule requires W04 freeze and T02 integration. Sequential
resource measurements and the cumulative [exit audit](feasibility-exit-audit.md)
remain before W03 completion; the full T01 gate stays deferred to W05.


## Sequential upper/boundary resource review

The [sequential measurement](sequential-resources/README.md) freshly generates
and checks fourteen representative upper/boundary recipes, one child at a
time, preserving exact canonical and accepted hashes. Construction, each
checker and complete certificate size/counts are recorded separately. All
fourteen checks and eighteen new evidence tests pass. Other host processes
remain uncontrolled. Exact limits/budgets remain W04 freeze work; the full
T01 gate remains W05 work. The final cumulative W03 exit review follows.
