# Direct review: W09 unit 3 scalar representation domains

Scope: primitive scalar and declared source-enum representation predicates,
independent original VIR reconstruction/import, scalar boundary observations,
26 original captures and 27 same-byte dual-checker certificates. No subagent
or review skill was delegated. Unit 3 and W09 remain incomplete.

## Finding addressed

The first 24 original captures covered Bits, decimal and declared-enum rules
but no Range rule. Synthetic core evaluation alone did not verify that actual
source lowering selects date/time/weekday bounds. Added the retained TimeOnly
addition and DateOnly weekday sources from data-stage-replay.json, asserted the
exact three bounds, rejected a substituted range, reproduced pinned bytes and
checked both additional certificates with both checkers. ParseError rank is
covered by the core corpus; a source-codec proof remains later W09 work.

## Final review

No remaining actionable findings in this component. The selected rules agree
with the existing monomorphic value validation and frozen non-template tags:
Unit is false; closed enums use exactly declared underlying carriers; date is
0..3,652,058; time is 0..863,999,999,999; weekday is 0..6; parse error is 0..4.
All integer/IEEE/Guid/duration/instant bit patterns are representation-valid.
NaN and signed-zero semantics belong to operation equality, not representation
rejection. Unsigned bitwise range comparison rejects negative raw date/time
encodings. Canonical signed enum strings are range-checked by validated source
import before their low two's-complement bits are emitted.

Decimal retains the frozen field/padding order, requires scale <= 28, uses
exactly 96 coefficient bits, and rejects every nonzero unused address. Both
signs of zero and trailing-zero scales remain valid; representation normalization
is not silently imposed. Core observations check all decimal scales with both
signs at zero/max coefficient, each unused address independently, all underlying
i8 enum values, high-bit u64 enum values and primitive boundaries. The scalar
corpus contains 24 definitions (21 primitives, two enums, one padding case).

All predicates have closed carrier-to-Bool types. Their finite expressions use
ordinary Boolean eliminators with Boolean results. No wide integer becomes
unary Nat, no function equality or extensionality axiom is used, and no core,
Certificate v0, checker rule or public route changes. Actual builder limits and
16 MiB import/output bounds apply. Metadata and bytes are regenerated from
validated VIR, including uninvoked reachable scalar types; substituted rules,
source/context/foundation/hash values and certificate bytes reject.

These predicates establish representation membership only. Recursive domains,
source public invariants, default eligibility, collection operations and proofs
remain recorded in unit-3-progress.md and the eight-unit implementation plan.
The existing structural storage generator and its bytes are unaffected; the
module adds only scalar-domain exports. The consumer inventory still passes
without a fingerprint update because no new searched namespace/path match was
introduced. Exact verification commands/results are in the linked receipt.
The repository-wide gate remains deferred to T06-W12.
