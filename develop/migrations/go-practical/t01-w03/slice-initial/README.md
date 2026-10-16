# Original positive slice-library diagnostics

These are failed positive proof attempts, not false-claim or completion
evidence. W03 remains In progress. The [exact observations](observations.json)
retain three raw certificates, both checker outputs/resources, binary
identities and two complete private Rust source snapshots. Every input is
bound by SHA-256; unchanged core/foundation inputs resolve against the same
repository files. Current successful ingredients are recorded in
[slice-stores](../slice-stores/README.md).

The `tag-predicate` group retains the original whole library and its earliest
constructor-stage failure. A tag word used a nested Boolean query-equality
predicate. Its value-zero query is semantically the tag, but reflexivity did
not prove that projection under the existing constructor-headed Bool iota
rules. Both checkers report core type mismatch. The revised ordinary
constructor directly branches on its bit-address coordinates, so the tag
projection follows without changing a checker rule or assuming a new axiom.

The `inline-views` group retains the later store/alias-stage conversion-fuel
failure. Inline read/write/view proofs expanded source-width address terms
inside higher-order store expressions. Both checkers reach their existing
conversion budget. Named checked read/write-definition, constructor
projection, equal/different-key and view-observation lemmas now compose those
same full-word/full-cell operations with explicit equalities. Neither the
source domain nor the checker budget is reduced or raised to conceal the
failure. The failed fuel result remains a resource finding.

The private snapshots include any diagnostic-only CLI code present at the
time. They are immutable historical input, excluded from current private
Rust formatting. They do not claim production profile support, an application
theorem, a false source program, or a completed W03 gate.
