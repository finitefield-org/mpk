# W04 operation, evaluation and loop-contract decisions

Status: inactive draft, not a published profile or wire-schema freeze.
Entry is `18ac6aee085cefa027ef7064cc1dfdded762f867`.
The [operation table](operation-table.json) has 56 logical operation families
covering v1 priorities1–4. Its GP-OP IDs are design references, not serialized
VIR tags. Exact successor schemas, hash preimages, overlay expression grammar,
limits and migration review remain W04 deliverables.

The [Go evaluation vectors](evaluation-vectors.json) and
[native receipt](evaluation-observations/observations.json) are separate from
production admission and source/model proofs. The observer's counters, recovery,
function literals and JSON aggregation expose native behavior outside verified
code. Positive byte/string copy and append forms retain arbitrary bytes and
named slice/string types. Their full ordinary binding composition and budget
review are still required; native observations do not discharge those proofs.

## 1. Evaluation and lvalues

These rules use the pinned Go1.25.0 bundled specification, whose whole-file
SHA-256 is `42a4ff1c01efe8dc23eca4378b8fb03d76e9f2f079dab01298ffd74d8b0e3149`.
Relevant sections are Order of evaluation, Assignments, Expression switches,
For statements, Length and capacity, and Appending and copying slices.

| Rule | Required behavior |
| --- | --- |
| EV01 | Preserve Go's lexical order of executed calls, method calls and logical operations, including short circuit. Ordinary variable/index reads are not all ordered left-to-right relative to calls. A normalization may choose an otherwise unspecified order only with checked absence of interference/commutation. Interfering or unresolved sibling effects reject; explicit source statements can establish sequencing. |
| EV02 | Parallel/tuple assignment has two phases. First evaluate location operands and RHS values in Go's usual order; freeze indices and any slice descriptor used by a destination. Then store left-to-right. Each RHS call executes once; discarding/projection/forwarding never repeats it. RHS panic precedes all stores; a later store panic does not roll back earlier stores. Safety members are attached to the actual execution point. |
| EV03 | An internal local location is a local-root ID plus a typed field/array-index path. It refers to that location after a preceding whole-root assignment. Existing array storage, including array fields, retains identity and updates all live views when assigned new element values. Crossing a slice freezes its descriptor/region/absolute cell address. Thus a,a[0]=b,9 updates the new array at a, while s,s[0]=t,9 writes the old slice backing. These internal handles do not admit source pointers. |
| EV04 | Reconstruct a field update from the current root/cell at store time, retaining prior stores to its other fields. Do not reconstruct from a stale phase1 aggregate. Compound assignment and ++/-- evaluate their target once. Address/index computations may read admitted values and call statically proved effect-free helpers; they cannot mutate/allocate Go storage. All their calls, checks and operand dependencies still execute once. |
| EV05 | A switch captures its tag once, evaluates nondefault cases in source order until a match, and selects default only if no case matches. A default clause's textual position gives no priority. Tagless switch uses true. Switch consumes only its own break; continue targets the enclosing loop. Fallthrough/type switches/labels/goto stay excluded. |
| EV06 | A call's product, post-state, permissions and dependency proof share one invocation token. Ordered positional results include arity0 and discarded values. Named results start at their Go zero values. Explicit/forwarding/naked return establishes the function postcondition with that same execution and skips every enclosing loop post. |
| EV07 | Integer arithmetic wraps as Go specifies. Division minimum/-1 is defined; only zero divisor needs a division panic check. Shift counts retain every signed/unsigned bit: negative signed count panics, large nonnegative count does not reduce modulo width. Source nominal/assignability rules and full mathematical-index bounds precede address projection. |
| EV08 | Builtin len/cap of an array and array range follow Go's constant-expression exception. A statically known length alone does not authorize deleting a function call or its checks/effects. Compile-time constants are exact; runtime materialization, not an untyped intermediate, must fit the corresponding carrier bound. Conversions use GP-OP-03's closed whitelist, including explicit exclusion of slice-to-array and integer-to-string conversion. |

An array/struct destination handle must differ from a captured slice
descriptor. A field store through a slice also differs from replacing a
whole copied cell: `s[0].A,s[0].B=s[0].B,s[0].A` must retain both updates.
The native examples expose these differences, assignment panic order,
single-call products, default position, short circuit and integer boundaries.

Every operation has independent source/context/type/contract linkage,
input validity, executed safety/domain/permission checks, a complete ordinary
value/effect relation, output validity and frame obligations. The table's
check lists describe obligation classes and dependency order; they do not
move all guards before operand evaluation. Any accepted proof must bind them
to the actual Go execution states and paths.

## 2. Function contracts and effect scope

| Rule | Required behavior |
| --- | --- |
| CT01 | A selected practical root has exactly one explicit overlay with nonempty ensures and mandatory termination=partial or total. There is no default totality claim. This changes no fixed-Go overlay rule. Generated contracts for private callees are untrusted candidates and still require body proofs. A missing root overlay/mode is a configuration rejection. |
| CT02 | Requires applies at function entry. Ensures reads positional results and the same invocation's post-state. Entry observations use an explicit old/entry selector; named result variables inside a loop are header locals, not already-existing function result projections. Unknown/ambiguous binding IDs, wrong nominal types, bounds or universe instantiations reject. |
| CT03 | All accessible root input backing is read-only in v1 and may alias other input backing. Entry states explicitly prohibit concurrent external mutation for the call's duration. Fresh writable storage is derived from tracked allocation/value-array-copy provenance. A helper may receive that fresh permission, return/transfer it and preserve all aliases; it cannot acquire external ownership from a caller assertion. Positive writes require FreshOwned; zero-count copy/append has no write. |
| CT04 | Every call needs a checked precondition/result/effect/frame dependency. A total root also needs termination of every reached callee/nested loop; a partial callee contract alone is insufficient. A separately checked termination member may establish the needed stronger dependency without pretending the original partial contract claimed it. |
| CT05 | Requested safety, postcondition, entry/source binding and frame members are mandatory. Totality adds mandatory termination members. Unknown/timeout/fuel/unsupported strategy/missing candidate is pending; a rejected candidate is not a proof of semantic falsity. Fully checked partial and fully checked total claims remain explicitly distinct. |

The full typed expression AST and normalized contract wire grammar are still
pending. These semantic defaults are fixed draft decisions for that grammar.
The overlay cannot execute Go, introduce an arbitrary semantic adapter, or
observe numeric region IDs/pointers/absolute descriptor offsets.

## 3. Loop references, transitions and obligations

A source LoopRef is the exact tuple
`(capture_context_id, source_object_id, function_decl_id, ast_kind,
start_byte, end_byte, function_preorder_ordinal, effective_language)`.
The span is the physical UTF-8 interval [start,end) of the complete ForStmt
or RangeStmt, including its body. Ordinal is zero-based AST preorder among
loops in that function. Source objects and function declarations are captured
identities, not host paths or printed names. Hash encoding belongs to the
successor wire freeze; a generated CFG block number cannot replace LoopRef.

| Rule | Required behavior |
| --- | --- |
| LC01 | Resolve each supplied loop overlay to exactly one LoopRef and normalized header. Duplicate, absent, ambiguous, crossed source/context/language or stale-span bindings reject. Missing loop entries synthesize invariant true as a candidate; explicit invariant lists are nonempty. Source/contract bindings and initialization/preservation must still be proved. |
| LC02 | Header state h is before the next condition evaluation. Init executes once. Model the complete test transition Test(h)=(condition,t), including its calls/effects/checks. False exit uses t, while true body starts at t. Do not reuse h as the post-test state or assume an effectful test leaves the invariant unchanged. Missing test means true. |
| LC03 | Normal and loop-targeted continue edges skip/execute exactly the proper body remainder, then the Go>=1.22 per-iteration binding transition, then post, then the next header. Init-declared variables are copied to the subsequent iteration before post. Condition-only/empty-post loops still preserve the complete state and correct target. |
| LC04 | Loop-targeted break exits with its actual state, without post and without a false-test assumption. Switch-targeted break is consumed by that switch and continues the enclosing body. Nested loops consume only their own break/continue. Return propagates the product/state to the function contract directly and runs no enclosing post. |
| LC05 | Slice range evaluates its expression and captures descriptor/length once. Index increases from0; fetch the current backing element at its iteration step only when a second nonblank value is needed. Rebinding the source slice cannot change the captured count; writes through fresh aliases remain visible. Nil range has zero iterations. |
| LC06 | Array range copies the array value when evaluation is required; its root descriptor elements are copied shallowly and retain backing sharing. With at most one effective variable, apply Go's constant-expression exception and perform no element read. A trailing blank variable is absent for this rule. Calls in a nonconstant range expression still execute once. |
| LC07 | Range := creates per-iteration bindings according to captured effective Go1.22..1.25. Range = updates existing source locations through ordinary two-phase assignment each iteration, after iteration values are produced. No fake fresh variable or duplicated element/index evaluation is allowed. Integer/string/map/channel/function/pointer-array range stays excluded. |
| LC08 | Prove invariant initialization at first header, each actual test/body/post safety condition, preservation on every normal/continue backedge, each false/break exit, every early-return postcondition and the frame. Structural counter/validity facts generated by lowering are also proof obligations, not extra caller assumptions. |
| LC09 | Partial correctness proves safety for every finite execution prefix and ensures/frame on normal returns; it makes no stopping promise. It cannot drop checks inside a possibly infinite loop. An inferred true invariant may leave required members pending. |
| LC10 | Total correctness requires a single Nat-valued variant per loop, nonnegative/exact representation and strict decrease from h to each next header h'. Break/return edges do not require decrease. Every executed condition/body/post call and nested loop must also terminate. No arbitrary iteration cutoff or collection bound substitutes for these proofs. |
| LC11 | For range, generate captured_length-index as a candidate Nat variant plus proved counter/step facts. For other loops, use an explicit supported Nat expression or a checked generated candidate. A missing strategy/variant leaves totality pending, while the original source/model may still be accepted. A malformed/type-invalid supplied expression rejects configuration. |
| LC12 | Nat measure observations include exact unsigned conversion, signed conversion with a nonnegative proof, finite lengths/capacities, ordinary Nat addition/truncated subtraction and typed conditional measures with branch-local definedness. A negative signed value is not silently reinterpreted or clamped to justify a variant. Full expression schemas and their closed operator vocabulary remain part of the overlay freeze. |

For each loop L, the required members have these meanings:

| Member | Antecedent and conclusion |
| --- | --- |
| init | Entry/preheader binding and function preconditions imply I(h0). |
| test | I(h) implies safe, domain-valid, permission-correct Test(h)=(b,t). |
| body/post | I(h), the checked true test and actual body path imply every executed operation's safety/domain/frame facts. |
| preserve | Every normal/continue path followed by binding transition and post gives I(h'). |
| false exit | I(h) and checked Test(h)=(false,t) imply the continuation's required relation at t. |
| break exit | Each actual L-targeted break path implies that continuation relation at its state; no false test premise. |
| return | Each actual return path implies the function's ordered result/post/frame relation. |
| variant | For total mode, V(h):Nat and every active backedge has V(h')<V(h), with complete source/operation binding. |
| dependencies | Every entered callee/nested loop satisfies the requested mode and its complete checked obligations. |

The loop predicate/measure refers to exact header binding IDs, entry values
and current permitted data observations. Results exist only at return;
source locals shadowing the same spelling remain distinct. A postcondition
about a business sum or output capacity is independent of loop termination.

## 4. Required ordinary termination foundation

The [W03 private Nat induction/control evidence](../t01-w03/control-induction/observations.json)
supports arbitrary-Nat invariant and abrupt-edge feasibility. It does not
already implement a source variant or prove totality of arbitrary loops.
The additive monomorphic Sort0 Nat.induct interface remains a separately
frozen core requirement; existing Nat.rec/Bool behavior is preserved.

The complete foundation freeze must specify and prove these ordinary members:

- Nat predecessor/addition/truncated subtraction and closed less-than/order
  relations with their equations and zero/successor lemmas.
- Strong/well-founded Nat induction: for a Sort0 predicate P, proving P(n)
  from P(m) for every m<n yields P(n) for every n.
- Exact unsigned/signed-nonnegative word64-to-Nat bridges; signed negativity
  is an explicit side condition, never a discarded high bit.
- Range counter initialization, guarded increment, bound preservation and
  strict decrease of captured_length-index.
- A general variant termination composition member using actual complete
  head/test/body/post transitions, proved invariant and terminating component
  calls. Initialization/preservation/decrease are proof premises discharged by
  members, not hidden function-entry assumptions.
- Targeted abrupt exits and source/result/effect/frame composition, including
  switch break consumption, enclosing continue/post and immediate return.

These members must be closed, zero-axiom and checked by both independent
checkers within the frozen budgets. Definition-only acceptance, finite runtime
examples or the existence of Nat.induct does not satisfy those proof gates.
Exact declaration/instance identities and canonical proof descriptors remain
foundation/wire freeze work; new required byte/string copy/append composition
also remains explicit.

## 5. Ownership and next boundary

Every operation row names its planned primary producer/test, independent
artifact validator and VC owner. Evaluation/lvalues belong to practical_lower
and practical_calls; closure/effect admission to practical_closure; loop
attachment/control to practical_contract/practical_loops; ordinary members
to go_practical_foundation. All are planned production files in the
[surface inventory](surface-inventory.json), not completed executors.

Selected validation checks the new native observer, independent expected
facts, catalogue consistency, ownership, exact source/toolchain/artifact pins,
formatting, links and whitespace. It does not rerun unrelated W03 proof
matrices or claim source-to-certificate support. W04 remains In progress.
W05 owns reviewed inactive publication, source-model feasibility review and
the complete T01 gate; T02–T06 still own production/application proofs.
