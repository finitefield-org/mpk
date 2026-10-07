# W09 Boolean cases registration-order review

This follows the user-approved Sort0-dependent Boolean elimination amendment.
W09 remains In progress, W10 remains Blocked, the practical profile remains
inactive, and all 987 original application proof IDs remain pending.

Both pinned checkers accepted a checked definition with `Bool.false.{0}` before
`Bool.cases`, but rejected the same definition after cases registration. The
before/after certificates, actual exit codes, reports and exact producer/binary
hashes are retained in `registration-order-checkpoint`. This concrete finding
supersedes the earlier historical core/specification review's absence of new
findings; the earlier review is preserved rather than rewritten.

The canonical cases registration validator now checks the term DAGs reachable
from all preceding declaration types, reducible and opaque definition values,
and theorem proofs. Both implementations visit Lam/Pi, App and Let children
and reject nonempty universe arguments on this exact family or its canonical
constructors. The Rust high-level generator validates before registering cases;
its regression checks that failure leaves the declaration environment intact.
Unused arena entries and unrelated families retain their previous behavior.
The prefix scan runs only at canonical cases registration. Existing inference,
reduction, ordinary producer code and Certificate v0 encoding are retained.

Twelve exact producer-generated shared fixtures add seven preceding-use
regressions, a later-use regression, and four positive legacy/unused/unrelated-
family controls. Every preceding-use regression was accepted by both prior
checkers, then rejected by both fixed checkers. All 31 shared fixtures plus
three predecessor certificates pass 68 source-free checker stages with matching
accepted module/certificate/export/declaration counts and zero axioms. Existing
completed reports are reused only after exact input/report/stderr/binary hashes
and every acceptance/rejection condition are checked. Rust passes 56 selected
core/kernel/feasibility tests; Go passes 16 selected top-level tests including
31 shared fixture subcases. Affected Clippy and format checks pass. The actual
producer recompiles and regenerates all twelve pinned byte sets exactly.

The selection covers the changed cases validator/generator, environment and
inference consumers, declaration checking, the W09 feasibility owner and the
source-free checker interface. The whole gate is deferred to final T01-W10 and
T06-W12. The exact public fix source now passes Linux validation, recorded separately
in `registration-order-linux-checkpoint`: 61 Rust tests, 16 Go top-level tests,
31 shared subcases, lint/format and all 68 source-free checker stages.
The initial invalid-name producer and type-mismatch producer runs, and two
report-shape harness failures, retain their real failure statuses and inputs;
none is counted as a successful verification.

A read-only inspection records all 1,920 current ordinary fixture and explicitly
unapplied candidate certificate paths (1,443 distinct byte sets). None contains
the reserved `.cases` name bytes. The new registration-only branch therefore
cannot execute for these exact ordinary bytes. This is a code correspondence
review, not new checker acceptance: older semantic/checker executions remain
attributed to their actual frozen source checkouts and binaries. Their normal
source/value/mutation tests continue unchanged, and pending executions remain
pending. Future promotions must verify both their exact original source pins
and the explicit reviewed source deltas, preserving this provenance.

See the complete checkpoint receipt and file manifest under
`../probes/boolean-proof-elimination-refreeze/registration-order-checkpoint/`.

The Linux source remains clean at public commit `fd99c03c`; 371 selected and
embedded input Git blobs are verified. The initial empty-checkout guard failure
and missing-embedded-input compile failure 101 are preserved. The six already
passed core/kernel checks retain exact source/log provenance; the failed and
unexecuted stages run after the missing public inputs are added. Final evidence
summary/freeze refresh remains deferred until all current original owner and
consumer acceptance runs complete. See the source-correspondence guard review
for the exact original-to-fixed source deltas used by later promotions.


The final closure review also found five changed source pins in the original
capacity and recursor reproducibility records. Their unchanged owners export
all twelve capacity and fifteen recursor certificate byte sets again. The fixed
Rust and Go binaries pass all 108 invocations: 48 capacity accepts and 60
recursor observations, including the ten retained type-mismatch report pairs.
Both complete probe objects remain identical. All recursor observations remain
identical; capacity observations differ only in measured elapsed times, with a
maximum of 3,335 ms below the frozen 60,000 ms bound. Binary, source, input,
report, stderr and actual toolchain observations are independently audited in
`registration-order-capacity-recursor-checkpoint`.

The current probe records and freeze stay pinned while the remaining original
consumer/source runs execute. Their final refresh must preserve six predecessor
files and change only the six reviewed evidence digests. Every complete row of
the 709 private vectors and every non-evidence freeze field must remain exact.
A shared-artifact specification paragraph still states that both checker rules
are unchanged; its prepared correction must reconcile the approved cases rule
at that final closure. Neither this evidence checkpoint nor that correction
completes W09, W10 or any of the 987 application proof IDs.


All fourteen original wrapper source/dependency/import owners now pass in the
isolated review checkout with normal candidate fixture comparisons. The full
source/value/mutation checks, compound definition closures, semantic-root
aliases, retained unguarded metadata and hostile-linkage rejection checks
remain unchanged. The independent review verifies all actual terminal logs,
4,430 original source/input hashes, all 288 candidate files and the current
checkout's exact pre-promotion fixture bytes. Execution remains attributed to
the original frozen source and its reviewed registration-only correspondence.
This source-validation evidence is in `wrapper-original-owner-checkpoint`.
The earlier real boundary-field owner failure and independently replayed parent
identity repair are retained. All 230 checker stages remain required before
promoting these candidate fixtures; this checkpoint does not promote them or
complete W09, W10 or any application proof.
