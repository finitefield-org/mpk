# CSHARP-03-T04-W01 implementation and review receipt

The user approved the partial-loop prerequisite amendment on 2026-09-07.
The freeze records base commit `5e2979c162e01a1e6b1e006aec4c5d9f566384ee`
and the exact previous freeze/publication hashes. The required `decreases`
field permits `[]` only for an explicitly partial method. Total methods still
require decreases; boundary/transition/example/installed routes remain total.
Four added W01 production vectors bring the published package to 709 rows.

## Implemented handoff

`CSharpPracticalLoopContracts.Capture` is the source half of the private
pre-lowering gate. It reuses the captured-source firewall and the existing
syntax prerequisite validators, with a narrowly selected array/string foreach
analysis route. Original byte spans, method-relative loop IDs, parents, lexical
and definitely assigned variables, transitive writes and structured exit targets
are retained. The original data route continues to reject foreach and all
loop-containing data emission; no source helper is substituted or executed.

`prepare_loop_contracts` is the native attachment/type-checking half. It consumes
the exact selected sidecar bytes and source handoff, rejects stale or mismatched
attachment, and uses W14's strict 33-arm expression parser. Method parameter,
result, old-value, lexical local and bounded-quantifier scopes are distinct.
Binding/member environments and concrete semantic types come from the same T03
projection closure. A collection operation mapping must match the captured
method's projected signature. The returned `PreparedLoopContracts` has no
frontend success, VIR, certificate or proof output.

Collection clauses carry source storage/member IDs, the allocation and length
operand spans, related count/fill obligations and their supporting typed
invariants. Ordered operations retain canonical order, uniqueness, bounds,
duplicate reject/replace policy and applicable insertion-order independence.
These are unproved clauses and obligations. The test helpers intentionally do
not implement their advertised collection semantics: successful attachment must
never be read as proof of commutation. W02 owns first loop CFG lowering,
source-order semantics and alias-aware borrow checking; T06 owns proof discharge.

## Executed cases

`source-cases.json` retains 32 original source selections and their pinned Roslyn
results: 26 source handoffs and six artifact-free source rejections. The cases
include for/while/do; typed and var foreach on arrays/strings; nested and sibling
loops; lexical shadowing; body-only and later locals; break/continue/return and
switch-local break targets; UTF-8 byte offsets; count/fill; borrowed-array writes
retained for W02; ordered set add/count and map add/replace; the 31/32/33 loop and
7/8/9 nesting boundaries; forbidden directives, framework effects and generics.

The nine Rust owner tests execute the handoffs with real captured, hashed method
sidecars. They cover all four forms' attachment/type/precedence failures;
63/64/65 combined clauses; partial/total empty/nonempty decreases; missing,
duplicate, extra, reordered, wrong-target and stale records; exact modifies;
out-of-scope and wrongly typed expressions, forbidden call/old/result contexts;
bounded quantifier scope; rehashed provenance, parent, span, exit and allocation
mutations; and source-derived collection binding/operation obligations.

## Review and fixes

- F01: source-design partial termination contradicted the nonempty frozen field.
  Resolved by the explicitly approved W09/W10 amendment, retaining the required
  field and total-route restrictions. The four new cases execute W01 production
  attachment, not only the schema model.
- Collection clauses initially retained only requirement names. They now bind
  exact source allocation/length/member operands and supporting invariants;
  count/fill tests assert shared allocation identity and reject invalid bindings.
- The new source entry initially bypassed the existing directive/var syntax
  firewall. It now calls the shared prerequisite validators. Conditional and
  nullable-directive negatives exercise that entry and would previously pass.
- Source-project collection expressions initially used the pre-projection
  expression environment. The real set-add invariant regression failed with
  `Expression`. Binding/member environments and closed types now come from the
  derived closure; the same regression and set/map variants pass.
- The standard gate exposed stale private-runner inventory counts and an
  order assertion that searched the entire syntax file, including the added
  helper. Inventory expectations now include the new runner and the order
  assertion selects the original `Normalize` entry. Targeted capture/control
  and syntax tests pass; the existing isolated syntax suite also passes.
- The first unprivileged disposable Linux invocation could not use `unshare`.
  This was an infrastructure failure. Subsequent offline containers used only
  the capability needed by the existing isolated runner, with the repository
  mounted read-only. No release or checker acceptance is inferred from it.

Final scoped diff review: no remaining W01 findings. The prior blocked entry
receipt is retained in ledger section 38 and followed by its approved resolution.
Current task state and the next serial owner are recorded in section 39.

## Verification

The final gate outcomes and fixture/input hashes are recorded in
`verification.json`. The native installed-release gate is outside this private
W01 task and remains with T07/T08; no installed bundle, registry, Certificate v0
or checker implementation changed.
