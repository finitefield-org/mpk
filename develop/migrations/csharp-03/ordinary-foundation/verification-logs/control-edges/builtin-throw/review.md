# Captured built-in throw guard review

The previously unresolved `type` exception edge belongs to a captured
`builtin_throw` for SwitchExpressionException. The producer now reconstructs
its source anchor and requires a SwitchExpression ordinal, empty inputs and
normal successors, a native Throw terminator, the exact closed tag-8 literal,
no invocation, one matching exception check and the exact target. Only then
is the local guard defined as true. This expresses execution of an
unconditional throw at that node, not reachability of the node or completion
of handler search. There is no fallback based only on an exception name.

The new guard and complete nullable slot transport are appended after all
existing definitions. The changed source passes 2,807 observations (49 new),
including physical state mutations with the throw guard enabled. A separate
source test checks the original anchor and tag-8 literal and rejects missing
or replaced source-binding metadata. Four declaration preservation tests pass;
the immediate predecessor's entire term/declaration prefix is identical.
All sixteen other edge artifacts regenerate byte-identically. All nine
source-slot pins also replay exactly, covering the shared builder dependency.

Both unchanged checkers accept the identical 105,258 certificate bytes with
matching reports and zero axioms, and both reject hash corruption. The initial
Go build could not access its default cache; the successful rerun uses the
workspace-allowed temporary cache. No checker ran in the failed build attempt.
Targeted clippy and format checks pass. The current seventeen-source corpus
has 704 guards, 667 slot transports, 62 native phi joins and 38,950 observations.
These counts replace previous counts; the separately retained source-binding
regression is not added to the runtime-observation count.

All guards in this corpus are defined. That does not cover every native data
family or every supported source. W04 logical observation mapping, node-entry
merging, non-transfer memory effects, exception values at handler entry,
filter search/finally unwinding, native execution and application proofs remain
open. Unit 5 and W09 are incomplete; the full gate remains T06-W12. No
component-only commit is made.
