# W09 internal unit 2: scalar progress

Unit 2 is **in progress**, not complete. This increment implements the Boolean
and fixed-width integer portion of the approved scalar work unit, followed by
the Time/Duration/Instant, Date/Guid/DayOfWeek, floating-operation and numeric
conversion and complete non-literal decimal operation components below. UTF-16
string operations still belong to unit 2.
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

## Binary32 and binary64 floating-operation component

The floating generator covers the 38 registered binary32/binary64 operations:
sign, absolute value, classification, equality/order, min/max, addition,
subtraction, multiplication, division and remainder. Source signatures and
ordered-check tables are independently reconstructed through the closed numeric
registry. Import reconstructs the original-source/foundation metadata and exact
ordinary certificate bytes. Numeric conversions are implemented in the following component.

The operations preserve the frozen T03 rules for signaling-before-quiet NaN
propagation in arithmetic, unchanged unary/min/max payloads, positive invalid
NaNs, signed zero and nearest-even rounding. Addition aligns bounded
significands with sticky bits; multiplication uses the full significand product;
division retains a quotient and sticky remainder. Packing normalizes, shifts
into the subnormal range, rounds once using guard/round/sticky bits and handles
carry and overflow. No host floating-point result enters an ordinary definition.

Remainder uses three ordinary helper definitions: initialize normalized
significands and the exponent difference, apply one modular-doubling state step,
and finish by packing the remainder and applying the special-value rules. Each
helper's internal metadata describes its concrete Boolean-cube types. The
public operation retains the original scalar signature. Static composition
reuses the same step term 276 times for binary32 and 2,097 times for binary64.
For finite nonzero operands, these are the maximum normalized exponent gaps.
Every occurrence counts toward the transformer bound before DAG sharing; a zero
counter makes subsequent steps identities. This is finite ordinary composition
with no recursive theorem, observation axiom or new checker rule.

The 38 individual certificates and measured costs are reproducible. Binary64
division is the largest at 158,773 terms and 1,665 declarations. Binary64
remainder uses 35,415 terms and 2,360 total transformer occurrences, including
its helpers and 2,097 repeated steps. Eight representative certificates are
pinned as exact bytes for same-byte dual-checker validation.

Verification includes the T03 numeric oracle on a 30-value boundary/sample
matrix per format, plus 1,024 deterministic random or cancellation-focused pairs
per arithmetic operation and format (44,440 oracle evaluations overall).
Cases include signaling and quiet NaNs, both zeros, subnormal/normal boundaries,
overflow, halfway rounding, near cancellation and extreme exponent gaps.
The staged remainder observer checks the zero-counter identity as well as the
final value. Actual-core evaluation covers nine cases, including both signs of
binary32 remainder through the full 276-step composition, nearest-even ties,
negative zero, NaN payload priority in min and NaN classification.

Eight original source captures exercise both addition widths, binary64
multiply/divide/min, both remainder widths and binary32 NaN classification.
Operation/type/check/definition/source/foundation/certificate mutations reject.
The additional direct Bool consumer is recorded in the consumer inventory;
its historical-cache link and total are updated consistently.

See `unit-2-floating-verification.json` and `unit-2-floating-review.md`.
This floating-operation component does not complete unit 2, foundation-wide
expansion, application VC proofs or W09. The full T06 gate remains deferred
to T06-W12.

```sh
MPK_W09_FLOAT_OUT=/tmp/mpk-w09-float-fixtures cargo test -p mpk-vc --lib floating_circuits_
```

## Six numeric conversions

The floating generator also reconstructs all six frozen `numeric.conversion.*`
signatures: int32-to-single, int64-to-double, single-to-double, double-to-single,
checked single-to-int32 and checked double-to-int64. Decimal conversions remain
in the decimal component. The exact argument/result types and ordered overflow
check come from the independent numeric registry. The same importer regenerates
metadata and canonical bytes for source, foundation and certificate linkage.

Signed integer inputs are converted to unsigned magnitudes before the existing
ordinary packer applies nearest-even rounding. Integer minimum magnitudes are
retained in the full unsigned source width. Float width changes use the source
significand and exponent with the target packer; special values preserve sign,
shift the payload and quiet every NaN even when narrowing discards all payload
bits. Signed zero and subnormal inputs retain the frozen T03 semantics.

Checked integer conversion uses saturating logical shifts with every exponent
bit, followed by a sign-dependent range check on the truncated magnitude. An
independent highest-set-bit bound rejects large exponents even if a left shift
has discarded every one. Special values always overflow. A negative exact
minimum is admitted; positive values at that same magnitude overflow. The
ordinary emitter gives failed conversions one overflow predicate, false success
and a canonical zero normal result. No host conversion result defines a term.

Verification covers 63,396 oracle evaluations: every source float exponent with
representative mantissas and signs, neighbours of integer limits and float32
rounding/underflow/overflow thresholds, signed integer power-of-two boundaries,
and deterministic random values. Twenty actual-core observations exercise all
six conversions, nearest-even ties, integer minima, subnormal rounding, signed
zero, NaN payload quieting, successful truncation and overflow with a zero result.
Six original source captures exercise all six signatures and reject linkage and
ordered-check mutations. Six certificates and their costs are pinned in
`conversion-circuits/`; the largest has 17,627 terms and 151 declarations.
Existing floating-operation certificate bytes and all 38 cost records are
reproduced unchanged.

See `unit-2-conversion-verification.json` and `unit-2-conversion-review.md`.
Unit 2 remains in progress for decimal operations/conversions and UTF-16 strings.
Input-domain predicates, all-instance expansion, application VC proofs and the
remaining W09 units are still outstanding. The full gate stays at T06-W12.

## Decimal conversion and rounding component

The decimal generator implements 33 signatures: plus/negate, truncate/floor/
ceiling, all five rounding modes at both frozen arities, and both directions
between decimal and the nine integer/char carriers. Decimal arithmetic/comparisons are added in the following component; literal
bodies remain separate work. At this increment, unsupported decimal signatures
failed closed rather than producing a partial decimal program.

The public carrier remains unit 1's depth-nine Boolean cube. Field selectors
precede child selectors. Sign is at address 0, scale bit i at `1 + 64*i`, and
coefficient bit i at `2 + 4*i`; every unused field/child address is zero on output.
Integer-to-decimal conversion preserves the unsigned magnitude of signed minima
and emits scale zero. Unary operations preserve decimal scale and signed zero.

Rounding and decimal-to-integer conversion use three ordinary helpers. The
initializer derives the target scale and a finite digit count. A single helper
step divides the 96-bit coefficient by ten, records the last discarded digit,
retains a sticky bit for earlier nonzero digits, and decrements the count. The
root statically composes 28 steps. After count zero, a step is the identity.
For valid decimal scales 0-28, the composition consumes every requested digit;
invalid input-scale domains still belong to the later domain work.

The finalizer rounds once using the retained digit and sticky bit, with the
original sign and requested mode. It preserves scale when no reduction is
needed. Round's two-argument variants reject negative or greater-than-28 digits
through the original ordered range predicate. Decimal-to-integer conversion
truncates first, checks signed/unsigned bounds, accepts a negative fractional
value that truncates to zero for unsigned targets, and rejects a remaining
negative unsigned value. Failures have a false Success, one ordered failure
predicate and an all-zero normal result. The internal state helpers use concrete
cube signatures; public metadata retains the original decimal signatures.

Verification covers 68,400 T03 oracle cases over every scale, coefficient/range
boundaries, both signs, all rounding modes/arities and invalid requested digits.
A bit-sliced test observer evaluates the same Boolean gates for 64 independent
cases at a time. It compares every physical result address, including padding,
and verifies the zero-counter identity. Eight actual-core observations cover
signed zero, char construction, nearest-even rounding without double rounding,
range failure, unsigned zero after truncation and overflow. Fourteen original
source captures cover all operation families and rounding modes; an additional
source combining conversion with then-unimplemented equality was rejected. The
following arithmetic component now covers that original capture positively.

All 33 individual certificates fit unchanged limits. Seven representative
certificates are pinned in `decimal-circuits/`; the largest definition has 14,491
terms, 139 declarations and 130 static transformer occurrences including its
28 steps. Source/foundation/signature/check/definition/certificate substitutions
are rejected by regeneration. Consumer inventory baselines remain unchanged.
See `unit-2-decimal-verification.json` and `unit-2-decimal-review.md`.

This component leaves decimal arithmetic/comparisons, UTF-16 strings, input
domains, all-instance expansion, application proofs and W09 units 3-8 unfinished.
The full T06 gate stays deferred to T06-W12.

## Decimal arithmetic, comparisons and shared assembly

The remaining twelve non-literal decimal signatures are implemented: equality,
value equality, inequality, four ordering relations, add/subtract/multiply/divide/
remainder. Together with the previous component, the generator covers all 45
non-literal decimal signatures. Literal bodies and input-domain predicates remain
separate work.

Alignment expands 96-bit coefficients into 192-bit words and applies at most 28
conditional multiplications by ten, preserving the original signs and choosing
the maximum source scale. Comparisons use the aligned magnitudes; differently
scaled and signed zeros compare equal. Addition/subtraction preserve the frozen
left-sign rule for exact cancellation. Multiplication uses 96 finite shift/add
steps and retains the full 192-bit product and summed scale.

Division and remainder use 288 restoring steps with a 193-bit remainder. Division
first scales the aligned numerator by ten 28 times in a 288-bit word, retaining
its quotient and exact nonzero/halfway residue information. Remainder uses the
aligned dividend and retains the dividend sign. No host arithmetic result defines
an emitted result.

A shared fit step chooses the greatest admissible scale, with at most 57 steps
for the maximum input product scale of 56. It retains the last discarded digit
and sticky residue and rounds once using nearest-even. For division, the initial
rounding decision uses the full binary remainder/divisor comparison; subsequent
decimal reductions preserve that fractional residue in the sticky bit. Completed
fit states are identities. Divide-by-zero precedes overflow, and failed normal
results are zero. Valid remainder operands cannot require overflow: after scale
alignment, either the dividend or divisor remains a 96-bit coefficient.

Internal helper sharing uses an exact serialized key containing every gate,
input width, physical output/failure mapping, type and ordered check. It does not
use source names or mathematical observations as an equivalence test. Shared
helpers remain ordinary definitions. Every explicit repeated step in each public
pipeline is counted; sharing removes duplicate helper declarations. Individual
certificates and metrics for the previous 33 definitions remain byte-identical.

A single certificate containing all 45 public operation definitions fits the
unchanged bounds: 173,588 terms, 2,270 declarations and 3,858 static transformer
occurrences, with 45 distinct internal helper circuits. The twelve new individual
certificates also fit; division is largest at 76,879 terms and 646 declarations.
Seven representative individual certificates and the combined certificate are
pinned in `decimal-arithmetic-circuits/`. This is foundation-definition assembly,
not an application-VC completion receipt or whole-foundation publication.

Verification includes 30,108 independent T03 oracle cases, all scale-pair
combinations with boundary coefficients, signed/scaled zero, representable and
unrepresentable results, division by zero and deterministic random pairs. Bit-
sliced observations check the terminal identities of alignment, multiplication
and fitting and full physical results. The shared-certificate observer test contains
16 actual-core cases, including prior rounding/conversion definitions that must
remain distinct under sharing. Twelve original captures cover eleven directly
emitted arithmetic/comparison signatures plus a mixed conversion/equality case;
value equality is also covered as a foundation signature. The preceding fourteen
original decimal source cases remain covered.

The test-only core observer uses an explicit continuation stack and shared Boolean
arrays. Captured environments use shared binding lists. Each closure owns a
compact two-slot Boolean memo; suspended arguments remain lazy, including unused
Boolean arguments. A poison-suspension regression checks that unused arguments
and unselected Bool branches are never demanded. Deep term evaluation no longer consumes the native recursive stack.
An earlier observer stack overflow and a memory-heavy observation attempt are
recorded as observer limitations, not checker rejections. No checker or Certificate
v0 rule was changed. Existing core observers are included in the scoped regression
checks; the final observer regression passed all eight test functions in 3628.93 seconds,
and all eight same-byte dual-checker fixtures passed with zero axioms.

See `unit-2-decimal-arithmetic-verification.json` and
`unit-2-decimal-arithmetic-review.md`. UTF-16 strings, input domains, typed literal
bodies, all reachable foundation instances, application proofs and W09 units 3-8
remain outstanding. The full T06 gate remains deferred to T06-W12.

## Basic UTF-16 operations

`string.length`, `string.index` and `string.is_null_or_empty` now have ordinary
core definitions for the full 16,384-unit string carrier and its nullable option.
Indexing uses the actual cube with dynamic index selectors; it preserves UTF-16
code units, validates the signed index before exposing a character, orders null
before range failure, and zeroes the failed result. The public importer rebuilds
source/context/foundation/signature/certificate links exactly.

The independent core observer passed 77 cases across nullable and non-null
carriers, including maximum length and isolated surrogates. Nine original source
captures passed regeneration and linkage-mutation checks. Their actual nullable
source certificates are pinned in `string-basic-circuits/`; the largest has 1,852
terms and 44 declarations. Final fixture replay, dual checking, inventory, lint
and format passed. The original mixed Length/concatenation case becomes positive in the next
construction increment; an ordinal-search source remains fail-closed. The consumer inventory adds exactly the new string
implementation and retains its add/remove rejection tests. Direct review has
zero findings; decimal arithmetic verification has also passed. See `unit-2-string-basic-review.md` and
`unit-2-string-basic-verification.json` for the reviewed address map
and remaining work. This component does not complete scalar strings or W09.

## UTF-16 substring, concatenation and interpolation

Substring(start, length), two/three/four-string concatenation, the three
string/char operator combinations and admitted restricted interpolation shapes
now generate ordinary result, success and ordered failure definitions. Null
strings contribute zero length to concatenation; character arguments preserve
all UTF-16 code units. Empty interpolation returns an empty string.

The full depth-19 output cube selects a source character dynamically from all
fourteen index bits. Substring adds the requested start; concatenation selects
the first segment whose prefix end exceeds the index and subtracts that segment's
start. Length-header padding and inactive content are zero. Range checks avoid
signed addition overflow, and output length is bounded by 16,384. Failure results
are zero, with null receiver before substring range failure. Domain predicates
and literal bodies remain subsequent work.

The composed binder depth is N + 25 for N construction arguments. The corrected
limit test uses the registered cap 256, accepts 65 and 231 arguments and rejects
232 and 257. A pinned 231-argument certificate is accepted by both unchanged
checkers; it has 8,211 terms and 100 declarations. This preserves the ordinary
limits rather than imposing an unrelated lower argument cap.

Forty-four oracle comparisons observe actual core results in both physical
carrier modes. They cover maximum length, overflow, index extremes, nullable
priority, binary index carries, surrogates, NUL and interpolation shapes. All
length bits and selected content/padding addresses are observed; large outputs
are sampled as specified in the review, not claimed exhaustively enumerated.
Seven original source captures regenerate pinned certificates and reject linkage
mutations. The previous nine basic captures retain their bytes. One original
ordinal-search source still fails closed. All eight construction certificates,
including the binder boundary, passed same-byte dual checking with zero axioms
and hash-mutation rejection. Source replay, inventory, lint and format passed.

The consumer inventory adds exactly the construction implementation, increasing
its standard-namespace path count from 103 to 104. Direct review corrected the
binder assumption and basic-string receipt log hashes; the final review has zero
findings. See `unit-2-string-construction-review.md` and
`unit-2-string-construction-verification.json`. Ordinal comparison/search and
W09 units 3-8 remain outstanding. The full gate remains at T06-W12.
