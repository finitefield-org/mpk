# T06-W04: loop and pattern VC handoff

This directory records source-bound, pending control obligations. The immutable
`ControlVcProgram` is reconstructed from independently imported VIR and source
captures, and is committed by digest and sequent identities in the unchanged
`mpk.vc.v3` wire. No discharge flag or semantic axiom is accepted from a caller.

- `loop-requests.json` reproduces the original 27 requests from the T04-W06 test;
  responses remain in `../control-emission/loop-responses.json`.
- The 34 original pattern captures remain in
  `../control-emission/source-cases.json`.
- `measure-requests.json` / `measure-responses.json` retain four original-source
  Roslyn captures: increasing variable measure with identical requires/ensures, partial with no measure, rejected
  total with no measure, and a loop descending from 3 to 0. New responses were
  produced with the existing Linux `--test-control-emission-requests` harness.
- `goldens.json` pins the complete handoff hashes and counts for the 61 original
  loop/pattern cases. `verification.json` and `review.md` record local checks.

Contract subjects are observed at the method entry, loop header, or after an
identified edge. A post-edge header observation differs from that header's
previous iteration observation. An explicit function-entry edge covers a header at entry. Exact source load/store/pattern-bind anchors
constrain logical slots, including contract-only variables absent from pruned
native SSA. Only parameters start assigned; used local reads require assignment.
Loop declarations retain modifies sets. All actual CFG edges and their guards
remain available, including explicit break/continue, exception and return paths.
Normal return postconditions occur once per return, not once per enclosing loop.

Strict decreases is the disjunction of lexicographic alternatives: equal earlier
components and a strictly smaller next component, with separate nonnegative and
definedness goals. Partial mode permits an omitted measure; a requested total
mode is not a proof. Generated construction loops retain their native header
phis and W03 ownership evidence as unresolved rank requirements.

Pattern recipes retain the complete original graph once per function, exact
source/native step and result bindings, original successor/arm/guard order,
property getter obligations, and the single governing SSA value. Nested pattern
steps belong to their innermost decision. Compiler exhaustiveness hints are not
proofs: a reachable no-match path is an explicit modeled exception. Control
normal/exception edge guards are complementary; concrete W03 checks reuse its
ordered failure formulas, while non-data exception predicates remain with W05.

W09 must expand source/native equivalence, slot flow, generated-loop ranks and
property totality recipes into ordinary definitions and prove the pending goals.
The arithmetic and Boolean test oracles here demonstrate positive/negative
formula cases and ordering; they are not kernel receipts. W05 owns exceptional
search/unwind composition. The full local gate is deferred to the final T06-W12.
