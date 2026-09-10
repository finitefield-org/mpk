# W09 unit 3 observer lifetime correction (in progress)

The ordinary-term test evaluator previously stored a suspended term and its
captured environment in every `V::Thunk` alias, separately from the shared
evaluated-result cache. Even after evaluating a closed Bool, surviving aliases
retained the obsolete environment. A weak-reference regression failed against
that implementation at `weak.upgrade().is_none()`; the failure log is
`/tmp/mpk-w09-thunk-release-before.log`.

The shared suspension now has mutually exclusive Pending(term, environment)
and Evaluated(value) states. Forcing Pending executes the same term in the same
environment. Completing evaluation replaces Pending with Evaluated for all
aliases, releasing the obsolete environment. An actual returned closure retains
its own required environment. No term reduction, condition, failure ordering,
Boolean memo key or evaluation transition is skipped or added. There is no
recognition of a generated foundation operation in this evaluator, and no
production generator or checker rule changes.

The two direct regressions distinguish obsolete environment retention from a
necessary captured environment in a returned closure. Existing poison/unused
argument tests retain call-by-need behavior. Additional scoped checks exercise
dense/sparse input selectors, cube helpers, aggregate modes/saturation/short
circuiting and computed-word sharing, plus source-domain/collection and composed
unit-3 consumers. Their terminal results are recorded separately; merely
launching a process is not acceptance evidence. Eight low-level tests passed
(40.92 seconds), all four source-consumer tests passed (49.28 seconds), and scoped
lint/format passed. The computed-word work counts remained exactly 1,967,080
shared and 15,628,958 repeated transitions. The aggregate poison-input test's
old Thunk constructor initially failed compilation and was migrated to the same
factory. No further actionable issue was found in this correction review.
Exact commands and hashes are in `unit-3-observer-lifetime-progress.json`.

The earlier recursive-domain process ended with SIGKILL, whose cause remains
unproven. This correction addresses a demonstrated lifetime defect and must
not be represented as proof of that termination's cause or as completion of
the outstanding source semantic checks. Existing long-running processes were
left intact while the correction was tested. Any subsequent replacement must
retain its original log, exact scope and explicit implementation-change reason.
After the scoped checks passed, confirmed old capacity-test PID 79060 was
explicitly stopped with SIGTERM and replaced by the full original suite using
the corrected observer and current writer. Its log and replacement reason are
retained in `unit-3-entry-collection-progress.json`. The three unfinished domain
tests are rerunning serially; no semantic requirement was removed.

## Environment destruction

The serial domain run v6 then aborted with stack overflow in its first decimal
case; no domain subtest completed. Although evaluation itself uses a worklist,
unreachable captured environment chains still had recursively generated Rust
destructors. Environment references now enqueue uniquely owned environments
on a per-thread release worklist. Shared references retain their owners; queued
values are dropped without holding a queue borrow, and nested releases append
instead of recursively draining. No input is evaluated during destruction.

Nine low-level tests passed with the new environment representation, retaining
the exact previous evaluation transition counts. The added 128-KiB-stack test
releases 100,000 bindings. A strengthened version mixes environment-tail edges
with captured poison suspensions, verifies that aliases retain the leaf until
the final owner is dropped, and then confirms reclamation; it passed in 0.03
seconds. This addresses destructor depth without increasing source-test stack
limits or dropping any semantic case. All four source-consumer tests passed
again (53.77 seconds), as did scoped lint and format after applying the one-line
rustfmt correction in the mixed-chain test. Review checked ownership transfer,
shared-alias retention, absence of queue borrows while dropping values, per-thread
isolation and unchanged lazy evaluation. No further actionable issue was found
in the scoped correction. The same three domain tests are running as v7 and
still require complete semantic results.

Internal unit 3 and original units 4-8 remain outstanding. This observer supplies
finite observations, not universal application VC proofs. The T06 full gate is
still deferred to W12; no new unit-3 commit or W09 completion is claimed.

The replacement integer map/set domain/capacity run v2 subsequently passed
both 4096-slot source cases in 4,341.47 seconds. This run used corrected thunk
lifetime and the current three-argument writer, before iterative EnvRef
destruction; it is not evidence for that later observer version. The completed
log hash is retained in the progress record.
