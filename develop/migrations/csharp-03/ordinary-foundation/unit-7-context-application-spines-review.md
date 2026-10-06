# Context application spines — partial W09 proof-generation fix

Captured source predicates contain nested ordinary applications. The context
Boolean normalizer previously stopped when an application head was itself an
application, even for `((Std.Bool.rec no) yes) condition`. It now exposes the
complete application spine before recognizing the recursor or substituting a
lambda. The shared Builder and its original term insertion order are unchanged.
The new branch is confined to context mode; closed ownership/default generation
keeps its existing behavior.

The regression fails with `Linkage` before the fix. After the fix, both unchanged
checkers accept the true and false nested cases with free branch values and
actual equality hypotheses, with zero axioms and matching report hashes. Both
reject the same deliberately incorrect equality proof at core checking. Missing
and conflicting facts still reject. All seven affected unit tests, Clippy with
library/test targets, and crate format pass. The four earlier context
certificates, including the shared DAG, remain byte for byte unchanged.

Direct review checked argument order, lambda substitution, free-variable shifts,
and the conversion required between the original nested expression and its flat
spine. The tests preserve the original full proposition and every binder; the
normalizer introduces no open auxiliary theorem, axiom, or checker rule.

An independent replay uses the original `is_binding` fixture, complete packed
environment, all scope projections, and exact named refinement types. The first
path now advances to an unknown packed-environment bit, so generation still
returns `Linkage`. All seven original source refinements and all 987 application
proof IDs remain pending. The diagnostic's exit code zero records that result,
not proof acceptance. Resolving the remaining source equations and supplying
complete execution/refinement proofs remain required.

Evidence, before/after logs, exact same-byte checker reports and source hashes
are under `verification-logs/control-predicates/with-context-application-spines/`.
An initial checker harness read failure is preserved externally: Go rejection
reports omit the accepted-report hash object. The corrected harness reuses the
five fully recorded earlier stages, reruns the unrecorded Go rejection, and runs
all six new nested-case stages. The final audit verifies all twelve reports.
Linux replay of this fix must be recorded after its public commit is available.
W09 remains In progress, W10–W12 remain Blocked, and the whole T06 gate
`./scripts/check-fast.sh` remains deferred to T06-W12.
