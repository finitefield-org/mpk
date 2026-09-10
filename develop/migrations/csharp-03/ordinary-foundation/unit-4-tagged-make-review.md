# Tagged construction component review (partial W09)

Reviewed the new `tagged_make` dispatch, its original-source tests, and the
shared stored-sum constructor dependencies. The implementation validates the
exact semantic instance, template, arm, arity, null-parameter distinction and
nominal payload identity before selecting a constructor. Result's error type
and Validation's sequence-of-errors type come from the validated arm layout;
physical carrier equality does not replace nominal checking.

The generated recipe is an alias of the existing constructor. Tests confirm
that exact global and its complete dependency closure, rather than comparing
only a few output samples. No alternative tag encoding or padding convention
is introduced. Role bounds are preserved as existing metadata; the helper
does not assert a new public-domain membership theorem for arbitrary raw
payloads. W03 definedness is compiled unchanged from the original contract.

The test scaffold initially represented the generic tagged payload as an
optional boxed value rather than its actual vector field, and later supplied
a one-leaf cube where the evaluator expects a Bool. Both test-only issues were
corrected. No production or frontend validation was weakened. The original
source test now covers all five families, nested construction, 13 contract
clauses, 12 exact constructor aliases and 2,406 storage observations.

No actionable implementation findings remain. The unchanged same-byte
checkers accepted the new candidate with zero axioms and matching reports,
and both rejected actual hash corruption. Final lint, formatting and evidence
reconciliation passed; details are retained in
`unit-4-tagged-make-progress.json`. This is not a full W09 review or completion
receipt; native/control, transition/replay and application proof assembly
remain within the original outstanding scope.
