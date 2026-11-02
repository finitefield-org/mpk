# Initial compositional word diagnostics

Status: partial W03 feasibility. This is not a completed word workload matrix,
a signed-integer/sum bridge, a Go source binding or a public profile.

The [actual observations](observations.json) retain two diagnostic runs:

| Candidate | Rust result | Go result | Meaning |
| --- | --- | --- | --- |
| 64-bit exact unsigned addition with final carry | Accepted, zero axioms, 198.099 s | Accepted, zero axioms, 476.319 s | Actual universal natural-value conservation |
| 32-bit dropped final carry | Conversion fuel exhausted, 19.460 s | Conversion fuel exhausted, 22.274 s | Failed candidate; not a core type-mismatch negative or a certified counterexample |

The accepted certificate has 423 declarations, 81,514 input terms and
1,043,626 bytes. It quantifies all 128 operand bits, every incoming carry
and every initial natural. Accepted export, certificate and axiom-report
hashes agree across the independent checkers. The
[model description](../word-conservation-model.md) gives the statement and
the ordinary function-equality proof construction. It splits only the local
full adder's three flags, then composes checked lemmas across word widths.

Construction of both 64-bit candidates took 69.432 s with 688,193,536 peak
resident bytes. The accepted 64-bit check used 431,833,088 peak resident bytes
in Rust and 1,018,724,352 in Go. The report preserves per-child CPU time and
raw JSON. Rust used the private optimized release build; the Go reference
binary was built with the pinned Go 1.25.0 host compiler. These local Darwin
measurements establish no Linux release or frozen profile budget.

## Exact source and artifact scopes

The `source/` directory preserves the exact original private Rust files and
measurement helpers. The source inventory also pins all unchanged checker
and foundation inputs. The report's checker binary identities refer to those
actual runs. Current experiments may evolve; do not reinterpret the archived
source as the current producer or regenerate its hashes to hide drift.

`width-32/` and `width-64/` retain their original generated certificates and
indices. Only the two cases with complete raw checker observations in
`observations.json` are the diagnostic evidence here. The 64-bit diagnostic
has one measured construction, not a claimed two-pass full replay.

`batch-artifacts/` contains output from the earlier interrupted matrix.
That runner stopped when the 32-bit negative reached conversion fuel instead
of a type mismatch. Those files alone are not proof-acceptance observations.
The original strict runner and report checker remain archived in `source/`;
no completed replacement matrix or unexecuted matrix tests are advertised.

The current private producer also has a low-result-bit mutation. A one-bit
debug run rejects that mutation with `CORE_TYPE_MISMATCH`/`core_check` in both
checkers. The exact one-bit debug certificate, raw results and producer snapshots are
retained in `low-bit-prototype/`. Its larger-width matrix is not measured. The ordinary existing
27-case representation and four-case local-Nat pilots were independently
replayed against the current source and retain their separate evidence.

## Review and next construction

The retained-checkpoint helper validates source snapshots, canonical raw-byte
identity, both independent results, zero axioms, quantification and the
explicit distinction between fuel failure and a normal negative. It does
not run either checker or elevate this checkpoint to a complete W03 gate.

The measured cost warrants reducing proof construction/checking work before
freezing an application workload. The next candidate keeps exactly the same
64-bit input domain and final-carry statement while representing operand bits
as the existing Bool-addressed cube. A balanced low/high split can keep the
operand binder count constant and reuse half-word lemmas through six levels,
instead of passing every bit through 64 levels. This is an unmeasured design
direction; both-checker proof and resource measurements remain mandatory.
It changes no checker rule or Go source admission.

Signed interpretation, no-overflow/order relations, mathematical prefix sums,
source/model bindings and aggregate boundary measurements remain open.
T01-W03 is In progress, W04 has not started, and the whole T01 gate remains
deferred to W05.
