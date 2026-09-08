# W09 internal unit 2: scalar progress

Unit 2 is **in progress**, not complete. This increment implements the Boolean
and fixed-width integer portion of the approved scalar work unit, followed by
the Time/Duration/Instant and Date/Guid/DayOfWeek components below. Float,
decimal, string operations and the remaining numeric conversions still belong
to unit 2.
Units 3-8 and W09's exit condition are unchanged. This record is not a W09 completion receipt.

## Implemented component

The generator derives exactly the Boolean/integer signatures retained in the
original validated VIR. It uses the existing closed scalar-signature registry
for operand/result types and ordered checks. It emits ordinary definitions for
the normal result, success predicate and mutually exclusive ordered failure
predicates. The normal result is zero on failure and must be consumed with its
success predicate. These definitions are not proofs of source invariants.

Covered operations are Boolean not/and/or/xor/equality, promoted i32/u32/i64/u64
arithmetic, division/remainder, comparisons, bitwise operations, the three shift
forms and checked/unchecked conversions among the nine integer/char carriers.
Arithmetic uses little-endian two's-complement words, widened mathematical
intermediates for overflow, restoring unsigned division with signed correction,
truncation toward zero and the low five/six shift-count bits. Division by zero
precedes signed minimum divided/reduced by minus one overflow, matching the
frozen signature registry. The C# specification also describes this division/
remainder edge in [integer remainder](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/expressions).

Signed multiplication uses an unsigned n-by-n product with the two sign
corrections modulo 2^(2n), rather than multiplying sign-extended 2n-bit operands.
Overflow compares the full product to the sign/zero extension of its low n bits.
No host arithmetic result is inserted as a definition or a trusted proof.

## Ordinary-core lowering

A private acyclic Boolean circuit records only inputs, constants, not, and, xor
and mux. Sharing is deterministic and dead gates are removed from the result
and failure roots. Gate evaluation in the tests is an independent observation
path; production emits actual Certificate v0 terms.

Each concrete state transformer binds at most 32 Bool gates with ordinary Let
and updates one aligned register block of a Boolean cube. Address selection and
updates use Bool-result recursors exclusively. Balanced static composition
preserves gate order. This bounds binder depth independently of arithmetic DAG
length; actual term/declaration/binder/transformer counters are still enforced.
The generator cannot replace a limit failure with an axiom, runtime observation
or partial accepted certificate.

The component retains the original VIR/foundation identities. Its importer
reconstructs the definitions and certificate bytes exactly; missing/reordered/
changed definitions and source/foundation/certificate substitutions reject.
It is a partial scalar capability with an explicit integer-only API name, not
a generic whole-foundation or application proof capability.

## Verification and review

Tests compare widened i128/u128 arithmetic with the bit circuits at signed and
unsigned endpoints, zero, minus one and deterministic samples. They cover all
162 conversion forms, both modes of promoted integer operations, unary edges,
and ordered failures. Separate tests evaluate emitted core definitions against
circuit observations, including updates across register blocks and canonical
zero results on failure. Original captured constructor programs exercise exact
VIR/operation-table reconstruction and substitution rejection. Unit 1's 24
carrier goldens are retained byte for byte.

`integer-circuits/*.hex` and `metrics.json` pin seven actual certificates,
including signed 64-bit multiplication and unsigned 64-bit division. Normal tests
regenerate and compare them. Both unchanged checkers must accept the exact same
bytes with zero axioms and reject hash corruption. Build/process/internal errors
cannot count as proof rejection. These tests verify definition typing and
translation; they do not discharge the later application VCs.

Direct review addressed core selector scope, Let de Bruijn indices, signed
multiplication correction, pruning/reindexing, shift masks, overflow/error
ordering and exact original-source linkage. The additional Std.Bool consumer
is reflected in the pinned consumer inventory and its historical-cache link.
The remaining scalar categories are deliberately recorded as unfinished.

Targeted commands/results are in `unit-2-integer-verification.json`.
The full T06 gate remains deferred to T06-W12; no workflow is introduced.
Candidate fixtures can be regenerated for review with:

```sh
MPK_W09_INTEGER_OUT=/tmp/mpk-w09-integers cargo test -p mpk-vc --lib integer_circuits_
```

## Time, Duration and Instant component

The temporal generator reconstructs exactly the retained Time/Duration/Instant
signatures, including their ordered exception/error tables, from the validated
VIR. It shares the ordinary Boolean-circuit emitter with the integer generator;
the integer API/schema/names and seven pinned certificates remain unchanged.
The temporal importer regenerates both canonical metadata and certificate bytes
and rejects operation/type/error/definition/source/foundation/certificate changes.

All 43 operations in these three scalar families are implemented: construction
where defined, component/tick/millisecond access, comparison and equality,
Time addition/subtraction modulo one day, checked Duration arithmetic and
Instant duration arithmetic/difference. Constant-divisor restoring circuits use
bounded remainder registers. Duration division truncates toward zero, including
negative component remainders. Time arithmetic uses a 65-bit intermediate and
Euclidean day remainder. Instant differences preserve 79 signed bits through
multiplication by 10,000 before checking the i64 range. Precision failure precedes
range failure, and the normal result is canonical zero on either failure.

Input domain membership, foundation-wide expansion of uninvoked definitions and
application invariant proofs remain later W09 work. These operation definitions
consume valid domain values; they do not themselves certify an input domain or
complete a VC. Date/Guid/DayOfWeek are covered by the subsequent component
below; the other unfinished scalar families remain outstanding.

Verification covers widened arithmetic at 31 deterministic boundary/sample
values (961 operand pairs per operation), the independent T03 BusinessOperation
oracle at eight pairs per operation, emitted-core evaluation and explicit
precision-before-range checks. All 43 individual certificates regenerate within
the practical limits; their term/declaration/transformer measurements are pinned
in `temporal-circuits/metrics.json`. Four representative certificates are pinned
as bytes and must pass both unchanged checkers with zero axioms, with actual
hash corruption rejected by each. Four original source/sidecar captures exercise
Duration addition, Time wrapping and two Instant error-enum layouts.

See `unit-2-temporal-verification.json` for the targeted results. The full T06
gate stays deferred to T06-W12. W09 is still in progress, and W10-W12 stay Blocked.

```sh
MPK_W09_TEMPORAL_OUT=/tmp/mpk-w09-temporal-fixtures cargo test -p mpk-vc --lib temporal_circuits_
```

The test-only core interpreter memoizes Bool applications within each captured
closure; caches never cross environments. The two temporal core-evaluation tests
use a bounded 32 MiB test-thread stack for their recursive interpreter. The expanded
uncached run was stopped for runtime, and the initial memoized run exposed the
default test stack limit. These test execution changes do not alter emitted
bytes, the practical certificate bounds or either production checker.

## Date, Guid and DayOfWeek component

The calendar generator adds all 27 operations in these three scalar families.
It reconstructs the retained signatures and ordered checks from the exact
validated VIR and rejects canonical metadata/certificate substitutions on import.
The common business-signature and constant-division helpers are shared with the
temporal component; all 43 temporal definition metrics and its four pinned
certificates, plus the seven integer certificates, remain unchanged.

Date operations cover construction, year/month/day/day-number/day-of-week
observation, equality/order and addition of days, months and years. Gregorian
400/100/4/1-year decomposition caps the terminal 100-year and single-year
quotients at three. Leap tests, month starts and month lengths are ordinary
circuits. Month/year arithmetic first checks the explicit offset and the full
33-bit resulting year/month range, then clamps the day to the target month's
last day. Failed construction/arithmetic has one range exception predicate and
a canonical zero normal result. Years 1-9999 remain the admitted date domain.

Guid uses the canonical N hex number in the 128-bit carrier's little-endian
bits. Its unsigned comparison therefore follows the canonical unsigned field
order, including high bits and every field boundary. Guid.Empty is a nullary
ordinary definition. DayOfWeek operations compare the valid 0-6 i32 carrier;
Date day-of-week uses `(day_number + 1) mod 7`.

The test network is evaluated against T03 BusinessOperation for calendar,
range, Guid and weekday cases. A bit-sliced test evaluates the same Boolean
gates for every one of the 146,097 days in a full Gregorian cycle and compares
the decoded dates to an independent month/day loop. Other boundary cases cover
later cycles through year 9999. Core-evaluation tests exercise actual definitions,
including Guid's nullary result and unsigned high bit, valid/invalid leap-day
construction, and addition overflow with zero on failure. All 27 individual
certificates and their measured costs are reproducible; five representative
certificates are checked as identical bytes by both unchanged checkers.

Seven exact original source captures cover date construction/month/year
arithmetic, Guid empty/equality/comparison and Date's weekday projection. Source
weekday comparisons are retained as enum comparisons in the existing VIR;
their structural lowering remains in unit 3 rather than being reclassified as
direct `day_of_week.*` signatures by this component.

See `unit-2-calendar-verification.json` and `unit-2-calendar-review.md`.
The component does not establish input domains, all-instance expansion,
application VC proofs, unit 2 completion or W09 completion. The full T06 gate
remains deferred to T06-W12.

```sh
MPK_W09_CALENDAR_OUT=/tmp/mpk-w09-calendar-fixtures cargo test -p mpk-vc --lib calendar_circuits_
```
