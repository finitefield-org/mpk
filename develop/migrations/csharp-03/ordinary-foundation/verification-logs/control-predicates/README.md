# W04 predicate definition checkpoint (partial unit 5)

The adapter retains all 215 original W04 sequents in the 18-source control
corpus and their exact assumption/goal bindings. It defines 383 of 500
predicates and 98 logical implications. The 117 remaining predicates name
their missing constants explicitly: 102 pattern-step and 21 noninteger
data-failure symbol occurrences (some predicates contain several symbols).
No missing predicate is replaced with an assumption or constant truth value.

`NonNegative`, `MathLess`, and `MathEqual` use ordinary Boolean terms for all
eight fixed-width signed/unsigned integer types. Signed comparison flips the
sign bit before unsigned comparison, so no modular subtraction overflow
changes mathematical order. Free W04 binding indices are translated beneath
the ordinary argument lambdas; local lambda/let indices remain local. Shared
sequent arguments retain kind, edge, node, value, and nominal type, including
distinct previous-header and after-backedge snapshots.

The logical implication is a Boolean relation over these free observations.
Native execution, slot/SSA transport, loop induction, remaining pattern/data
semantics, and application proof assembly are still required. Every program
retains `application_scope_pending: true`. This checkpoint does not complete
internal unit 5, W09, or any original application proof. W10-W12 remain blocked.

Target selection follows the changed adapter and its W04/scalar consumers:

- Two library tests pass: 1,800 mathematical boundary/bit observations over all
  eight integral types, plus free/local binder and snapshot checks.
- The 18-source pinned regression passes: original VC hash/sequent preservation,
  all argument maps, six original variable-decrease observations, 392 logical
  implication observations, and eight distinct before/after snapshot pairs.
  Strict import rejects metadata, edge-binding, certificate, and source-context
  substitutions. Unused original subjects retain their original carrier types.
- All 19 certificate occurrences (14 distinct byte sets) pass the existing
  Go/Rust binaries with identical module, declaration, export, axiom-report and
  certificate reports, all axiom categories zero, and hash-mutation rejection.
  `checks/stages.json` retains all 76 actual process exit codes. The runner's
  first final-audit attempt used nonexistent report-field names; the retained
  terminal stage reports were reaudited with the actual schema in
  `checker-audit.log`. No checker stage was inferred from a verdict alone.
- Targeted Clippy and format checks pass. The existing control-edge generator
  and its pins are unchanged; the new adapter has its own pins under
  `ordinary-foundation/control-predicates/`.

Raw Cargo logs retain their terminal blank lines byte for byte. The Git
whitespace check covers code, documentation, pins and checker records; those
raw `.log` files are excluded from whitespace normalization and are hashed.

The ongoing Linux real-value test at revision `7c321e4d` is separate evidence
and is not counted as testing this adapter. The targeted Linux run at `5cf114ba`
passed its two library tests and 18-source regression, then failed Clippy with
fourteen existing library style diagnostics. Exact logs, 370 source hashes and
both test-binary hashes are retained in
`verification-logs/server-linux-control-predicates-5cf114ba/`. The equivalent
style repairs and the next integrated adapter have separate current evidence in
`with-execution/`; its Linux lint retry does not rewrite the old revision's result.
The repository-wide `./scripts/check-fast.sh` gate remains deferred to the
last W of T06 (W12), per AGENTS.md. These are scoped test results, not a T gate.

To regenerate checker reports using the pinned bytes, run `tool-sources/check.py`
with `--pins`, `--reports`, `--rust`, and `--go` paths. `--audit-only` verifies
the retained input bytes, report/stderr hashes, process exits and report
agreement without rerunning completed stages. `verification.json` records
the scoped observations, source/test provenance, and outstanding work.
