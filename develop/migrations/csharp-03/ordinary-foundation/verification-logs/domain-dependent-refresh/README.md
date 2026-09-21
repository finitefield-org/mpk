# Public and boundary consumers of the domain correction

All five delegated checks have passed. Every one of the eighteen changed
certificates now passes both checkers, and both complete runtime tests pass.
See `user-long-completed/verification.json`; no checks remain pending in this
handoff. Other downstream consumers and unit/W09 obligations remain open.

This is the next affected-consumer slice after all 103 certificates in
`../domain-pin-refresh/` passed. It preserves that completed evidence and
refreshes public domains, public defaults, the integrated public profile, and
the integrated boundary profile. These consumers embed the changed recursive
domain/aggregate helpers. Original-source generation, metadata mutations and
transitive definition-closure checks are included in the selected tests.

Twenty candidates were regenerated: 18 changed certificates and two unchanged
enum certificates. `changed-pins.json` records exact old/new hashes and
`previous-pins/` retains every replaced file. All four generation tests passed.
Fifteen changed certificates passed agent same-byte Rust/Go checks with zero
axioms, report agreement and corrupted-hash rejection. The three large document
certificates reached the 55-second agent limit and subsequently passed in the
user run. No timeout is treated as a semantic rejection or acceptance.

`reconciled-runtime.json` links 34 existing passing tests to the current observer
and unchanged domain/scalar/aggregate producers: six domain/source checks,
eight aggregate/zero-region checks and twenty observer regressions. They were
not rerun. The original 65,536/65,537 total-cell boundary was rerun with the
current observer and passed in 42.82 seconds. The other new runtime attempts
and their limits are recorded separately; `verification.json` states the final
scope and status, while earlier attempts remain historical diagnostics.

Both lists in `pending-long-checks.json` are empty after completion; the exact
executed manifest is archived in `user-long-completed/manifest.json`. The full
aggregate passed in 42.55 seconds and the eight decimal domain-order cases in
92.64 seconds. A future affected aggregate run is agent-owned; the other four
checks are user-owned. The runner will not repeat completed tests.

For a populated handoff, `run-long-checks.py` validates
certificate, checker and runtime-source hashes, runs only the pending tests,
requires actual passing test results, and writes a fresh log directory. Go
checks have a 25-minute process timeout; runtime evaluation has a 600-second
deadline and no step cap. Runtime filter/generation environment variables are
cleared. A failure stops the script and preserves all completed logs.

Unit 3 remains incomplete: other downstream clause/data fixtures and the full
unit review are outstanding. Native invocation/control, transition and final
proof assembly remain W09 requirements. No component commit or W09 completion
receipt is issued. `check-fast.sh` remains deferred to T06-W12.
