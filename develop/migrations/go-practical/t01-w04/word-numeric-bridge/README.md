# Exact numeric bridge for the actual word addition definitions

Status: private W04 ordinary addition component. Entry:
`04fd6d9f267b705cdfc93d408b45a1ec5780a8bc`. The complete W04 freeze and all
T02–T11 work remain required. This component connects the actual preceding
ripple addition to the checked W03 Nat/signed library, including13 typed
Value definitions. It does not complete all word/Nat/Int operations or a Go
source/application proof.

The [catalogue](schema.json) specifies the exact evidence boundary and owners.
The [complete expansions](expansions.json) retain every ordinary declaration
for8/16/32/64-bit words. The
[untrusted producer](../../../../probes/go-practical/word_numeric_bridge.py)
starts with the unchanged [signed64 W03 certificate](../../t01-w03/balanced-words/signed-tree-64-nat-conservation.mpcert).
That seed has187 zero-axiom declarations and exact balanced Bool-cube value,
weight,addition and signed cross-sum theorems. Each resulting certificate
rechecks the complete seed as well as the new members in both independent
checkers. No old helper,proof,semantic identity,core reduction or budget changes.

For every aligned binary block,the bridge proves three facts about arbitrary
whole operands and the incoming Bool: the balanced block's outgoing carry
equals the actual ripple prefix carry; every result address equals the same
address in the actual AddCarry; and the balanced sign-stage incoming carry
equals the actual ripple carry at that position. Each parent composes checked
child theorems. Only the three local full-adder flags and address selectors
use Bool elimination. Complete operand bit patterns are never enumerated.

A separately checked value-congruence theorem lifts pointwise bit equality
through the balanced denotation by structural low/high composition. It does
not assume word function extensionality or eta. Combining this with the
retained addition theorem yields,for every initial Nat and incoming Bool:

```text
initial + unsigned(x) + unsigned(y) + incoming
  = initial + unsigned(actual AddCarry(x,y,incoming))
            + 2^width * actual final carry
```

The actual ordinary statement uses genuine Std.Nat and exact successor
denotations. Value recursively composes low-bit steps with high-bit weighted
steps; Weight recursively squares those steps. The initial natural is an
unbounded quantified Nat,not a machine word or JSON safe integer. No giant
unary literal,unchecked arithmetic adapter or opaque denotation axiom is used.

The signed bridge reads the highest bit of the same operands and result.
Under equality of the actual sign-stage incoming and final carries,it proves
the natural cross sums for two's-complement balanced differences:

```text
unsigned(x) + unsigned(y) + incoming + 2^width * sign(result)
  = unsigned(result) + 2^width * (sign(x) + sign(y))
```

The ordinary theorem also preserves every arbitrary initial Nat on both
sides. The guard is an explicit equality between computed carries. A caller
true flag cannot replace it. Go addition itself keeps its modular result
when this extra mathematical no-overflow claim is unavailable. The definition
NoOverflow uses the actual two ripple carries; finite observations independently
check its equivalence to the signed mathematical range condition. The logical
Int language and its remaining algebra are still separate required work.

The [13 source-typed rows](source-values.json) identify the exact preceding
GP-OP-05/add Value and Defined declarations. Complete source-context/operation
reconstruction runs before copying them unchanged. Each receives a checked
word-value definition equality and a Nat or conditional signed arithmetic
theorem about that exact Value constant. Distinct Amount/OtherAmount and
int/int64 identities keep distinct operation hashes and theorem names. These
are proofs of the value component's arithmetic; the selected Go AST/evaluation,
state,permission,frame and enclosing function claims remain unproved.

Two initial positive proof constructions reached existing conversion fuel.
The first used Carry0 as a wrapper where the retained root theorem used the
incoming parameter. Using that exact parameter fixes root correspondence.
The second implicitly converted an actual source Value wrapper inside a
whole Nat equation. Rust exhausted conversion fuel while Go accepted that
candidate; both raw outcomes are retained. Proving word equality first and
lifting it explicitly through sign/value/Nat contexts makes both checkers
accept every final positive. Neither checker nor its fuel ceiling was widened.

The [replay](../../../../probes/go-practical/run_numeric_bridge.py) rebuilds the
unchanged77-source joint private checker proposal and retained ordinary emitter
in a fresh temporary workspace. It emits each of20 certificates twice with
identical bytes. Four full-width sets pass both checkers with zero axioms and
matching certificate/export/axiom hashes. Fourteen deliberately incorrect local
carry,leaf projection,overflow premise or dropped-carry statements fail by
ordinary type mismatch. The32/64-bit dropped-final-carry candidates instead
exhaust conversion fuel in both checkers. Those two are failed candidates,
not evidence of a certified false source claim. No timeout/format failure is
counted as an ordinary negative.

The64-bit positive has1,159 declarations,25,583 terms and337,297 bytes,well
inside the preceding candidate artifact envelope. Observed direct proof-child
time/RSS is local Darwin evidence; it is not a Linux release or process-tree
resource guarantee. Production core/kernel,Certificate v0 and installed
profiles remain unchanged.

The [288 queries](queries.json) include both incoming carries,signed extrema,
overflow,high-bit patterns,fixed-seed operands and initial naturals0,17 and
10^50+37. The [independent Go observer](../../../../probes/go-practical/numeric_observer.go)
runs actual typed signed/unsigned addition,math/bits carry extraction and
math/big exact sums. Two identical [native outputs](native.json) agree with
the [actual ordinary ripple/carry/guard evaluation](model.json) and a separate
integer oracle. All unsigned carry equations agree;210 signed no-overflow
cases satisfy the cross sum and78 overflow cases do not. This finite comparison
does not execute2^64 unary successors or substitute for the universal checked
Nat theorems.

The [receipt](observations.json) pins the complete current source/data set and
the retained14,496-file Go1.25.0 inventory. The [12 targeted tests](../../../../probes/go-practical/test_numeric_bridge.py)
cover exact native evidence,isolated statement mutations,all binary blocks,
nominal/source references,current ripple observations,unbounded initial values
and the distinct fuel/pending boundaries. The
[verification record](verification/verification.json) retains raw local logs,
selection reasons and both initial conversion findings. Unchanged older
matrices are not rerun; the relevant seed and preceding source/evidence pins
are checked.

General Nat/Int and contract operations,multiplication/division/shifts and
their source correctness,all remaining state/permission/frame/control/totality
definitions/proofs,and complete Go statement/program/skeleton assembly remain
due. Frontend-envelope/policy/API/release roots and reviewed migrations remain
W04 work. W05 owns inactive publication and the cumulative T01 gate. All
T02–T11 implementation/application/release gates remain active.

```sh
python3 -B develop/probes/go-practical/run_numeric_bridge.py --check
PYTHONPATH=develop/probes/go-practical python3 -B -m unittest -v test_numeric_bridge
python3 -B develop/probes/go-practical/check_numeric_metadata.py
```
