# Decimal observer memory investigation — 2026-09-14

This section records the initial comparison. The subsequent default-policy fix
and its final verification are recorded below.

The large peak is reproducible with one original `decimal 1 + 2` success
relation. The test input is two small numbers, each using the existing 512-bit
representation; the original acceptance cases were not reduced. Preparation is
small. The evidence identifies retained intermediate evaluation results as the
main contributor in this measured case.

## Isolated process measurements

Each row is one release test process, measured using `/usr/bin/time -l`.
Peak below means physical footprint, not RSS; exact bytes, wall times, exit codes
and commands are in `verification-logs/decimal-memory/measurements.json` and the
adjacent per-process receipts. MiB = 1,048,576 bytes. All rows passed.

| Phase / original case | Default peak MiB | Weak-result peak MiB | Default seconds | Weak-result seconds |
| --- | ---: | ---: | ---: | ---: |
| Source import and emission | 9.47 | — | 0.605 | — |
| Generate and compare pinned add certificate | 30.14 | — | 0.180 | — |
| int32 0 to decimal, success relation | 19.81 | 18.20 | 0.054 | 0.073 |
| decimal 1 + 2, success relation | 8,616.74 | 135.20 | 95.670 | 188.727 |
| Round(2.5, 0, ToEven), success relation | 379.02 | 27.91 | 2.266 | 3.003 |

The isolated addition peaks at about 8.4 GiB, exceeding the earlier 6.4 GB
sample. It falls by about 64 times with weak result retention, while wall time
nearly doubles. These are diagnostic runs, not a controlled benchmark: heap
sampling paused each addition process once, and another long-running test was
present. The weak addition completed normally with a passing assertion.

## Retention evidence and scoped change

The default heap snapshot contains 57,493,352 allocated nodes / 7,297,722,208
bytes, including 28,130,729 allocations of 192 bytes and about 29.3 million of
64 bytes. Requested Rc allocation layouts are 192 bytes for LambdaMemo and
64 bytes for Env or Suspension. Heap size classes alone cannot identify Rust
types or prove a leak; allocation stack traces were not enabled.

The controlled cache-policy comparison provides stronger evidence. The
opt-in `MPK_W09_CORE_CACHE_POLICY=weak-functions` diagnostic stores non-Bit
results for Boolean arguments and unused arguments through the existing bounded
weak identity cache. Scalar Bit results remain strongly cached. It introduces
no argument forcing, operation-specific evaluation or certificate changes.
Expired results are recomputed. The weak heap snapshot contains 1,443,566 nodes /
120,112,992 bytes, with 45,264 allocations of 192 bytes. This supports strong
memoized function/environment retention as the main contributor in this case.

At this initial checkpoint the default cache policy remained unchanged. This experiment was not yet a
default performance fix or a global memory bound. The user subsequently accepted
the recomputation cost and requested that the lower-memory policy become normal.

## Reproduction and verification

`verification-logs/decimal-memory/measure-executed.py` is the exact executed
measurement driver, including the historical local release binary path. The old baseline requires
the observer revision before the default-policy fix; the saved
`baseline/observer-diagnostic.patch` records that comparison implementation.
For the final default use `measure-default.py` and set `MPK_W09_MEMORY_BINARY`.
Rebuild with
`cargo test --release -p mpk-vc --test csharp_practical_vc --no-run` and replace
that path with the resulting executable when replaying elsewhere. Run the
profiles `source generate-add convert-zero add-small round-tie`; for the weak
comparison set `MPK_W09_CORE_CACHE_POLICY=weak-functions` and a distinct
`MPK_W09_MEMORY_OUTPUT`, then run `convert-zero add-small round-tie`. Unset the
cache-policy variable for the default comparison. Every process receipt records
its explicit probe selectors. The probe checks existing pinned certificate
hashes and original-case expectations. It does not replace acceptance tests.

Selected verification reflects the actual edits: both cache modes passed all
12 `ordinary_core_` regressions (lazy demand, sharing and cache semantics).
The isolated source/generation/evaluation probes passed. Clippy for mpk-vc lib
and the csharp_practical_vc integration target passed with warnings denied;
rustfmt for the two changed Rust files passed. No generated certificate changed,
so neither full decimal runtime nor dual checkers were rerun for this diagnostic.
Their previously pending final shared-byte runs are now confirmed passed:
82 runtime observations (4,102.33 s) and all three dual-checker cases (355.521 s).

Scoped review found no actionable issue in probe selection, expected results,
weak cache lifetime/identity handling or preservation of lazy evaluation.
The probe is opt-in; without explicit context it performs no measurement.
Logs and the observer patch are retained under `verification-logs/decimal-memory/`.
The full gate remains deferred to T06-W12. This checkpoint does not complete
original unit 5, units 3–8, or W09, and observer results are not application proofs.


## Default-policy fix requested by the user

Non-Bit results in Boolean-key and unused-argument caches now always use weak
retention. The diagnostic cache-policy environment switch was removed; no
configuration is needed to obtain the memory reduction. Scalar Bit results
remain strongly cached, live function results still share their identity, and
expired results are recomputed by the same lazy evaluator. This change applies
to the test-only observer, not the certificate generator or either checker.

A new lifetime regression keeps the source function alive, verifies sharing
while two result aliases exist, drops both aliases, and checks that the result
closure is released. It then reapplies the function and checks the recomputed
value. Both Boolean keys and the unused-argument cache are covered, including
an invalid unused thunk that must never be forced. The old strong-cache code
would fail the release assertion. All 13 ordinary_core_ tests pass, including
the existing 736 eager/delayed Boolean observations and captured-environment,
cache identity, scalar sharing and lazy-demand regressions.

Verification is limited to the changed observer and its representative decimal
consumers: core regressions, three previously measured original cases, a wrong
round-result rejection, clippy for the affected lib/integration targets, and
formatting for the three touched Rust files. Generated certificate bytes and
checker behavior are unchanged; the full 82-case decimal run and both checkers
are not repeated for this observer-only fix. The full gate remains deferred to
T06-W12. Measurements and final code hashes are recorded in the final-default
receipt alongside the logs. No global memory ceiling is claimed.


Final default-policy verification passed: 13 core regressions, three positive
original-case predicates and one mutated round-result rejection. Clippy and
formatting pass, and scoped review has no actionable findings.

| Original case | Final default peak MiB | Final default seconds |
| --- | ---: | ---: |
| add-small | 134.88 | 173.348 |
| convert-zero | 18.11 | 0.085 |
| round-tie | 28.20 | 3.283 |

The final addition peak is about 98.4% lower than the historical strong-cache
baseline; elapsed time is about 1.8 times the baseline. These are single-run
observations, not promised bounds. The final run was not heap-sampled. The
initial sandboxed conversion assertion passed, but the time utility could not
read kern.clockrate; the successful measurement rerun was outside the sandbox.
Both records are preserved. Exact source hashes, selection rationale, commands
and logs are in `verification-logs/decimal-memory/final-default/verification.json`.
The full gate remains deferred to T06-W12. No original W09 internal unit boundary
was completed by this observer fix, so no component-only commit/push was made.


## Scalar observation cache follow-up

The later domain-runtime work added bounded scalar observation trees without
retaining function environments. The user reran the original decimal 1+2
success case and it passed in 127.59 seconds, at 255,066,688 bytes (243.25 MiB)
peak physical footprint. Against the final weak-cache measurement above,
time decreased about 26.4% and peak footprint increased about 80.4%. This is a
speed/memory tradeoff, not an additional memory reduction. It remains below
the original 8.4 GiB case, but no global ceiling is established. Logs and source
hashes are in `verification-logs/domain-scalar-cache/verification.json`, with
the copied user receipts in `add-small-user.log/json`. This was user execution;
the agent ran only the short conversion/rounding/negative and core regressions.
