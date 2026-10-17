# W04 closed typed overlay draft

Status: private design/type/scope probe, not a published schema, production
frontend, independent normalized artifact validator or discharged theorem.
Entry: `48ab07a0622c3d5993070f52ddb9eb3b58cdeced`.
The [receipt](observations.json) retains two identical native observations of
the intact [Go source](../../../../probes/go-practical/overlay-fixture.go.txt)
and 24 source-overlay vectors: five unproved normalized candidates, one with
missing variant candidates and 18 configuration rejections. The original file
includes an unselected map function; it is type checked without deleting it.

The [operator inventory](operators.json) closes 70 expression heads. The
[normalizer/type probe](../../../../probes/go-practical/overlay_contracts.py)
and [vectors](vectors.json) use native Go1.25.0 type/scope facts. Twenty
targeted tests include a typed witness for every head, nominal types, result
phases, old snapshots, finite binders, UTF-8 spans, shadowing and error-cell
plainness. A well-typed candidate establishes no source binding or proof.

## 1. Objects and identity order

The private source schema is `mpk.go_practical.source_overlay_draft.v0`.
Its required root keys are exactly:

```text
schema, source, function, termination, requires, ensures, error_inputs, loops
```

The only optional key is modifies, an empty array. Source is the exact captured
logical file key. Function is {name:string}, or {name:string,receiver:TypeRef}
for a method; receiver aliases resolve to their captured target. Receiver
precedes ordinary positional inputs. Termination is explicitly partial/total.
Requires is a Boolean-clause array; ensures is nonempty. Empty requires does
not remove profile entry-domain, read-only input or no-external-mutation
conditions. An overlay grants no ownership or external write permission.

A loop row contains selector and nonempty invariants, with one optional Nat
variant. Selector is exactly {kind,start_byte,end_byte,ordinal}; kind is
for/range and spans are physical UTF-8 [start,end) within the selected function.
Unknown/stale/duplicate bindings reject. Missing rows synthesize true invariant
candidates. Missing total range variants synthesize captured-length minus
index; missing other total variants leave strategy/proof pending. Generated
candidates never become extra caller assumptions.

Source overlays contain no capture/normalized-contract digest. Seal source and
raw-overlay bytes first, then normalize against that capture. A derived
contract digest cannot enter its own capture. The probe demonstrates this DAG
with its own domain-separated digest. Its content hashes, native type spellings
and position-based decl/binding keys are private observer keys, not frozen
production TypeID/SourceObjectID/FunctionDeclID or hash preimages. Exact
successor identity and migration work remains open; no legacy crossing is
authorized.

The private normalized schema is
`mpk.go_practical.normalized_overlay_draft.v0`. Loops carry all eight LC01
LoopRef fields, resolved header bindings and an explicit variant union:
{kind:absent} or {kind:candidate,origin:supplied|generated_range,expression:Expr}.
Candidate origins, missing-strategy records, entry storage and unproved status
are explicit. Production import must independently reconstruct typing,
scope, source/loop targets and obligations. This Python producer does not
provide that independent validator.

Normalized expression heads replace local with binding (op,id). Field objects
also carry owner_type and replace the field spelling with its resolved key.
Type/case references resolve aliases. Other shapes are the source shapes;
the inventory lists both exact 70-head sets. A normalized root has exactly
schema, capture_context_id, source_object_id, function_decl_id, termination,
requires, ensures, error_inputs, modifies, loops, entry_storage,
pending_candidates and proof_status. A normalized loop has loop_ref,
invariants, invariant_origin and variant. These are exact private candidate
shapes, not authority to extend an installed schema.

## 2. Closed expression grammar and types

Logical Bool, mathematical Nat/Int and source Go types are distinct. Source
types retain nominal identities; aliases retain their targets. Exact Root,
Plain and acyclic global Error<U> rules remain TG/ER rules. A nonplain payload
cannot be hidden by choosing a smaller universe for []error.

Every expression has op. Unless a special shape is listed, its only other
field is args, a fixed-arity array. There are no lhs/rhs/value synonyms,
implicit conversions, null expressions, executable adapters or arbitrary
source calls. The following table lists every closed head:

| Heads | Shape/type |
| --- | --- |
| `bool.literal` | op,value:JSON Boolean -> logical Bool |
| `nat.literal`, `int.literal` | op,value:canonical decimal text -> Nat/Int; no plus/leading zeros/negative zero |
| `word.literal` | op,type,value:representable decimal -> exact source integer |
| `bytes.literal` | op,type,value:lowercase even raw hex -> source string; arbitrary bytes |
| `source.zero` | op,type -> admitted Root zero value, no allocation/ownership |
| `input`, `result` | op,position:nonnegative JSON integer -> exact positional input/return type |
| `local`, `bound` | op,name -> exact live header binding/scoped logical Nat |
| `range.index`, `range.length` | op only -> current-header Nat counter/captured length |
| `old` | op,value -> evaluate complete child at entry with fixed logical binders |
| `if` | (logical Bool,A,A) -> A |
| `bool.not`, `bool.and`, `bool.or`, `bool.implies` | unary/binary logical Bool -> logical Bool |
| `bool.value` | source Bool/named Bool -> logical Bool |
| `nat.add`, `nat.mul`, `nat.sub` | (Nat,Nat) -> Nat; sub is truncated |
| `int.add`, `int.sub`, `int.mul` | (Int,Int) -> exact mathematical Int |
| `int.neg`, `nat.to_int` | Int/Nat -> Int |
| `nat.from_signed`, `nat.from_unsigned` | exact signed/unsigned source word -> Nat; signed needs nonnegative proof |
| `int.from_signed`, `int.from_unsigned` | exact signed/unsigned source word -> mathematical Int with actual sign/value |
| `eq`, `ne` | exactly typed logical or comparable source pair -> Bool; error/slice and aggregates containing them excluded |
| `lt`, `le`, `gt`, `ge` | exact Nat/Int/source word/string pair -> Bool; strings ordered by bytes |
| `word.neg`, `word.not` | source integer -> same nominal type |
| `word.add`, `word.sub`, `word.mul`, `word.and`, `word.or`, `word.xor`, `word.andnot` | same-type source integers -> same type, Go wrapping width |
| `word.div`, `word.rem` | same-type integers -> same type, signedness preserved; divisor nonzero, minimum/-1 defined |
| `word.shl`, `word.shr` | integer and any admitted integer count -> first type, full count bits |
| `word.convert` | op,type,value -> explicit exact nominal integer target, Go truncation/extension |
| `field` | op,value,field:declared name -> exact direct field; normalized owner type/key, no promotion |
| `length` | array/slice/string -> Nat length |
| `capacity` | array/slice -> Nat capacity |
| `is_nil` | slice/error -> Bool; non-nil empty/concrete zero remain non-nil |
| `byte_at`, `element` | string/array-or-slice,Nat -> uint8/exact element |
| `bytes.prefix`, `bytes.suffix` | exact same-type strings -> Bool byte relation |
| `bytes.concat` | exact same-type strings -> same type with result-domain obligation |
| `bytes.slice` | string,Nat lo,Nat hi -> same type |
| `slice.view2`, `slice.view3` | slice,lo,hi[,max]:Nat -> same nominal slice type, full-capacity observations |
| `error.is`, `error.payload` | op,value,case:member of U -> Bool/exact concrete payload |
| `sequence.equal` | arrays/slices with same comparable element -> logical length/content equality, not source slice/storage equality |
| `forall`, `exists`, `nat.sum`, `int.sum` | op,name,lower,upper,body; fresh Nat index in finite [lower,upper), Boolean/Nat/Int body/result |

Intervals are empty when upper<=lower. Quantifiers/folds add no unbounded
quantifier or machine-width cutoff. Sum means an exact ordinary fold, not an
uninterpreted promise. Multiplication, signed observations, folds and bridges
need zero-axiom ordinary laws and complete budget evidence in the foundation.
The type probe does not establish those gates. The literal string carrier uses
the existing candidate 256-byte bound. The private construction guards
(depth 128, nodes 16,384, decimal text 4096 characters) are operational probe
ceilings, not semantic Nat bounds or frozen profile limits; exhaustion is a
separate resource exception. Final counters/bound identities remain open.

## 3. Phases, state and definedness

Input is the invocation's entry argument value; parameter reassignment does
not change it. A descriptor obtained from input observes current permitted
backing when an element operation executes. Local observes actual header
bindings, including initialized named results. Result exists only in ensures
and belongs to the same invocation's ordered return product. Thus
old(element(input,i)) freezes the read; element(old(input),i) freezes only
its descriptor child. Old forbids locals, results, current range counters and
nested old, but preserves immutable logical bound variables.

Native scopes distinguish shadowed names. For-init variables exist at the
first header; body/future variables do not. Range := variables are unavailable
before their first iteration assignment. Existing range = destinations remain
visible, and already assigned outer range variables may be visible at a nested
header. The probe covers named-result shadowing and both range forms.
Package variables are excluded from header locals even when Go's lexical
lookup can find them. The unselected fixture may use ambient state; that does
not turn it into an admitted contract observation or verified dependency.

Well-typed partial observations need separate obligations, never hidden entry
requires. Signed-to-Nat needs value>=0; division/remainder needs nonzero divisor;
shift needs nonnegative signed count with no upper-count truncation. Index
needs index<logical length. Slice views need lo<=hi<=capacity or
lo<=hi<=max<=capacity; substring uses byte length. Concat needs carrier-bound
domain proof, not a fabricated panic. Error payload needs the selected tag.
Quantifier/fold bodies must be defined at every index; an existential witness
does not excuse other undefined observations.

All child obligations remain. And/implies guard right definedness by the left
predicate; bool.or guards it by the left predicate's negation. If guards only
the chosen branch. Both branches
must nevertheless be well typed. Old shifts child observations/guards to entry;
bounded bodies retain their binder. Independent VC reconstruction must enforce
this complete definedness/domain calculus. The probe currently checks syntax,
types and scope; it neither emits nor discharges that calculus.

Every error-bearing entry path needs an explicit allowed non-nil case set.
A selector is (input position,path), with direct field keys and elements for
all array/accessibly backed slice cells, including tails. Empty permits nil
only. Aliases normalize; omitted/unknown/duplicate/non-global cases reject.
Aliased paths impose both restrictions on shared values and imply no
disjointness. Production must recompute global U from reachable boxing and
explicit sets; the probe's supplied fixture U is not trusted closure evidence.

## 4. Header scheduling clarification

LC02 stays authoritative: invariant/rank are at h before condition evaluation.
Encode Live(h) or a stopped exit with its real state/payload. H reads only the
stopped tag. I/R extend to stopped states with canonical ordinary values,
without imposing preservation/decrease on them.

N(Live(h)) evaluates Test(h)=(b,t) once. False produces stopped false-exit(t).
True executes body, consumes the proper targets, then stops on break/return or
executes binding transition/post and returns Live(h'). The next condition is
on the next N step. Init/range capture precedes the first header once. A body
backedge whose later test is false still decreases rank; the false-test exit
step itself does not.

This replaces the preceding variant README's separate initial test and
body/post/next-test proposal. Abstract H/N/Step theorem/certificate bytes are
unchanged. Rank+1 counts complete header steps, including the final test-only
exit. Every entered condition/body/post/callee/nested loop needs its own
totality/safety/domain and source/frame proofs. Source composition remains due.

## 5. Remaining work and ownership

R018 retains practical_contract Go producer/test ownership at T02-W03. The
successor Rust importer must independently reconstruct all normalized types
and bindings. Practical_loops and ordinary VC foundation own guard/control
composition at T06-W03/W04. The private schema labels change no installed
identity, legacy parser, Certificate v0 or existing profile. Exact successor
schemas/hashes, counters, source/effect admission and exhaustive migrations
remain W04 work. W05 owns inactive publication and the cumulative T01 gate.

The [verification record](verification/verification.json) retains native replay,
targeted tests and metadata/format checks. Unchanged production/core proof
suites need no repeat for this producer-only design probe. T01–T11 remains
active; this checkpoint completes neither W04 nor a release.
