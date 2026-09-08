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

## Shared ordered folds for subsequent relations

An ordinary ordered-fold program now reconstructs the index widths needed by
all reachable array/sequence carriers, including referenced and uninvoked
instances. A single closed C6 state pipeline carries a u32 index and the first
nonzero i32 word. Each StepTwo is a concrete C6->C6 value under fixed predicate
and length arguments. Balanced guarded composition preserves source index order
and counts all 8,192 leaves before sharing, even when only a smaller capacity is
used. All index widths 0..14 share this same checked definition.

First returns the first nonzero word unchanged. Any and All embed Boolean
predicates in the low word bit and retain false/true respectively for empty
input. Every wrapper clamps count to its power-of-two capacity, and the second
read of an odd final pair is guarded. This clamping is not proof that an
application length is valid: callers must separately enforce their real domain
and role bounds. Validation's shared sequence still uses the 4,096-slot carrier;
its 256-error bound remains a separate domain requirement.

The original direct unsigned-comparison expression duplicated the lower-bit
subexpression, causing exponential syntactic dependency traversal while building
the axiom inventory. The two old verification processes were explicitly stopped
after diagnosing that construction issue, and their logs were preserved. A
linear expression now references the lower result once. A regression counts
expanded syntax without enumerating it, rejects exponential growth before
certificate construction, and checks boundary comparisons. Neither checker nor
its acceptance/performance behavior was changed.

Small order/count/carry cases, Boolean truth tables, source generation/mutations,
syntax-growth regression and lint have passed. Original sources cover bounded
sequences, validation, ordered maps, UTF-16 indexing and a no-fold scalar case.
The full 16,384-element execution, the odd 16,383-element exclusion case,
pinned replay and all six same-byte dual-checker cases passed. The core suite
completed in 324.77 seconds; it was not restarted after the corrected run
began. The inventory also passed without a fingerprint change. See
`unit-3-ordered-fold-verification.json` and `unit-3-ordered-fold-review.md`.
This completes the common fold component only. No structural equality, canonical
order, recursive domain or application proof is claimed by these fold helpers.

## Structural equality and eligible canonical order

The relation generator reconstructs all storable reachable carriers from the
validated original VIR, including uninvoked concrete instances. It lowers scalar
value equality/order through the existing finite integer, IEEE and decimal
circuits. Products compare stored fields, sums compare tags and active payloads,
and length-bearing sequences use the common ordered fold over their shared
prefix before comparing full lengths. Money compares currency before its numeric
decimal amount. Internal construction state receives no storable relation.

Canonical ordering is absent for IEEE values, closed exceptions and every
containing type, including absent options and empty sequences. Representation
padding, valid tags/lengths, map/set canonicality and public source invariants
remain caller obligations; these helpers do not establish their domains or
prove application VCs. Exact metadata and certificate bytes are reconstructed
on import. Neither checker nor its acceptance rules are changed.

The original-source observation suite initially used nonexistent IDs in raw
source member facts; matching the recorded declaration order fixes that sample
construction error. An initial Money observation exhausted the default native
test stack; the suite now uses the existing decimal tests' 64 MiB thread stack.
A live sample confirmed that the longer run was reducing ordinary terms in the
test observer. The initial 21-source suite subsequently passed 408 core value
pairs in 804.30 seconds. The strengthened suite now includes same-length element
differences, later source fields, nullable IEEE NaN/zero cases, signed/scaled
decimal zero, user-exception payloads and ignored empty storage. It passed
979 value-pair/storage observations in 1,859.76 seconds. Two additional Money
cases passed in 223.50 seconds: equal currencies with numerically equivalent
decimal representations, and equal currencies with different amounts. These
close the relation component only; unit 3 and W09 remain incomplete.

Source certificate generation/import mutations and exact pinned replay passed
for all 21 captures and 102 relation-root occurrences across those contexts. A supplementary scalar
certificate passed 260 signed/unsigned boundary pairs over widths 0..128, with
normalized -1/0/1 results. All 22 certificate bytes are pinned in `relations/`;
both checkers accepted all 22 with zero axioms and rejected all 22 hash
corruptions in 355.53 seconds. The source certificate pass separately
validates every test sample against the existing value model before deep core
observation. Its `certificates.json` records zero value-pair observations intentionally;
the extracted index-word checks are separate from those value-pair metrics.
Only the full semantic pass emits value-pair observation metrics.

The extracted test observer matches its committed implementation after formatting
and the visibility change to `eval`. Existing lazy-argument and four structural
storage regression tests passed. Scoped lint passed after replacing three
manual divisibility expressions and two unnecessary Box reallocations in test code. The namespace inventory adds only
`csharp_practical_ordinary_test_eval.rs`: removing it reconstructs the old
106-path fingerprint exactly. The updated 107-path fingerprint, aggregate count
4,930 and linked raw inventory hash passed all five inventory tests; historical
cache entries are unchanged. See `unit-3-relations-verification.json` and
`unit-3-relations-review.md`. The full T06 gate remains deferred to W12.

## Remaining unit 3 work

Inspected next-component requirements, including logical cell-count exceptions
for map entries and transition events, are in `unit-3-domain-notes.md`. These
notes do not implement the missing domains or defaults.

- Complete semantic domains/default eligibility for all concrete values,
  including recursive canonical padding and inactive storage, sum tags, nullable
  payloads, string bounds and role-specific sequence bounds. Primitive scalar
  and source-enum representation domains are implemented as described above;
  source public clauses and default eligibility still require assembly.
- Unit/parse-error/exception operations and semantic constructors/accessors over
  these helpers, retaining domain, validation and ordered failure obligations.
- Bounded sequence and construction-state operations, ownership/initialization,
  ordered maps/sets, outcome/validation operations and error concatenation.
- Expand every operation of every reachable concrete foundation instance,
  including uninvoked operations, and close unit 3's full scoped acceptance.

Bindings, business adapters and codecs/literals remain unit 4; native body and
control relations, transitions/replay, proof assembly and W09 acceptance remain
units 5-8. This component does not move those ownership boundaries.
