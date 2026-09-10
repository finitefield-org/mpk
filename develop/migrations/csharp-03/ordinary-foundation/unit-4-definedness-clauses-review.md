# Direct review: separate integer contract definedness

No outstanding findings in this component after the corrections and targeted
verification recorded in `unit-4-definedness-clauses-progress.json`. This is not
a full review or completion receipt for an original internal unit or W09.

- The ordinary term comes directly from W03 definedness. Branch guards and let
  values/scopes are retained; no blanket branch conjunction or assumed truth is
  introduced. Its metadata is bound to the original expression and attachment.
- Checked integer signatures, result types and ordered checks must match the
  existing scalar generator. Its failure predicates are disjoint for division
  and remainder, and single-overflow for other supported integer operations.
  Reusing ordered functions is valid here; extending this to overlapping failures
  is explicitly excluded. Scalar generation and checker rules are unchanged.
- Public-domain/default/structural-public consumers reject optional definedness
  until their use-point owner is implemented. They cannot admit a failure-side
  zero, silently narrow a domain, or treat helper checking as an application proof.
- Exact import rejects removal of definedness metadata. Nine source expressions
  and81 checked-oracle cases cover overflow,zero divisors,min/-1 remainder,
  selected/unselected branches,unused checked let values and nested local scopes.
  Prior structural and total source/integrated bytes,metadata and full dependency
  closures remain identical. Both unchanged checkers accept the one new canonical
  certificate with zero axioms and reject its hash corruption.
- The fixture initially used unordered serde_json maps and was correctly rejected.
  The corrected tests emit the frozen field order,pass independent expression
  typing,and use a fresh source context/capture. The preliminary capture is not
  used by the published candidate. A test-only wrong field access was also fixed.
- Inventory detected the new Std. reference. Excluding exactly that one path
  reproduced the previous124-path fingerprint before the125-path expectation,
  aggregate count and dependent hash were updated. Closure verification then
  passed;no unrelated fingerprint was refreshed.

The source/import/core test took106.22s and dual-checker agreement80.954s. Only
changed/possibly affected tests and lint/format/inventory were selected. The
published program and certificate are exact copies of tested outputs, independently
hash/size/metadata reconciled;the81 unchanged cases were not repeated just to
publish those files. The full gate is deferred to T06-W12. All original remaining
work stays open and no component-only commit/push is made.
