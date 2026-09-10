# Calendar/temporal contract connections (partial W09)

Eight captured C# contexts connect all 50 native unary/binary DateOnly, TimeOnly,
TimeSpan and Guid operations to their existing ordinary definitions. The 54
contract attachments include four failed definedness cases. Fourteen selected
ordinary evaluations passed, including Gregorian month/year movement, range
failure, TimeOnly construction and wrapped subtraction, negative duration
components, duration overflow and Guid ordering.

Every operation alias and complete dependency closure matches its standalone
definition. Both runtime and candidate runs generated identical certificate
and metadata files. `certificates.json` records the eight pins; the source
transport and pinned input hashes are in `capture-receipt.json`. All eight
same-byte Rust/Go checker cases passed, including zero-axiom/hash agreement and
corruption rejection. Current-code regeneration preserved every pin; see
`../unit-4-calendar-clauses-progress.json` for the reconciled results.

The production adapter also preserves all 70 existing calendar/temporal
standalone pins. Its 65 unary/binary signatures include internal DayOfWeek and
non-error Instant entries without new native-source coverage in this corpus.
The three Instant error-outcome operations require an outcome-aware adapter
and still reject, rather than exposing a failed operation's zeroed normal
result. Date's three-argument constructor and Guid's zero-argument Empty are
outside this unary/binary connection.

These are ordinary helper/contract certificates, not application VC proofs.
Native constructor/getter bodies and all other outstanding W09 units retain
their original completion requirements. The full repository gate stays deferred
to T06-W12; no original unit is completed by this component.
