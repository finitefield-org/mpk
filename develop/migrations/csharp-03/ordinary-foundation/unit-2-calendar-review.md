# W09 unit 2 calendar component: direct review

Scope: Date/Guid/DayOfWeek ordinary circuits, shared business-signature and
constant-division helpers, original-source import/export, fixtures and records.
This is a component review, not completion of unit 2 or W09.

The direct review checked the 400/100/4/1-year decomposition, terminal quotient
caps, leap-year and month-start arithmetic, explicit offset bounds, widened
33-bit range checks before truncation and target-month day clamping. It checked
that invalid construction never yields a successful normal result. It also
checked Guid N-field order against unsigned 128-bit comparison, the nullary
Guid.Empty path and the three-argument Date constructor in actual emitted core.
All host calendar arithmetic remains in independent tests; production uses only
the finite Boolean gates lowered to ordinary terms.

The importer regenerates the exact operation/type/check table, metadata and
certificate bytes. Source/foundation/certificate hashes, definition lists,
result names and operation tables cannot be substituted. Source tests retain the
original enum representation for weekday comparisons and separately cover
Date's weekday projection; the fixture assertion was corrected rather than
rewriting the existing source-operation contract. Structural enum lowering
remains in unit 3.

The final diff keeps both checker implementations and Certificate v0 unchanged.
It preserves all existing integer and temporal fixture bytes and keeps W09
Ready/in progress and W10-W12 Blocked. Input-domain predicates, all-instance
expansion and application proofs remain outstanding.

Direct review findings for this component: 0. The targeted verification results
and exact fixture hashes are recorded in `unit-2-calendar-verification.json`.
The full T06 gate remains deferred to T06-W12.
