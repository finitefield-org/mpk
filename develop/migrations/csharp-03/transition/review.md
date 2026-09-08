# CSHARP-03-T05-W04 task review

Scope: exactly pure transition attachment and pending typed obligation recipes.
Review performed locally without delegation.

Resolved findings:

- The success invariant originally reused an expression bound to input State.
  It now explicitly substitutes `next_state` and rechecks that expression in
  the success scope. The actual-source negative-state mutant is checked against
  the emitted predicate and must violate `NewStateInvariant`. Substitution also
  renames colliding local binders and uses a scope containing only the new state;
  a unit regression prevents accidental variable capture.
- Comparing complete logical and emitted source-call signatures also compared
  exception-check lists. A valid Apply with checked arithmetic was rejected.
  Compare the exact call tag, argument types and normal result type; ordinary
  source validation retains and independently checks exception metadata.
- Logical signatures include an instance receiver. A `State.Apply(Command,
  Context)` could therefore imitate three explicit arguments. Attachment now
  requires static source and exactly three declared parameters. A retained
  actual-source instance-method case must fail at the transition Method gate.
- Transition predicates initially lacked the shared constructor, property and
  semantic-binding environment and did not contribute codec-expression roots.
  They now use the same environment/root walk as other data contracts, in both
  emission and independent import, with the shared expression/node limits.
- The observation wrapper initially cast enums to integers, which is outside
  the admitted source conversion subset. Explicit source comparisons expose
  carriers through ordinary admitted operations; no producer rule changed.

Review checks include original capture identity, exact source/binding/member
attachment, enum coverage and precedence, signed 64-bit time classification,
input/success scopes, postcondition rebinding, pending-only obligations,
manifest/source-artifact linkage, source-effect rejection and independent
import reconstruction. Source scalar/array checks remain owned by the common
emitter and importer. No second production transition evaluator is introduced.

Final review: no findings. Both targeted integration tests, the substitution
unit regression, the frozen-profile/package/vector/build-input checks and the
complete local `./scripts/check-fast.sh` gate passed. The full gate completed
121 suites with 909 passed tests and 8 ignored tests, followed by CLI certificate
accept/reject checks. `verification.json` binds the final file and log hashes.
