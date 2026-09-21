# W09 string SSA data relations — scoped review

Current follow-up: see `unit-5-string-construction-runtime-review.md` and
`verification-logs/string-data/hoisted-index/verification.json`. Constructor
sharing changed emitted bytes and moved the accepted all-char arity from 231
to 234 under the unchanged 256-binder cap. Earlier results below retain their
historical scope; replaced pins are preserved. The reported interpolation test
now passes; the user also completed the constructor oracle, binder checker and
all remaining string runtime partitions. Final scoped evidence is in
`verification-logs/string-data/user-long-completed/verification.json`.

Reviewed the new string-data emitter, shared native cache path, raw substring
checks, source tests and same-byte checker harness. Static review identified no
actionable finding in the following boundaries; full runtime verification is
still pending and this is not a completed component or original-unit receipt.

- Reconstruct signatures from the validated closed context before admission.
  Nullable/non-nullable cache reuse and altered signatures reject. Existing
  unary/binary contract admission remains unchanged; native calls admit larger
  arities through the same checked scalar emitter.
- Preserve complete original scalar dependency closures. Raw substring range
  and output-bound checks are distinct from the ordered legacy failures, so a
  preceding null/range failure cannot silently change the W03 raw formula.
- Compare the complete physical result representation. Closed recursive-depth
  equality definitions cover all addresses; a Let shares the computed result.
  No host operation computes a production result or discharges a proposition.
- Keep original SSA subjects, success/failure guards, successors and static
  obligations. Reject unexpected tagged-result checks; retain all unrelated
  definitions explicitly as pending.
- Reconstruct exact metadata/certificate bytes on import, under existing byte
  and certificate-structure limits. Source successor and certificate mutations
  reject. Both checkers receive identical bytes and retain zero-axiom rules.

Four core tests pass: complete-carrier comparison and bit corruptions,
57 legacy nullable/non-nullable operation closures, 33 raw/ordered failure
observations, and three legacy larger-arity bodies with raw substring cases.
Eight source candidates and requests pass; the final pin replay also preserves
the six existing string-contract and eight calendar-data certificates.
Both checkers accept all eight new certificates, across the preserved six-case
run and the additional two-case run. All five affected inventory tests pass
after recording the single added standard-namespace consumer. Scoped lint and
format checks pass.

The source runtime suites retain full physical comparisons, malformed result
rejection, null/range/output-capacity cases and original bounds. Their current
low resident memory is not a peak bound or a semantic verdict. Do not replace
their pending status with the checker or comparator regression results.

Original units 3–8, application/native-body proofs and W09 remain incomplete.
The repository-wide gate stays deferred to T06-W12. No original internal-unit
commit boundary has been reached.

## Current runtime follow-up

Per-case progress and exact operation selection were added without changing
semantic cases or certificate bytes. All eight candidate manifests remain
unchanged. Basic/ordinal passes cover 22 typed signatures and 130 observations.
Exact selection passes and unmatched selection rejects zero observations.
Clippy, formatting and script syntax checks pass. The remaining 26 signatures
are partitioned without duplication in `string-data/pending-runtime.json` and
remain user-owned. The first concat.string2 case exceeded the 45-second agent
diagnostic ceiling; do not interpret that as a semantic rejection or repeat it
unchanged. Full current receipts are in
`verification-logs/string-data/current-runtime/verification.json`.
