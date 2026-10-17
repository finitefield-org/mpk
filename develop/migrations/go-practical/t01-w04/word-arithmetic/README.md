# Full-word addition in the ordinary Nat and Int library

Status: private GO-PRAC-T01-W04 addition interpretation. Entry:
`d796283d544fbc17bd435583ff103353bba7dcbe`. W04 remains open. This connects
actual8/16/32/64-bit ripple addition to the preceding general Nat/Int library.
Source/application correctness and other word operations remain required.

The [catalogue](schema.json) records scope and planned production owners.
The [producer](../../../../probes/go-practical/word_arithmetic.py) starts with
the unchanged634-declaration [arithmetic certificate](../ordinary-arithmetic/theory.mpcert).
The [untrusted exporter](../../../../probes/go-practical/word_arithmetic_export.rs)
checks that seed and the retained187-declaration
[balanced word certificate](../../t01-w03/balanced-words/signed-tree-64-nat-conservation.mpcert).
It compares complete symbolic signatures of22 shared declarations and exports
165 remaining ordinary definitions/theorems. Signatures retain levels,kinds,
types,values/proofs,family and generated flags. Names alone never authorize
replacement. The [exact export](retained.json) and [closure receipt](export.json)
are reproduced twice. Every result rechecks all ordinary members in both cores.

At every cube depth,the [complete expansions](expansions.json) prove:

```text
Given forall n. f(n) = n+k:
  TreeValue(depth,f,word)(n) = n + k*Unsigned(depth,word)
  TreeWeight(depth,f)(n)     = n + k*Power(depth)
```

The coefficient k and initial n are arbitrary unbounded Std.Nat values,and
the whole word stays quantified. The premise is an explicit pointwise equation
for f. Power0=1+1 and Power(d+1)=Power(d)*Power(d); depth d has2^d bits and
Power(d)=2^(2^d). Every named power has a checked ordinary definition equality.
Successor translation follows from AddOne. No function extensionality or
complete operand-pattern enumeration is used.

Combining the interpretations with the retained ripple/balanced correspondence
proves U(x)+U(y)+incoming=U(actual result)+final carry*2^width in genuine Nat.
Bool values select0 or the appropriate weight. The carry is the actual circuit
output. SignedValue(word)=Int.Make(U(word),sign(word)*2^width). Equality of the
actual sign-stage incoming and outgoing carries implies semantic Int Equiv
between the interpreted sum,including incoming,and the interpreted result.
Explicit Positive/Negative computation laws connect the cross sums; equality
of pair functions never supplies semantic integer equality.

The [13 typed Value rows](source-values.json) preserve exact source types,
operation hashes,scopes and prior Value-definition theorems. Checked word
definition equality lifts each actual Value through the new denotations.
Unsigned claims retain final carry; signed claims retain the actual sign-stage
premise. An explicit proof removes the literal zero incoming contribution.
These arithmetic claims keep source_evaluation_proved=false; selected Go AST,
evaluation,state/frame and enclosing-function correctness remain unproved.

The [replay](../../../../probes/go-practical/run_word_arithmetic.py) freshly
builds the unchanged77-source private checker proposal,unchanged emitter and
new exporter. Four positive sets pass both checkers with zero axioms and
matching certificate/export/axiom hashes. Sixteen altered final-carry,overflow-
premise,result-sign or signed-denotation candidates reject by paired ordinary
type mismatch. Every certificate emits twice identically. The64-bit positive
has1,831 declarations,38,672 terms and539,484 bytes. Production core/kernel,
Certificate v0,old helpers,semantic identities and installed profiles do not
change. The artifact envelope and parser/conversion budgets remain unchanged.

The [96 queries](queries.json) cover both incoming bits,signed extrema,overflow,
high-bit patterns and fixed-seed operands at all widths. The unchanged pinned
[Go observer](../../../../probes/go-practical/numeric_observer.go) executes
typed addition,math/bits and math/big. Two identical [native outputs](native.json)
agree with an independent integer oracle. Seventy no-overflow cases satisfy
the signed relation;26 overflow cases stay modular. All
[24 selected8-bit observations](model.json) execute actual new UnsignedValue
and SignedValue definitions over actual ripple results and agree with the
independent interpretation. Larger unary-model denotation runs were not
selected; universal ordinary proofs cover all four widths. Finite observations
do not establish general source/elaborator correctness. The preceding ten
large logical-model pending cases retain their original status.

The [receipt](observations.json) pins729 inputs and the retained14,496-file
Go1.25.0 inventory. The [16 targeted tests](../../../../probes/go-practical/test_word_arithmetic.py)
cover native bytes,isolated controls,shared export closure,arbitrary pointwise
premises,all source identities,actual carry guards,observations and proof
boundaries. The [metadata checker](../../../../probes/go-practical/check_word_arithmetic_metadata.py)
retains preceding arithmetic evidence and all143 inventoried predecessors.
Only the new Rust source is format-checked; there is no new Go source.

The [verification record](verification/verification.json) preserves commands,
raw logs,selection reasons and six initial finding groups:exporter lock
resolution,composed producer context,computed Nat/Int projection typing,large-
iteration conversion fuel,wrong observation-query fields,and embedded temporary
paths in the standalone exporter binary. Exact original
inputs and outcomes remain available. A wrong initial Go CLI invocation is
separate from its proper core result. No construction/format/resource failure
is counted as false-source evidence. Naming powers alone did not repair the
32/64-bit fuel failure. Generic checked ApplyEquality and MaybeTranslation
lemmas give direct result types when instantiated,avoiding beta conversion of
concrete huge iterations. Explicit literal/projection lifting repairs typing.
Both final checkers pass without a budget or rule change. The new exporter
build shares the retained private tool package and dependency lock as a second
binary,and remaps its workspace prefix to a fixed virtual path. Standalone
prefix remapping alone remained nonreproducible. Checker/emitter sources and
their dependency lock remain unchanged.

Other word arithmetic/order/division/shifts/conversions,partial/source
expressions,bounded definedness,independent elaborator correctness,remaining
state/permission/frame/control/totality proofs and complete Go statement/
program/skeleton assembly remain due. All169 complete Go requests stay
unelaborated. Downstream roots,reviewed migrations and W04 freeze remain open.
W05 owns inactive publication and the cumulative T01 gate; all T02–T11 work
remains active.

```sh
python3 -B develop/probes/go-practical/run_word_arithmetic.py --check
PYTHONPATH=develop/probes/go-practical python3 -B -m unittest -v test_word_arithmetic
python3 -B develop/probes/go-practical/check_word_arithmetic_metadata.py
```
