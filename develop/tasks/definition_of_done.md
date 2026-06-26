# Definition of Done

A task is done only when all relevant criteria are satisfied.

## For specification tasks

- The document states scope, non-scope, invariants, and rejection behavior.
- Trusted and untrusted data are explicitly identified.
- The document contains enough examples to write tests.
- Ambiguous behavior is resolved by fail-closed rejection.

## For kernel tasks

- Unit tests cover positive and negative cases.
- Errors are deterministic structured errors.
- No parser, source map, tactic trace, or AI trace is read.
- No unbounded recursion or non-deterministic timeout affects acceptance.
- Fuel exhaustion is deterministic rejection.
- Unsafe code is absent in MVP.

## For certificate tasks

- Canonical encoding is tested with golden bytes.
- Non-canonical encodings reject.
- Hashes have domain-separated test vectors.
- Decoding malformed bytes never panics.
- Re-encoding validates byte identity.

## For reference-checker tasks

- Implementation is independent from the Rust kernel.
- Positive and negative fixture verdicts match Rust.
- Any checker disagreement is treated as a release blocker.

## For Go frontend tasks

- Unsupported Go features fail closed.
- The rejected-feature report is exact and deterministic.
- Source hashes and toolchain version are recorded.
- The frontend is not treated as proof evidence.

## For VC tasks

- Generated obligations are stable and hashable.
- Runtime-safety obligations are generated where needed.
- A fixture demonstrates expected VCs.
- VC output is treated as untrusted until encoded in a checked certificate.

## For AI API tasks

- API cannot bypass certificate checking.
- Batch checking returns deterministic verdicts.
- Diagnostics are structured and snapshot-tested.
- Failed candidates do not mutate accepted module state unless explicitly committed.

## For backlog and issue-seed tasks

- `TASK_BACKLOG.md`, `task_backlog.csv`, and `github_issues_seed.jsonl` describe the same task IDs in the same order.
- Every dependency references an earlier task ID or `none`.
- GitHub issue labels are deduplicated.
- Any generated issue seed is regenerated from the CSV source of truth before commit.
