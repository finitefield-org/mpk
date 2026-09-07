# CSHARP-03-T04-W02 implementation and review receipt

The baseline is `16946c6ae2a82004be03f49ba52726001dd3317f` (T04-W01).
No frozen schema, value representation, specialization engine, installed
bundle, registry or proof-authority path changes in this task.

## Implemented source handoff

`PracticalLoopLowering.cs` consumes W01 source sites and T03 normalized
operations, types, array analysis and sequence-construction steps. It emits a
private typed register/slot CFG. The graph has deterministic node/value IDs,
explicit ordered operands, conditional/short-circuit branches, loop guards,
updates, iteration bindings, nested break/continue targets and return/exception
exits. A foreach collection is evaluated once; strings iterate UTF-16 units.
The structural loop header has a true entry condition followed by the original
source guard, so invariant entry precedes guard side effects and do's condition
runs at its continuation point. No source guard is moved or duplicated.

The opt-in array route reuses T03 ownership states and `RequireWritable`.
It retains break and continue states, scopes active foreach read borrows,
rejects alias/frozen writes and a freeze followed by a write on another
iteration, and carries initialized-prefix/ownership merge obligations.
Loop-free callables retain their T03 normalized bodies. Their data operations,
constructor implementation and logical representation are not replaced by a
collection binding or executed as a hidden implementation.

`prepare_loop_lowering` enforces frozen structural budgets, then attaches and
type-checks W01 contracts before examining phase-8 graph errors. It checks body
hashes, original normalized operands, complete loop inventories, parent and
header linkage, structured targets, dominance and reducibility. Exact type
bindings, source facts and source sequence steps remain available to the
consumer. Source regeneration rejects altered candidate bytes. The result is
private, immutable through its API, and has zero frontend-success artifacts.

W06 owns composition with ordinary data bodies, whole-control VIR emission,
independent import and construction-state elimination. T06 owns the semantic
proofs. The retained count/fill and map/set clauses remain pending; neither a
successful lowering nor a compiler/runtime comparison discharges them. In
particular, the four mapped collection helpers intentionally do not prove their
advertised add/replace semantics. Their purpose is to exercise real source
binding/operation linkage through loop lowering without trusting that linkage
as commutation. Correct sorting, deduplication, counting and replacement traces
are separately exercised by the executable algorithm cases.

## Review iterations

- Exact array foreach has a compiler-inserted collection conversion that the
  default normalizer rejects. Only that implicit conversion directly owned by
  the exact foreach operation now retains its original array/string carrier.
  Explicit interface conversions and arbitrary enumerators remain rejected.
- Iteration-variable ordinals and exact types now extend the existing T03 local
  normalization in W01's specified order; explicit and var graph outputs agree.
- An unconditional structural header places invariant entry before source guard
  effects and has the canonical body/exit shape. Do continues evaluate the tail
  condition; for continues execute the source update list before the backedge.
- A conditional with two abrupt arms initially left an unused empty join.
  The builder removes only that unreferenced join and deterministically renames
  node references. The both-abrupt source differential covers the fix.
- A first-iteration-only uniqueness check could miss a freeze on a continuing
  path. The existing ownership states now reject such backedge writes, and the
  dedicated source negative exercises that rejection.
- Normalized type/source information is retained with the result, including the
  iteration variable's exact type, rather than being dropped after graph checks.
- CFG limits now retain phase-0 precedence over malformed contracts. Native
  1023/1024/1025 boundary tests and a mixed limit/missing-decreases case exercise
  it. Producer counters enforce 1024 blocks per method and 8192 per closure
  before retaining a new node.

- A source binary operation could initially be relabeled as a synthetic loop
  comparison while keeping valid operands. The regression accepted the
  mutation before the fix. Opcode/source-kind linkage now rejects it; missing
  operand identities are also explicitly exercised.

Latest scoped code review: no remaining findings. Runtime, fixture and standard
gate results and exact file hashes are recorded in `verification.json`.

## Verification corpus

There are 32 captured source cases: 26 private handoffs and six artifact-free
source rejections. Twenty-two executable cases compare 528 original CLR runs
with an independent Rust CFG interpreter. Cases include all four loop forms,
explicit/var foreach, UTF-16, nested targets, guard/index side effects, early
return, both-abrupt arms, short-circuit evaluation, one-time foreach collection
evaluation, array fill, count/fill disagreement, membership, lookup, aggregation,
sort, deduplication and duplicate detection/replacement.

Four additional source-defined map/set operation cases reuse the W01 binding
and expression validation harness and check that exact ordering, uniqueness,
bounds and applicable duplicate/insertion-order clauses survive lowering.
These are source-bound pending obligations. The eight W02 native tests also
cover source/hash/target/operand mutations, private input manifests, termination
and diagnostic precedence, and the inclusive CFG block boundary. The nine W01
owner tests remain active in the same suite. The existing 777-case T03 replay
was executed on the modified shared source validators and matched the retained
result byte for byte.
