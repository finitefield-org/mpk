# W09 internal unit 3: structural and collection foundation progress

Unit 3 is in progress, not complete. Units 1 and 2 are complete as internal
units. W09 is not complete, and W10-W12 remain blocked. The full T06 gate
stays deferred to W12.

## Product/sum storage construction and access

The new ordinary structural generator independently reconstructs the complete
carrier inventory from validated original VIR, then emits storage operations
for every reachable product and sum, including uninvoked concrete instances.
Each source field ID/order and each closed sum arm ID/tag is retained in the
metadata. Imports regenerate metadata and exact certificate bytes; neither
caller-supplied layouts nor accepted certificate hashes replace reconstruction.
A source containing no products/sums legitimately has an empty definition list.

Product MakeStorage takes the stored children in order and projects their
Boolean-cube values into the frozen field/leading-padding layout. It zeros
newly introduced padding and unused field addresses. Projections supply fixed
field bits and zero leading selectors to unpad a child. Empty products are the
single storage-zero Bool value. Validity of each supplied child remains a
precondition; MakeStorage is not a public default or a source constructor proof.

Sum MakeStorage encodes the full u32 tag and the selected arm's product in the
single active payload region. It zeros the new padding around that payload.
Tag reads the full u32 field. Each arm has an IsActive and complementary
InvalidOperation definition. Field getters expose the payload only at the
matching tag and otherwise return storage zero, including for unknown tags.
The payload is shared storage; no separate physical region is reserved for
inactive arms. Role bounds remain in the reconstructed carrier shape.

All emitted types are concrete Boolean cubes and all definitions use ordinary
terms and existing Bool eliminators. There is no new template, generic type,
recursor, checker rule, axiom or production route. No fold is needed for these
fixed product/sum addresses. Actual source and synthetic boundary certificates
are retained in `structural-storage/` for same-byte dual checking.

## Verification and review

The storage component passed targeted verification and direct review; this
does not complete unit 3. Exact commands and log hashes are recorded in
`unit-3-storage-verification.json`; see `unit-3-storage-review.md`. The core tests cover
field order, non-power-of-two field/child padding, empty products, sparse and
high-bit tags, active/inactive/unknown-arm field access, padding beyond 32 bits,
the inclusive binder depth 256 and rejection at 257, missing references and
duplicate tags. Original source replay covers the existing 24 binding,
exception, transition and construction captures. Those remain storage-helper
positives even when an application invariant/transition in the capture is
intentionally invalid: no application proof is claimed.

The initial source mutation test assumed a nonempty structural definition list;
it was corrected to test forged entries for scalar-only sources. The initial
lint finding was a test-only unnecessary Vec, changed to an array. The new
standard-namespace consumer is the structural source module: removing that path
reproduces the previous 105-path fingerprint; the updated count is 106. The
linked historical inventory raw hash changes with it; cache paths are unchanged.
The inventory test also retained the old aggregate count: correcting 4,928 to
4,929 closed its final finding, and all five inventory tests then passed.

## Primitive scalar and source-enum representation domains

The scalar-domain generator reconstructs all reachable Bits carriers and the
closed decimal product from validated VIR. It retains each source enum's exact
canonical declared values. Fixed-width integer/IEEE/Guid/duration/instant
carriers admit every bit pattern, including IEEE NaNs and signed zero. Unit
admits only storage false. Date, time, DayOfWeek and ParseError enforce their
frozen inclusive bounds. Decimal checks scale 0..28 and all unused product and
coefficient addresses; sign and scale are not normalized away. Negative zero
and trailing-zero representations remain valid, as in the existing value model.

Each predicate has a concrete carrier-to-Bool type and uses only finite
ordinary Boolean expressions. There is no function equality, extensionality
axiom, generic recursor, trusted host observation or application proof.
Imports independently regenerate exact metadata, domain rules and bytes.
String/exception/collection/product domains, source public clauses and default
eligibility remain explicit later work; this component does not claim them.

Core tests passed for all 21 primitive scalar kinds, all 256 underlying i8
enum values, high-bit u64 enums, all 256 decimal scales at zero/max coefficient
and both signs, and every unused decimal address. Original-source generation
and metadata/rule/context/hash mutation tests passed over 24 captures. Review
added two original time/date/weekday captures to cover range-rule selection.
All 27 certificates (26 sources plus one core corpus) pass both checkers with
zero axioms; corruptions reject. Pinned replay, lint and all five inventory
tests passed. See `unit-3-scalar-domain-verification.json` and
`unit-3-scalar-domain-review.md`. This closes the scalar representation-domain
component only; unit 3 and W09 remain incomplete.

## Remaining unit 3 work

- Complete semantic domains/default eligibility for all concrete values,
  including recursive canonical padding and inactive storage, sum tags, nullable
  payloads, string bounds and role-specific sequence bounds. Primitive scalar
  and source-enum representation domains are implemented as described above;
  source public clauses and default eligibility still require assembly.
- Field/active-payload/element semantic equality and canonical ordering,
  including non-reflexive scalar cases and complete source-snapshot observation.
- Unit/parse-error/exception operations and semantic constructors/accessors over
  these helpers, retaining domain, validation and ordered failure obligations.
- Bounded sequence and construction-state operations, ownership/initialization,
  ordered maps/sets, outcome/validation operations and error concatenation.
- Expand every operation of every reachable concrete foundation instance,
  including uninvoked operations, and close unit 3's full scoped acceptance.

Bindings, business adapters and codecs/literals remain unit 4; native body and
control relations, transitions/replay, proof assembly and W09 acceptance remain
units 5-8. This component does not move those ownership boundaries.
