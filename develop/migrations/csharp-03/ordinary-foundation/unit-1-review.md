# Direct review: W09 internal unit 1

Scope: the approved first internal unit, its generated corpus, exact
reconstruction path, actual certificate limits and same-byte checker driver.
No subagent or review skill was delegated. W09's completion criteria are intact.

## Findings addressed

- Generated numeric-only name components were invalid Certificate v0 names.
  Prefix depth components with D and encode concrete type IDs injectively with
  an alphabetic T prefix. Actual canonical decode and dual-checker tests cover
  generated names, not just term-construction success.
- Shared sequence storage must retain validation's smaller role bound. Preserve
  the invalid-payload bound of 256 while its referenced sequence has 4,096 slots;
  test it in reconstructed original binding layouts. Domain predicates remain
  unit 3, so metadata alone does not claim the bound is proved.
- Repeated static-transformer leaves could otherwise disappear from cost
  accounting through DAG interning. Charge all occurrences cumulatively before
  sharing. Test 16,384 acceptance, plus-one rejection and ordered evaluation.
- The importer capped metadata but the generator did not. Enforce the same
  16 MiB cap on generated metadata before returning the carrier program.
- The new Std.Bool consumer changed the pinned path inventory. Update only the
  corresponding count/path digest after identifying the new carrier module.
- Go internal errors in the negative carrier test must not count as proof
  rejection. Require a typed non-internal verification error, matching the
  general agreement driver's distinction between verdicts and execution errors.

## Final review

No remaining actionable findings in this internal unit. Checked the ordinary
term de Bruijn scopes, pointwise Bool recursion, padding address order, concrete
composition order/counts, source member/exception/closed-instance linkage,
strict reconstruction and inventory effects. The checked-in certificates carry
only carrier/helper definitions; operation semantics, public domains/defaults,
proofs and application acceptance remain the explicit later units.

Normal regeneration tests compare the committed fixtures without updating them.
Both checkers must accept every positive carrier certificate with zero axioms
and reject hash corruption; the predecessor corpus is rerun locally. Exact
commands, result counts and log digests are in `unit-1-verification.json`.
