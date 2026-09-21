# Domain and aggregate certificate refresh

All 103 changed certificates now pass both checkers: 92 agent checks plus the
11 completed user checks. See `completed-checker-results.json` and
`user-long-completed/verification.json`. No checks remain pending in this
refresh; other downstream consumers and whole-unit obligations remain open.

This refresh replaces stale pins after the recorded zero-region, zero-tail and
aggregate-fold performance corrections. It does not change those semantic
implementations or lower any carrier capacity. The domain, scalar and aggregate
producer hashes still match `../total-cell-runtime/verification.json`. The test
observer changed subsequently for lifted/string evaluation, so the runtime
receipts retain their original source scope; this pin-only refresh does not
claim those runtime tests were replayed with the latest observer. Their coverage
must be reconciled before internal unit 3 closes.

`changed-pins.json` records the before/after file hashes; `previous-pins/` keeps
the replaced files. Ten immediate families were regenerated from original
sources. Exactly 103 certificate byte sequences changed. Identical certificates
are listed separately and do not need another checker run for this refresh.
This is not a claim that all downstream W09 clause/data fixtures are refreshed.

The current integrated structural corpus covers 44 original-source contexts
and every expanded structural operation, including uninvoked operations, with
the transition template explicitly deferred to internal unit 6. The exact
component/integrated closure check passes for 118 source-component pairs,
1,235 roots and 12,178 transitive declarations, including rejection of root and
dependency mutations. See `component-closure.log` and the corresponding receipt.

`checker-results.json` records each changed certificate's isolated Rust/Go
same-byte check. Passing tests require zero axioms, matching verification
reports and rejection of hash-corrupted bytes. The agent stops each process
group after 55 seconds; exit 124 means a pending user check, not acceptance or
semantic rejection. `check-short.py` resumes without repeating recorded cases
and refuses to reuse a receipt if its certificate file has changed.

The completed handoff manifest and all 11 logs are archived under
`user-long-completed/`. `pending-long-checks.json` now contains no cases, so
`run-long-checks.py` will not repeat completed tests. The old 55-second attempts
remain unchanged as historical records; `completed-checker-results.json`
combines them with the definitive user results. The measured 57.74-second
sequence decimal-set case is agent-owned if affected again; the other ten
user-run cases exceeded one minute.

The new `verification.json` is the scoped status; older family receipts remain
historical for replaced bytes. Other domain-dependent consumers, source-state
and transition obligations, and final ordinary theorem assembly remain open.
Neither internal unit 3 nor W09 is marked complete. The full repository gate
remains deferred to T06-W12.
