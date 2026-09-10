# W09 internal unit 1: ordinary carriers

Current unit4 finding: typed JSON used an incorrect cumulative cell bound and
string count. The required bound is65,536 and a string contributes1+UTF-16
units. Affected component reviews are reopened; historical checker/test results
do not close this semantic finding. See `unit-4-json-cell-count-review.md` and
`unit-4-json-cell-count-progress.json` for correction and replacement evidence.

This directory records the first of eight user-approved internal W09 units.
It contains actual ordinary carrier/helper definitions and regression fixtures.
Internal unit 2 (scalar operations/checks) is also complete; see
`unit-2-verification.json` and `unit-2-review.md`. W09 is still in progress:
structural/collection foundations, domains, bindings/codecs, control/transition
relations and application proof assembly remain units 3-8 in
`implementation-plan.md`. Checking these carrier certificates does not
discharge an application VC.

The contract-carrier coverage correction is recorded in
`contract-carrier-extension/changes.json`. Eleven helper certificates now include
the previously omitted contract-only Bool carrier. Their previous bytes are
retained and both versions pass the unchanged checkers; 144 other regenerated
certificates are identical. Older checkpoint counts describe their original
scope. Current defaults and finite unit/parse-error/exception operations are
documented in `defaults/README.md` and `finite-operations/README.md`.
Bounded-sequence operations and construction storage bodies/predicates are now
recorded in `sequence-operations/README.md` and
`construction-operations/README.md`; their checkpoint separates completed
source and same-byte checker checks from pending deep semantics and application proofs.
Option/result/lookup/validation/boundary-field operations and validation-error
concatenation are recorded in `outcome-operations/README.md` and the scoped
outcome verification/review records. Source binding and application proof
obligations remain separate.

Ordered-entry operations and in-progress map/set operations are described in
`entry-operations/README.md`, `collection-operations/README.md` and
`unit-3-entry-collection-progress.json`. The checkpoint distinguishes pinned
certificate checks from versioned semantic tests and superseded collection
generator versions. The full original collection run subsequently passed all
669 observations across nine sources/ten instances. It does not close unit 3 or W09.

The shared ordinary program is described in `structural-foundations/README.md`.
It combines unit-3 generators within the frozen transformer budget and retains
deferred application obligations. Its 43 actual-source candidates now include
complete source observation. Generation/import/pinned replay and 116 transitive
source-component comparisons pass, including metadata and body mutations. The
current 43-pin checker rerun passed with zero axioms and hash-corruption rejection. The previous 42 checked pins and their
receipt are preserved in `structural-foundations/pre-observations/`. See
`source-observations/README.md` and `unit-3-observation-progress.json` for the
23 passing observation pairs, ten IEEE equality contrasts and current scope.

Unit 3 has completed its product/sum storage and primitive scalar/source-enum
representation-domain components, plus shared ordered iteration helpers; see
`unit-3-storage-verification.json`, `unit-3-scalar-domain-verification.json`,
`unit-3-ordered-fold-verification.json` and `unit-3-progress.md` for verification
and the remaining semantic/collection work.

## Representation and trust boundary

`generate_csharp_practical_ordinary_carriers` consumes only a validated original
VIR. It reconstructs the reachable type inventory, including verified contract
terms, definition signatures and subject bindings, and every expanded instance,
including instances whose operations are not invoked. Source products retain
stored-member order and IDs; enums use their underlying width; exceptions use
the derived closed tag/payload universe. Recursive references, unknown types,
residual template shapes and unresolved constants fail before a result is returned.

A cube C(d) is `Bool -> ... -> Bool`, with d address arguments and a Bool leaf.
Within each field/array/scalar address group, selectors are least significant
bit first. Products use a field address followed by the padded child address.
Sums use a tag/payload selector; tag is u32, payload is the largest active-arm
product. This storage reserves one active payload, rather than concatenating all
arms. Constructors, active-tag validity and inactive/unused-zero predicates
belong to unit 3. This unit defines their layout, not those predicates.

Widths are the specified integer/IEEE/UTF-16/Guid/time widths. Decimal stores
sign, scale and a 96-bit coefficient explicitly. String capacity is 16,384
UTF-16 units. Sequences/map entries/set elements have capacity 4,096. Validation
invalid payloads retain a role bound of 256 over their shared sequence carrier.
Construction state holds length, 16,384 element slots and initialization bits.
Product children and sum payloads have explicit padding to the maximum child
depth; pad prepends address bits that must be false, and unpad supplies false
for those bits. Public domains and default eligibility remain separate from
the storage-zero helper.

The builder starts from hash-pinned frozen Std.Bool bytes and emits only ordinary
Sort/Var/Const/App/Lam/Pi/Let terms. Pointwise mux calls Std.Bool.rec at Bool
leaves. Concrete `S -> S` composition uses Lam/App/Let, ordered first f then g;
fixed lists use balanced composition and count all leaf occurrences before DAG
sharing against the cumulative 16,384 transformer ceiling. Empty composition
is the identity. No Nat recursor or function-valued recursor is introduced.
Terms are interned; actual term/declaration/binder limits are enforced, including
forward/missing term references. Checkers still validate declaration typing.

`import_csharp_practical_ordinary_carriers` reconstructs both metadata and core
bytes and requires exact equality. A caller-provided digest, inventory or
certificate cannot replace reconstruction. Raw metadata SHA-256 in the golden
file is distinct from the domain-separated Certificate v0 hash. Generated names
use valid alphabetic prefixes and an injective hex encoding of concrete type IDs.
The checkers and Certificate v0 acceptance rules are unchanged.

## Local verification

`carriers/*.hex` are reproducible from the existing binding, exception, transition
and constructor request/fact captures. Cases with intentionally broken source
invariants are carrier positives only: their application VCs are not proved.
`carriers/goldens.json` pins original VIR hashes, layouts, byte hashes and actual
term/declaration counts. Source field order is also checked independently against
captured stored-member declarations. All role layouts and enum/source products
are exercised. Unit tests evaluate helper behavior with a separate finite Bool
interpreter; this interpreter supplies test observations, not trusted proofs.

The Go checker-agreement harness sends exactly the same decoded bytes to both
checkers and requires every carrier certificate to be accepted with zero axioms.
It compares module/declaration/axiom counts and all report hashes, and requires
hash-corrupted bytes to be rejected. Process/build/internal errors cannot pass
as proof rejections. The predecessor corpus remains part of the same local gate.

Targeted commands and results are recorded in `unit-1-verification.json`.
To regenerate candidate fixtures for review, run:

```sh
MPK_W09_CARRIERS_OUT=/tmp/mpk-w09-carriers cargo test -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_carriers_original_inputs_and_mutations
```

Normal tests compare checked-in bytes and layouts without updating them.
The repository-wide `./scripts/check-fast.sh` is deferred to T06-W12, per the
repository rule for intermediate W work. No GitHub Actions workflow is used.

## Internal unit 4 Money component

`money-operations/` contains three concrete instances and all 30 operations.
Both source semantic matrices passed, totalling 100 ordinary observations;
both pinned programs passed the unchanged checkers with zero axioms and rejected
hash corruptions. This preserves the full scope of units 3-8. Unit 3
still has pending deep semantic checks and its final review; neither this Money
component nor the existing structural helper certificates complete W09. See
`unit-4-money-progress.json` for scoped evidence and pending work.

Ordinary VIR block/check literal bodies are now emitted by the unit-4 literal
component. The 64 actual-source contexts contain 88 constants and 105 bindings
across 33 nonempty sources; 31 empty inventories are tracked separately. All
source pins and two supplemental scalar/deep edge pins passed both unchanged
checkers. See `literal-definitions/README.md` and `unit-4-literal-progress.json`.
Boundary-run constants are now lowered separately in `boundary-literals/`: all
three existing boundary-output sources, nine runs and 20,258 bit observations
pass, as do exact pin replay and both unchanged checkers. The complete run hash
remains bound even when provenance changes without changing the literal bytes.
See `unit-4-boundary-literal-progress.json`. Codecs, source relations, proof
assembly and the rest of the original eight-unit plan remain outstanding.

## Internal unit 4 forward projection checkpoint

The ordinary binding projection generator covers all twelve frozen roles with
exact source/binding/closed-instance linkage. 44 original-source contexts pin
34 projection definitions; 144 core value comparisons cover mapped tags, nested
elements, zero inactive storage and unmapped stored members. All 44 same-byte
certificates pass both unchanged checkers with zero axioms and reject hash
corruptions. See `binding-projections/README.md`,
`unit-4-projection-progress.json` and `unit-4-projection-review.md`.

No nonidentity inverse or source/target-domain/commutation proof is inferred from
forward conversion. The remaining unit 4 business/codec/reconstruction work,
units 5-8, unit 3's deep checks/full review and complete W09 acceptance stay open.

## Internal unit 4 binding-relation work in progress

Proof-level W06 comparisons now preserve admitted observations, including NaN
bits and signed zeros; native IEEE foundation equality is unchanged. 45 original
source contexts produce 325 predicate occurrences and 37 result agreements,
including the actual normal-commutation comparison for a captured Make method.
126 ordinary comparisons and exact fixed replay passed. 2,401 unresolved VC
symbol occurrences remain explicit, including linear construction-state equality.

The previous 44 relation certificates are retained under `binding-relations/
pre-observation-agreement` as superseded evidence: both checkers accepted their
types, but their IEEE proof-comparison choice was incorrect. Current verification
and the completed 45-case checker job are in `unit-4-binding-observation-progress.json`.
See its review for the correction. This component supplies no reconstruction
witness, native-body theorem, or accepted application certificate. The complete
original W09 plan remains open.

## Binding guards (partial unit 4)

The source-tag, semantic-arm, payload, member-projection, semantic-bound and
nonempty-invalid predicates now have ordinary definitions. The 45 pinned
programs in `binding-guards/` resolve 141 additional W06 symbol occurrences.
Exact regeneration, importer mutations and transitive preservation of the
existing projection/observation/agreement definitions pass. Source and deep
member observations and both full checker runs are tracked separately in
`unit-4-binding-guard-progress.json`; the scope and direct review are recorded
in `binding-guards/README.md` and `unit-4-binding-guard-review.md`.

This does not close unit 4 or W09. Reconstruction, source/native witnesses,
canonical codecs, transition/replay and proof assembly retain their full scope
in `implementation-plan.md`. No application-proof or gate completion follows
from the well-typed helper definitions or finite observations.

## Binding canonical order (partial unit 4)

`binding-orders/` contains nine actual-source contexts with ten ordinary
canonical-order predicates. The definitions compare only adjacent active
keys/elements using the frozen total-order matrix, enforce the full 32-bit
length bound and reject duplicate keys independently of map values. All 45
existing guard pins are preserved; the 36 contexts without order obligations
retain identical certificate bytes. Exact replay, importer mutations, base
definition closure equivalence, lint, format and inventory checks pass.

The 140-case semantic test, full 4,096-element map/set cases and both checker
runs are tracked independently in `unit-4-binding-order-progress.json`; their
completion must not be inferred from generation checks. See
`unit-4-binding-order-review.md` for direct review scope. All remaining W09
internal units and the unchanged final acceptance gate remain open.

## Source reconstruction with explicit completions (partial unit 4)

`binding-rebuilds/` pins 45 actual-source contexts with 37 ordinary binary
reconstruction definitions. Mapped semantic fields replace their source fields;
unmapped and inactive source-product fields come from the explicit completion.
All 35 nonidentity unary reconstruction witness occurrences remain pending.
See `unit-4-binding-rebuild-review.md` and `unit-4-binding-rebuild-progress.json`
for test improvements and completed scoped verification. This does not
complete reconstruction proofs, unit 4, the remaining units, or W09.

## Hexadecimal boundary codecs (partial unit 4)

`hex-codecs/` contains ordinary binary32/binary64 and GUID N/D parsers and
formatters. Eleven actual-source contexts pin 14 codec occurrences, preserving
raw scalar bits and exact lowercase spelling, ordered parse errors, bounded
text and zero inactive output storage. See `unit-4-hex-codec-review.md` and
`unit-4-hex-codec-progress.json` for independent oracle coverage and actual
verification status. Other codec families, JSON and the full W09 proof scope
remain outstanding.

## Canonical integer formatting (partial unit 4)

`integer-formats/` adds ordinary decimal output for the eight integer types,
duration ticks and instant milliseconds. 65 captured source contexts produce
54 pinned programs and 86 formatter occurrences, using original-width unsigned
magnitudes and shared finite quotient/remainder stages. Independent semantic,
pin and checker results are tracked in `unit-4-integer-format-progress.json`;
all 392 ordinary observations have passed. See `unit-4-integer-format-review.md`.
Registered codec linkage and all remaining
W09 codec/proof requirements stay open.

## Canonical integer parsing (partial unit 4)

`integer-parsers/` pins 54 programs with 86 ordinary parser definitions for the
same ten integer/duration/instant codecs. Full UTF-16 syntax, canonical spelling,
width-specific range and full input-bound precedence are represented by ordinary
terms. The 456-case independent semantic matrix passed; full-capacity execution,
pin replay and both-checker verification are tracked separately in
`unit-4-integer-parse-progress.json`. See `unit-4-integer-parse-review.md` for
scope and pending registered-result linkage and proofs. This does not complete
unit 4 or W09.

## Normalized decimal formatting (partial unit 4)

`decimal-formats/` documents the new ordinary 96-bit digit pipeline, bounded
trailing-zero removal and normalized text layout. Its implementation and
actual-source tests are tracked in `unit-4-decimal-format-progress.json`.
Compilation, six same-byte zero-axiom checker cases, exact pin replay, lint and
inventory have passed; the ordinary semantic matrix passed all 72 cases.
Fixed-scale rounding, parsing and all original codec/proof requirements remain
open.

## Fixed decimal formatting (partial unit 4)

`decimal-fixed-formats/` pins six ordinary programs covering all 145 fixed-scale
rounding configurations per decimal source. Both unchanged checkers accepted
all six with zero axioms and rejected hash mutations; exact fixed/normalized
pin replay, lint and inventory passed. The ordinary semantic matrix remains
running in `unit-4-decimal-fixed-format-progress.json`. Parsing, exact registered
linkage, universal proofs and the remaining original W09 units stay open.

### Unit 4 decimal parser checkpoint

Ordinary parsing definitions now cover normalized decimal and every fixed
scale/rounding configuration. The implementation preserves input-bound,
syntax, noncanonical, precision, and range precedence, and reduces excessive
coefficients by fractional trailing zeroes only as permitted by the frozen
codec. Source regeneration/import/mutation and exact pinned replay passed across 65
contexts, producing six programs with 146 configurations each. Compilation,
lint and all five inventory tests passed. All-bit semantic execution passed
446 observations; same-byte checking passed all six certificates with zero axioms
and hash mutation rejection.
See `unit-4-decimal-parse-progress.json` and `unit-4-decimal-parse-review.md`.
Registry/source linkage, universal codec proofs and all remaining original
W09 work remain open; this is not an internal-unit completion or commit.

### Unit 4 date/time codecs (verification running)

Ordinary `date` and `time` parse/format definitions preserve the fixed spelling,
Gregorian ranges, seven fractional digits and input-bound/syntax/range order.
All 67 source contexts regenerated, producing three pinned programs. Existing
Gregorian circuit certificates and metrics are unchanged. Compilation, lint,
formatting, all five inventory tests and exact 67-context/three-certificate
pin replay passed. All 795 semantic cases and three same-byte checker cases
passed; scoped review found no additional issue. See
`unit-4-calendar-codec-progress.json` and `unit-4-calendar-codec-review.md`.
These definitions do not complete source/registry linkage, universal codec
proofs, strict JSON, unit 4 or the original W09 acceptance gate.

### Completed scoped semantic checks

The normalized decimal formatter matrix passed all 72 cases (5841.65 seconds),
including all scales, signs, coefficient limits and cohorts. Its direct scoped
review has no further findings; universal codec/source proofs remain open.
The decimal parser's four full-16,384-unit cases passed (2039.92 seconds);
its separate configuration and short-input matrix passed all 446 observations
(14167.71 seconds). Date/time
codec semantics passed all 795 cases (901.46 seconds), and all three same-byte
checker cases passed (1694.257 seconds). These results do not complete an internal unit or W09.

### Unit 4 boundary document capacity correction (verification running)

W09 found that the pending W07 boundary VC typed a document as an application
string, whose 16,384-unit storage cannot retain the full 1 MiB transport.
Document binders and encoder results now use a private ordinary C24 UTF-8 byte
carrier with full length, bounded access and canonical storage helpers.
Application string types and checker rules are unchanged. All 68 source/storage
cases, three same-byte checker cases, exact pin replay, and four affected W07/W09
linkage tests passed. All nine literal certificate bytes were preserved. Current
compilation/lint/formatting and all five final inventory tests passed. The scoped
review found no additional issue.
Archived pre-correction goldens and verified comparisons are recorded in
`boundary-documents/linkage-correction.json`. See
`unit-4-boundary-document-progress.json` and `unit-4-boundary-document-review.md`.
This storage fix does not complete JSON grammar/encoding, source linkage,
universal proofs, an internal unit or W09.

### Boundary UTF-8 definitions (verification in progress)

The ordinary byte DFA now covers the full private 1 MiB document carrier.
Initial actual-source tests caught static-transformer budget exhaustion; the
scan now counts two 64-byte blocks per step and 8,192 outer steps together with
all helper circuits, retaining the frozen limit. Source/pin, semantic, block
boundary and full-document tests are in progress. See
[the progress record](unit-4-boundary-utf8-progress.json) and
[the scoped review](unit-4-boundary-utf8-review.md). UTF-8 validity does not close
JSON grammar, typed boundary/source relations, universal proofs, internal unit 4
or W09. No component-only commit or completion receipt is issued.

### Canonical JSON string encoding (verification in progress)

The ordinary encoder converts the complete admitted UTF-16 string into a quoted
private byte document, preserving lone surrogates and emitting shortest UTF-8
for scalar values and surrogate pairs. All-control input can produce 98,306
bytes, beyond application-string storage and within the document bound. See
[progress](unit-4-json-string-progress.json) and
[direct review](unit-4-json-string-review.md). Source and semantic verification
is in progress; parsing, typed boundary relations and universal proofs remain
open. This component does not complete internal unit 4 or W09.

### Canonical JSON string parsing (verification in progress)

The ordinary quoted-string parser validates UTF-8 and canonical escapes while
reconstructing up to 16,384 UTF-16 units, including lone surrogates. It rejects
trailing data, noncanonical escaped surrogate pairs and decoded-length overflow.
See [progress](unit-4-json-string-parse-progress.json) and
[direct review](unit-4-json-string-parse-review.md). General JSON grammar, typed
fields and universal source/boundary proofs remain open; this does not complete
internal unit 4 or W09.

### Quoted-string prefix consumption (verification in progress)

The ordinary string parser now returns a prefix value and exact consumed byte
count while its whole-document wrappers still require the closing quote to be
the document end. Shared byte-fragment definitions allow direct ordinary
Slice-to-parser composition in the same certificate. Current 68-context source
generation/pinned replay and all three same-byte checker cases passed, along
with document/u32 bounds, twelve high-offset compositions and inventory checks.
The 62-case prefix and 296-case whole-document matrices also passed. The
full-decoded-bound/consumption runs remain live. See [progress](unit-4-json-string-prefix-progress.json) and
[review](unit-4-json-string-prefix-review.md). General typed JSON grammar,
source relations and universal application proofs remain open.

### Money integration into the ordinary program checkpoint

The integrated program now includes every Money operation and shares its
Builder, relation/storage caches and counted pipeline. All 44 captured source
programs passed generation/import/mutations and exact replay. Complete
component comparison passed 118 pairs, 1,235 roots and 14,183 transitive
declarations. All three Money instances retain explicit source currency
predicates; those arguments are not admitted application proofs. The 44-case
same-byte checker run passed all 44 certificates with zero axioms and hash
mutation rejection (5570.951s). See
[progress](unit-4-integrated-money-progress.json) and
[review](unit-4-integrated-money-review.md). Units 3–8 remain incomplete.

### Byte-document composition (component verification passed)

Ordinary slicing and concatenation preserve exact bytes and enforce full-u32
ranges and the cumulative 1 MiB bound. Invalid ranges/sums produce a zero
document; canonical output masks inactive storage and header padding. Source
generation/pinned replay, three same-byte checker cases, lint/format/inventory
and both complete semantic/full-address subtests passed (561.49 seconds). See [progress](unit-4-boundary-fragments-progress.json) and
[review](unit-4-boundary-fragments-review.md). These functions support subsequent
typed JSON composition; they do not establish UTF-8/JSON grammar or universal
boundary/source proofs. Internal unit 4 and W09 remain open.

### JSON keyword recognition (component verification passed)

Ordinary terms now recognize the exact lowercase `null`, `false` and `true`
prefixes in a bounded byte document. The C6 packet separates prefix success,
whole-document success, null/value kind, Boolean payload and a full u32 consumed
count. Every invalid result and every unused packet bit is zero. The lexical
prefix leaves following bytes and delimiter validation to the enclosing grammar.
The same certificate includes byte Slice/Concat for direct core composition.
Source reconstruction across 68 contexts, 401 lexical cases, full-u32 bounds
and direct compositions passed. All three fixed certificates passed exact
replay and same-byte zero-axiom Rust/Go checking with hash-corruption rejection.
Inventory, lint, format and direct component review passed. General typed JSON
and boundary VC proofs remain incomplete.
See [progress](unit-4-json-keyword-progress.json) and
[review](unit-4-json-keyword-review.md). Units 3–8 and W09 remain open.

### Shared JSON lexical program (verification in progress)

The shared program emits one byte-document/fragment environment for canonical
strings, keywords and u8/u16/u32/u64 metadata numbers. Twenty counted numeric
steps plus lookahead enforce greedy digits, no leading zeros and exact u64
and field-width overflow behavior. Whole/prefix results remain distinct;
enclosing typed grammar owns suffix delimiters and field/structure validation.
See [progress](unit-4-json-token-progress.json) and
[review](unit-4-json-token-review.md). Both standalone string/keyword replays preserved all prior metadata and
certificate bytes. Inventory passed. The first integrated run found duplicate
C7 helper registration; corrected generation, pinned replay and both-checker
verification passed. The 228-case unsigned semantic matrix also passed; its verified checkpoint
is archived under json-tokens/previous-unsigned while the signed extension is
verified separately. This does not
complete internal unit 4 or W09.

### Signed semantic JSON token checkpoint

The shared lexical program now adds i8/i16/i32 raw integer token wrappers using
ordinary Slice and Unsigned64 magnitude parsing. Signed limits, negative zero,
full original document bounds, exact counts and two's-complement bits are
explicit ordinary circuits. Tests also compare every earlier term/declaration
against [the unsigned archive](json-tokens/previous-unsigned/certificates.json).
See [progress](unit-4-json-signed-token-progress.json) and
[review](unit-4-json-signed-token-review.md). Semantic i64/u64 remain quoted codec
values. The 126-case signed matrix, source/pin preservation and same-byte
Rust/Go checks of all three certificates passed. The signed checkpoint is
archived under json-tokens/previous-signed. Complete typed grammar and application
proofs remain outstanding.

### Scalar JSON grammar consumer checkpoint

The shared program now appends bool/null and small integer consumers that take
an absolute u32 start cursor and an exact expected ending (EOF, comma, array
close or object close). They return the typed value and absolute end without
consuming the delimiter. Full original document bounds, cursor arithmetic and
delimiter existence are checked; invalid packets are entirely zero. Existing
lexical definitions are retained and compared against both archived checkpoints.
Only the new grammar cases, source/pin linkage and enlarged certificate checking
need execution; unchanged lexical semantic matrices are reused after exact
preservation succeeds. See [progress](unit-4-json-scalar-progress.json).
All 119 grammar cases, source/preservation, the distinct-certificate checker case,
lint and format checks passed. The metadata kind-label correction preserved every
certificate byte and changed only the discriminator key in the manifest.
This is not complete typed object/array grammar or unit 4 completion.


### Shared collection/JSON integration (verification in progress)

The full 8,192-transformer collection pipeline and old 8,744-transformer JSON
program could not share the 16,384 limit. JSON Scan now groups adjacent pairs
into StepFour and retains exactly 16,386 ordered packet steps. Both affected
source suites passed expanded-sequence and all-other-definition comparisons.
JSON now costs 4,650 transformers, and the combined full collection/JSON fixture
passed generation at 12,842 transformers, 53,948 terms and 756 declarations.
Existing C19/C24 storage helpers are reused. No limit or checker rule changed.

The shared-emitter test also confirms exact standalone bytes after helper reuse.
The combined fixture is source-free definition integration; complete actual-source
assembly and application proofs remain open. Targeted boundary/capacity cases,
distinct-input dual-checker cases and two affected consumer-edge inventory tests
are recorded in [progress](unit-4-json-collection-integration-progress.json) and
[review](unit-4-json-collection-integration-review.md). Prior lexical checkpoints
are retained under their archive directories. W09 remains incomplete.

The three distinct regrouped/shared certificates passed same-byte Rust/Go checking
with zero axioms and hash mutation rejection. The two affected consumer-edge
inventory tests and final lint/format checks passed. Group-boundary and capacity
runtime checks remain in progress; this does not complete actual-source assembly.


### Actual-source structural/boundary integration (verification in progress)

Boundary mode now emits source-specific structural/Money and shared JSON
definitions through one Builder, retaining the exact boundary-program hash.
Its distinct schema and import path reject source/context/mode substitutions.
Four actual-source cases passed complete component declaration closure and limits;
the largest has 96,775 terms, 1,369 declarations and 13,252 transformers. The
no-boundary case preserves the previously verified unit/void certificate bytes.
Existing structural replay and three new distinct checker inputs are tracked in
[progress](unit-4-structural-boundary-progress.json) and
[review](unit-4-structural-boundary-review.md). Full typed grammar, source relations
and application proofs remain outstanding; W09 is not complete.


### Framed JSON strings (component verified)

Shared JSON now parses canonical strings at a full-u32 absolute cursor and
checks an exact enclosing ending, including the colon required after a field
name. One bound scan supplies the decoded UTF-16 value, absolute end, decoded
length and validity. Invalid results are entirely zero; the delimiter is left
for the enclosing grammar. Original document bounds and live-byte checks remain
independent of slicing. All previous lexical declarations are preserved.

The new 44 runtime cases, 68 source contexts, four structural/boundary contexts,
full collection pipeline integration and affected lint/consumer checks passed.
The shared JSON program uses 4,713 transformers, the full collection integration
12,905, and the largest actual-source integration 13,315, all below 16,384.
Token and collection certificates passed both checkers, as did all three changed
actual-source structural vectors. The last structural case required a targeted
retry after a Rust CLI rebuild read an incomplete subsequent edit; its retry
passed without rerunning the first two cases. This checkpoint remains under
`previous-quoted-scalars` after the next extension. See [progress](unit-4-json-string-framing-progress.json)
and [review](unit-4-json-string-framing-review.md); completed verification logs
are retained beside these records. Full typed grammar, native/transition source
relations and proof assembly remain outstanding. W09 is not complete.


### Quoted semantic scalars (verification in progress)

Quoted char, i64, u64, duration ticks and Unix milliseconds now use the canonical
string frame. Char requires one UTF-16 unit; numeric codecs require complete
consumption of the exact quoted interior and retain all sign/canonical/range
checks. The raw scalar type list stays unchanged. All 68 original source
contexts passed old-definition closure and import/mutation checks. JSON uses
4,787 transformers; full collection coexistence uses 12,979. The distinct JSON
certificate passed both checkers with zero axioms. All 70 runtime cases and four
source-specific structural generation/closure/import cases passed. Intermediate
structural bytes were superseded before dual checking by the next quoted-codec
extension; no checker acceptance is claimed for those archived intermediate bytes. See [progress](unit-4-json-quoted-scalars-progress.json)
and [review](unit-4-json-quoted-scalars-review.md). Remaining typed grammar,
cumulative limits, source/native/transition relations and proof assembly are
still required before W09 can complete.


### Quoted floating-point and GUID codecs (verification in progress)

The shared JSON environment reuses the exact binary32, binary64, guid.n and
guid.d ordinary codec emitters. Each wrapper decodes a complete canonical JSON
string, checks codec success and the enclosing ending, and returns every payload
bit plus the absolute cursor. The C8 result preserves NaN payloads, signed zero
and all 128 GUID bits, with zero padding and a fully zero invalid result.

All 68 source contexts passed lexical preservation and complete parse/format
dependency comparison against the existing floats/guid certificates. The three
new JSON certificates are byte-identical, with 68,973 terms, 1,073 declarations
and 4,810 transformers. Targeted lint/format and the two affected consumer-edge
tests passed. Runtime, shared-collection and final structural/checker results
are tracked in [progress](unit-4-json-quoted-codecs-progress.json) and
[review](unit-4-json-quoted-codecs-review.md). Full typed grammar, cumulative
limits, source/native/transition relations and actual proof assembly remain
outstanding; W09 is not complete.

### Aggregate regrouping for shared decimal/JSON generation (in progress)

The aggregate fold now composes four original guarded `StepTwo` operations
into `StepEight`, retaining the complete 16,384-cell scan. This lets the full
collection, JSON and decimal definitions coexist at 15,434 cumulative static
transformers. Old-definition preservation and audit mutations pass; affected
parser, full-capacity and same-byte checker verification is tracked in
[the progress receipt](unit-4-aggregate-step-eight-progress.json) and
[the component review](unit-4-aggregate-step-eight-review.md).
This is partial W09 implementation; typed schema grammar, remaining relations,
actual propositions/proofs and the original acceptance gate remain open.

### Quoted decimal tokens (component verification passed)

The shared JSON environment now reuses all 146 normalized/fixed decimal
parser configurations. Quoted decimal packets preserve the complete C9 value,
absolute end and framing flags, and zero every invalid result. Full collection,
JSON and decimal generation fits at 15,453 cumulative transformers; standalone
decimal certificate bytes remain unchanged. All 146 closed-setting samples
and the 28 complete token packets passed. See
[the progress receipt](unit-4-json-quoted-decimal-progress.json) and
[the component review](unit-4-json-quoted-decimal-review.md) for the selected
checks and pending work. This adds a token component, not complete typed JSON
grammar or W09 proofs.

Aggregate impact correction: the direct module calls and transitive consumers
add twelve certificate families beyond integer/decimal parsers. All twelve now
pass source generation and full old-definition/non-cost-metadata preservation:
133 vectors reassociated, 92 byte-identical. Their 122 distinct changed byte
vectors passed both unchanged checkers, zero-axiom checks and hash-corruption
rejection. Earlier verification receipt hashes identify
the archived bytes mapped in
[the repin map](unit-4-aggregate-step-eight-repin-map.json); they do not certify
the new root bytes; the completed current check is recorded in
`unit-4-aggregate-step-eight-progress.json`. The outstanding domain runtime
cases remain separate and do not establish completion of unit 3 or W09.

### Closed JSON field syntax (in progress)

Ordinary literal matchers now cover punctuation and exact canonical field names
from sidecars, source members and semantic value vocabularies. They preserve
lone UTF-16 surrogates, exact cursors and zeroed invalid packets. The targeted
runtime test passes 111 complete packets; 68 original source contexts generate
three pinned programs, including a contract without fields. See
[the component review](unit-4-json-syntax-review.md) and
[the verification receipt](unit-4-json-syntax-progress.json) for current results.
Typed compound grammar, field presence and boundary proofs remain outstanding.

### Typed primitive JSON results (in progress)

JSON token results now expose exact value carriers and a common header with
validity, absolute end and logical cell count. The component retains full string
and decimal storage, masks invalid packets and rejects field-name endings when
reading string values. Packet tests, 72 original contexts and a source-free
166-route primitive matrix pass. The largest matrix uses 219,046 terms and
8,117 static transformers. Eight distinct pinned vectors passed
both unchanged checkers and hash-corruption rejection. See [the component review](unit-4-json-values-review.md)
and [the verification receipt](unit-4-json-values-progress.json). Compound
grammar, field presence and application proofs remain outstanding.

### Reachable calendar JSON tokens (in progress)

Date/time JSON tokens use the ordinary calendar parsers and canonical string
frame, selected from reconstructed reachable carriers. Four actual C# captures
cover date, time, both, and both with decimal. BCD formatting and direct
128-gate format blocks resolve the cumulative limits without dropping required
definitions or bypassing accounting. The largest source program fits at
243,567 terms, 3,199 declarations and 8,490 transformers; the full shared
collection/calendar/JSON environment fits at 207,455 terms, 2,780 declarations
and 16,293 transformers. Source/import/mutation checks, archived parser/token
preservation, 30 complete token packets and 68 formatting/round-trip cases pass.
Ten distinct current byte sequences from twelve complete pins passed
both unchanged checkers and hash-corruption rejection. See [the progress receipt](unit-4-json-calendar-progress.json)
and [the component review](unit-4-json-calendar-review.md).

The earlier calendar-codec verification receipt applies to the exact bytes
archived in `calendar-codecs/previous-json-format`; intermediate 32-gate vectors
are retained under `previous-wide-blocks` with their verification status recorded
separately. Complete typed JSON grammar and W09 units 3-8 remain open. These
ordinary definition certificates do not establish application propositions or
proofs, and no component-only commit is made.

The source-product JSON component now composes exact stored-member syntax,
typed primitive/compound packets, checked cursor/cell/depth headers and original
source storage constructors. Seventy-two original contexts preserve complete
primitive and syntax dependency closures; seven complete programs contain four
source-product parsers. Ten actual-core calendar/decimal product cases pass all
packet bits, including field order, missing/duplicate fields, depth and padding.
Two additional original C# captures exercise nested and empty products; their
replay, dependency preservation, import/mutation checks and all 13 actual-core
packet cases pass. Both nested/empty vectors passed both unchanged checkers;
all seven earlier source-product vectors also passed both unchanged checkers.
See [the product progress receipt](unit-4-json-products-progress.json) and
[the component review](unit-4-json-products-review.md). Complete boundary grammar,
proofs and W09 units 3-8 remain outstanding.

Source enums now parse quoted canonical underlying integers and reject values
outside their declared set before narrowing to the original carrier. An actual
C# capture covers all eight underlying types and a product holding each.
The 52 complete converter packets, source/import/mutation checks and exact
preservation of nine prior product vectors pass. The new complete certificate
contains 146,028 terms and 2,102 declarations. All 115 actual-core packet cases
and its same-byte dual-checker run passed. See
[the enum progress receipt](unit-4-json-enums-progress.json) and
[the enum component review](unit-4-json-enums-review.md). Builtin enum/error results are recorded below; collections/sums, absence/null
rules and complete W09 proofs remain outstanding.

Builtin day-of-week and parse-error now have ordinary typed JSON parsers. An
actual day-of-week source product and a parse-error type-contract literal
exercise both reachable carriers. All 32 vocabulary match masks and original
source/import/mutation checks pass; the shared source-enum certificate is
unchanged. The new complete program has 141,862 terms and 2,046 declarations.
All 39 actual-core packet cases and the unchanged dual-checker run pass.
See [the builtin progress receipt](unit-4-json-builtins-progress.json) and
[the component review](unit-4-json-builtins-review.md). Remaining semantic
products/collections/sums, absence/null rules and complete W09 proofs stay open.

Closed ordered-entry and Money products now select their exact template fields
and compose existing typed child parsers. Captured bindings cover enum/string
Money, an ordered entry and four source products. Source/import/mutation and
representative prior-byte preservation checks pass. The complete seven-product
certificate (146,723 terms,2,263 declarations) passed both unchanged checkers;
all 16 actual-core cases passed, with full small packets and explicitly
selected wide-carrier probes. See [the progress receipt](unit-4-json-semantic-products-progress.json),
[the component review](unit-4-json-semantic-products-review.md) and
[remaining composite rules](unit-4-json-composite-followup.md). Transition,
collections/sums, absence/null and the remaining W09 proofs stay open.

The compound-header join now supports excluding an implicit JSON container's
cell before checking the parent total. All 99 complete packet cases pass,
and shared source/Money/ordered-entry certificate bytes are preserved.
The separately generated join certificate passes both unchanged checkers with
46 declarations and zero axioms. Caller integration for Transition/map
containers remains pending; see [the join receipt](unit-4-json-container-join-progress.json)
and [the join review](unit-4-json-container-join-review.md).

Recursive bounded-sequence JSON parsing now composes bool/int arrays and arrays
of source structs containing arrays. Source/partition/import checks pass, and
the existing long[] consumer's updated certificate passes both checkers. A
checker finding in the shared definition helper was fixed: cube depths at least
32 now retain their full input type. Narrow definition bytes are preserved;
the five affected wide runtime cases pass. Capacity-bound and final checker
status are tracked in [the sequence receipt](unit-4-json-sequences-progress.json)
and [review](unit-4-json-sequences-review.md). Whole boundary and remaining W09
implementation/proofs remain outstanding.

### Tagged JSON value progress

The recursive compiler now emits the frozen option/lookup/result/validation/
boundary_field sum syntax and active-arm storage, including Validation's
nonempty 256-error role bound. The existing document fixture newly resolves
Option<i32>, Option<string>, and its enclosing source product. The changed
certificate and its prior bytes are linked in `unit-4-json-sums-progress.json`;
verification of the additional roles and nested sums remains outstanding.
This is implementation progress inside unit 4, not W09 acceptance.

The corrected recursive sequence certificate has now passed both unchanged
checkers on identical bytes, with zero axioms and matching reports. Its
4095/4096/4097 original-document capacity run is still tracked separately in
`unit-4-json-sequences-progress.json`.

The tagged-value follow-up now has accepted original-source fixtures for all
five template roles and Result<Option<int>,bool>. Twenty-six complete narrow
core packets and four selected wide Option<string> packets passed. The original
Validation 0/1/256/257 documents and new same-byte checker run are tracked in
`unit-4-json-sum-roles-progress.json`. The original validator's rejection of
Option<Option<int>> is preserved; no profile rule was relaxed.

### Ordered JSON collection progress

Map/Set parsing now reuses complete typed sequence storage and the existing
semantic key relations, enforces strict order at every append, and excludes
only Map's implicit entry-wrapper cell. Four accepted original-source contexts
cover integer, decimal, string and compound keys. Source reconstruction,
metadata mutations, primitive/syntax dependency preservation, exact unchanged
array/sum bytes, high-slot admission and targeted lint/inventory checks passed.
The complete-document semantic cases and four new same-byte dual-checker cases
are still running; see `unit-4-json-ordered-progress.json`. This remains partial
unit4/W09 work, with Transition and full boundary relations outstanding.

### Transition JSON progress

Typed Transition parsing now preserves state, ordered/repeated events and
response, excluding only the events array wrapper from semantic cells. Two
accepted original C# contexts include compound values and a shared Map parser.
Source reconstruction, metadata mutations, complete primitive/syntax dependency
preservation, unchanged Map/Set and sum pins, request replay and affected lint
passed. Runtime and the two new identical-byte checker cases remain tracked in
`unit-4-json-transition-progress.json`. Full boundary relations and W09 units3-8
remain open; this does not complete W09.

The Transition component's24 original-document runtime cases and both new
identical-byte checker cases have now passed; the receipt records terminal
results and the limited storage-observation scope.

### Missing/null boundary predicate progress

The actual W07 MissingRule and NullRule symbols now have ordinary definitions
over projected source values, using exact frozen typed constants and existing
observation equality. Required/null rejection and presence/Option distinctions
are preserved. Verification of15 original boundary contexts and the subsequent
new-candidate checker run is tracked in `unit-4-boundary-rules-progress.json`.
Input admission, reconstruction invariants and remaining boundary/proof work
stay unresolved; this remains partial unit4/W09 work.

### Compound and directly projected JSON input depth progress

Eleven original compound source contexts now pass complete parse-header and
canonical typed-depth observations. Five additional original C# contexts bind
the actual input root to Money, OrderedEntry, OrderedSet, OrderedMap or
Transition; their offline double captures match, and runtime tests pass full
headers, selected argument bits, depth boundaries and wrong-source-format
rejection. These scopes are deliberately distinct: nested source representations
do not stand in for directly projected semantic roots. Candidate metadata and
bytes are pinned under `json-depth-compound/` and `json-semantic-root/`.
Their new same-byte dual-checker runs are tracked in the respective progress
records. The earlier15 field decoder candidates now pass both checkers.

The affected consumer closure test exposed an old aggregate4944 count after
two already-recorded Std namespace consumers;the expectation now matches4946,
and all136 unchanged fingerprints and consumer edges pass. Final affected
integration lint and scoped formatting pass. These are partial unit4 results;
source reconstruction, complete boundary relations and the remaining W09
implementation/proof/assembly/acceptance scope remain open. The T-wide gate
remains deferred toT06-W12.

### Canonical typed JSON node-count progress

Typed JSON now has separate ordinary node-count definitions. Strings/scalars
count as one node;objects,arrays,explicit sum tags,Map entry objects and
Transition event arrays count as JSON nodes. Counts saturate at262145 above the
inclusive262144 limit. Active sequence elements use the existing finite sum
pipeline through quotient/remainder accumulation, preserving its65537 cell
saturation while computing the larger JSON-node bound exactly.

268 arithmetic/container cases and53 value observations across31 original
source contexts pass against independent canonical JSON trees. A test-only dense
allocation at carrier depth35 was corrected to the existing sparse storage
oracle;the same16 compound cases then passed16.04s. Final affected lint,format
and consumer closure pass. All31 source programs plus one helper are pinned in
`json-typed-nodes/` and all32 candidates passed unchanged same-byte dual checking,zero axioms
and hash mutations in461.820s;see `unit-4-json-typed-nodes-progress.json`. Envelope admission composition,raw-tree
limits,source reconstruction and the remaining W09 implementation/proof scope
remain open. No T-wide gate or component-only commit/push was performed.

### Typed-node guard integration progress

The new typed-guarded envelope variant applies canonical node bounds after the
existing depth-guarded parser,to every decoded/defaulted argument. Any failed
field or prior parse invalidates the complete packet. Old unguarded/depth-guarded
required-source metadata and bytes remain identical. The guard passes2560 full
packet-bit observations,including second-field cutoff and a synthetic failed
parse retaining nonzero payload. Sixteen compound/direct-root programs preserve
all prior parser and node-count definitions and exact metadata;their expensive
parser runtime is not repeated. The16 base-source full-packet runs and new-byte
dual checking remain tracked in `unit-4-json-typed-guard-progress.json`.
This is partial unit4 work;raw-tree/source/reconstruction/output and the full
remaining W09 proof/assembly/acceptance scope remain open.

### Raw canonical JSON limit scan progress

An ordinary byte scanner now counts raw JSON value nodes and nesting depth,
excluding object keys and string contents while retaining escape/primitive state
across blocks. Syntax/UTF8 validation remains a separate precondition. The final
32-byte block passes23 complete-state cases;8 blocks per step and4096 steps cover
the1MiB document bound. Generated scan costs are58023terms,223declarations and
4275transformers. Earlier64-byte source/document results are kept historical;
final32-byte source,document and high-address checks are tracked in
`unit-4-json-raw-limits-progress.json`. Raw+typed integration and full W09
source/reconstruction/output/proof/assembly scope remain open.

### Raw and typed JSON guard integration progress

The opt-in limits-guarded envelope combines raw document limits with the
existing typed-depth/node parser and zeroes the entire packet on raw failure.
The actual raw scanner mask passes1024 packet-bit checks;existing original/depth
metadata and certificate bytes remain unchanged. Shared document/cube helpers
are reused rather than redeclared. See `unit-4-json-limits-guard-progress.json`
and its linked review for targeted source/import/compound checks and measured
combined costs. These definitions do not complete source domains,reconstruction,
output,application proofs,assembly or W09 acceptance. No T-wide gate or
component-only commit/push was performed.

### Ordinary source public-clause progress

Imported public-clause terms now lower Var/Const/App/Lam/Let with nominal type
and binder checks to ordinary core. Stored-field references,total Bool/integer
operations and exact Bool/integer literals use existing finite definitions.
Original source/context/foundation/construction/expression linkage is retained;
recipe names map to legal ordinary identifiers. At this initial checkpoint,
unsupported or partial recipes rejected. Subsequent checkpoints below add
recursive PublicDomain equations, richer recipes and separate integer definedness;
use-point obligations, reconstruction and application proofs remain open.
See `unit-4-source-clauses-progress.json` and its direct review for original
construction sources,mixed-width member storage,binder checks and current-byte
checker evidence. This component does not complete unit4 or W09.

Recursive public-domain work now shares the total source-clause builder and
filters reached source values through their declared clauses. The original
nested direct/array/Nullable source and hash-bound capture are retained in
`public-domain-sources/`. See `unit-3-public-domains-progress.json` and
`unit-3-public-domains-review.md` for targeted results and open verification;
this remains partial unit 3/4 work and does not complete W09.

The `structural-public/` profile now integrates existing structural/collection
operations with compiled source public clauses, recursive domains and public
default predicates. Three original contexts passed full definition-closure
comparisons; see `unit-3-structural-public-progress.json` and its direct review
for current verification. Native/body/proof assembly and W09 acceptance remain
outstanding.

Total conditional verification expressions are now ordinary leaf-selection
functions with nominal type checks. `conditional-clauses/` retains the helper,
fresh exact source/contract capture and integrated structural/public candidate;
all three passed same-byte checking. See `unit-4-conditional-clauses-progress.json`
for scoped semantics, prior candidate preservation and remaining recipe/proof
work. This does not complete an internal unit or W09.

Total structural equality and eligible comparison recipes now reuse existing
ordinary semantic relations. The exact source context and two checked candidates
are retained in `structural-clauses/`; both public-domain and integrated consumers
reuse the same definitions. See `unit-4-structural-clauses-progress.json` for
selected verification. Remaining unit/W09 completion conditions are unchanged.

Rich canonical source-contract literals now share the existing typed decoder
and ordinary literal emitter, retaining UTF-16 and repeated structural parts.
`literal-clauses/` records eighteen actual-source constants and their source/
integrated candidates. See `unit-4-literal-clauses-progress.json` for scoped
verification and open work;this does not complete an internal unit or W09.

`sequence_length`, `tagged_is` and `parse_error_kind` now resolve to ordinary
source-contract functions using the existing frozen layouts and shared caches.
`total-clauses/` retains eight source clauses,two checked candidates and scoped
bit/tag evidence. See `unit-4-total-clauses-progress.json`;remaining recipe,
source/native and application-proof requirements are unchanged.


### Separate checked contract definedness

The source-clause builder now emits the canonical W03 definedness term for
supported integer unary/binary checks, separately from the original value. The
same source/expression/attachment identities bind both functions. Guarded
conditional branches and lexical lets retain W03 semantics. Existing total
clauses retain their bytes and metadata. Public domains, defaults and integrated
structural-public consumers reject partial clauses until the original use-point
owner can discharge their definedness; they do not assume a narrower domain.

`definedness-clauses/` records nine source expressions,81 boundary/interior cases
and one ordinary certificate. See `unit-4-definedness-clauses-progress.json` and
its review for verification results. These are helper definitions, not
application proofs. No original internal unit or W09 is complete from this
checkpoint, and the full gate remains deferred to T06-W12.


### Partial sequence and payload contract reads

`sequence_index` now shares the existing checked sequence read and full-width
range predicate;`tagged_payload` shares the active-arm predicate and masked stored
projection. Both retain separate canonical W03 definedness and all public consumers
still refuse them pending original use-point discharge. Fresh and repeated sequence
emission preserves existing bodies. The new13-clause capture,2620 guard/storage
observations and one dual-checked candidate are in `partial-read-clauses/`;see
`unit-4-partial-read-clauses-progress.json` for the targeted verification and
coverage limits. Original internal units and W09 remain open.

### Contract attachment values and definedness

`contract-expressions/` adds ordinary functions for every captured W03 contract
attachment, including method and loop scopes, with exact subject order and
separate canonical definedness. Unsupported recipes reject generation. Identical
public expression hashes on different nominal subject signatures now have
distinct value/definedness names; unambiguous existing output is unchanged.
Three control fixtures, two multiowner source contexts and an independent
old/current/result binding fixture passed targeted verification, together with
nine same-byte dual-checked certificates. See
`unit-4-contract-expressions-progress.json` and its direct review for evidence.
Concrete control use-point instantiation and application proofs remain open;
this checkpoint does not complete an original internal unit or W09.

### Exceptional contract predicates and payloads

`exception-clauses/` connects contract type predicates and payload reads to the
existing finite exception definitions, preserving ancestry and exact-tag W03
definedness. Source regression testing also corrected the data attachment
recheck's missing exception scope. Two source contexts cover nine built-in
types, a three-field user exception, guarded reads and 2,808 observations; both
new certificates pass unchanged same-byte dual checking. See
`unit-4-exception-clauses-progress.json` and its review for evidence and scope.
Concrete control use-point bindings and application proofs remain open.

### Bounded contract quantifiers

`quantifiers/` defines half-open forall/exists for all four integer range types,
with full-width range checks, static literal expansion rejection and universal
body definedness. The endpoint/reversed-range rules follow the user's explicit
clarification. Core range cases, a full-capacity traversal, actual offset-selector
observations and original-source attachment cases passed. See
`unit-4-quantifier-progress.json` and its review for exact verification status,
including the separate same-byte checker gate. This component does not complete
an original internal unit, prove application VCs or advance W09/W10 status.

### Tagged contract construction

`tagged-make/` connects all five admitted tagged construction families to the
existing ordinary stored-sum constructors, retaining exact nominal payload
types and absent-payload parameters. A fresh original-source context covers
13 clauses,12 constructor aliases with complete dependency equality,and2,406
storage observations. The new certificate passes both unchanged checkers on
identical bytes,with zero axioms and hash-corruption rejection. See
`unit-4-tagged-make-progress.json` and its direct review for scoped verification.
The original internal units and application-proof work remain open.

### Transition contract field projections

`transition-clauses/` connects state, ordered events and response expressions to
the existing nominal Transition getters. Two fresh scalar/compound source
contexts cover six clauses and getter aliases, complete dependency equality,
and999 field/selector/tail observations. Both new candidates pass unchanged
same-byte dual checking with zero axioms and hash-corruption rejection. See
`unit-4-transition-clauses-progress.json` and its review for targeted evidence.
Actual native outcome instantiation and transition/replay/application proofs
remain part of the original outstanding W09 scope.


### Scalar codec contract connections

`integer-codec-clauses/` connects the eight integer codecs plus Duration and
Instant. `fixed-codec-clauses/` adds binary32/binary64, GUID N/D and Date/Time,
including a mixed-family source context. The compiler aliases existing ordinary
codec definitions and preserves exact nominal Result types and W03 format
bounds. Targeted source/alias tests and identical-byte checker gates passed;
see `unit-4-integer-codec-clauses-progress.json` and
`unit-4-fixed-codec-clauses-progress.json` for counts, receipts and direct reviews.
The fixed-codec extension preserves the prior 11 hex, 3 calendar and 10 integer
contract certificate pins; prior unchanged arithmetic runtime suites were not
repeated. `decimal-codec-clauses/` now implements normalized and all 145 fixed
scale/rounding adapters with shared digit helpers. Its 29 preservation pins and
six final candidates have same-byte dual-checker evidence. A runtime assertion
corrected an erroneous maximum-value expectation; all affected fixed-mode
conditions now pass. The unchanged normalized context reuses earlier runtime
and checker evidence on identical bytes, as documented in
`unit-4-decimal-codec-clauses-progress.json`. All remaining
original W09 work stays open. These helper certificates do not prove
application VCs.


`floating-clauses/` connects all 38 retained floating operations and six numeric
conversions to the existing ordinary definitions. Five source contexts pass
all 44 alias/dependency checks and 18 selected runtime paths, including two
checked-overflow definedness rejections. An oversized whole-double source still
rejects the frozen limit. All five same-byte Rust/Go checker cases passed;
reconciled evidence is recorded in `unit-4-floating-clauses-progress.json`.
This does not complete an original unit
or provide native-body/application proofs.


`decimal-clauses/` connects decimal unary/binary recipes with independent
arithmetic failure predicates. Existing 45-operation metadata, 14 decimal
certificate pins, four quantifier tests and 22 prior source certificates pass.
Nine accepted native-source inputs cover 44 public operations; internal
value_equality remains covered by the adapter signatures. All ten selected source conditions (five failures) and all nine same-byte
checker cases have passed; the exact combined evidence is recorded in
`unit-4-decimal-clauses-progress.json`. This does not complete an original unit
or W09.


`collection-clauses/` has completed its component verification: nine source
contexts, 18 aliases and 5,598 ordinary observations passed. Current-code
regeneration preserved every tested certificate and metadata byte, and all nine
same-byte Rust/Go checker cases passed. See
`unit-4-collection-clauses-progress.json` and its direct review. Public-domain
and native/application proof obligations remain separate outstanding work.

The test observer cache experiment is tracked in
`unit-3-observer-cache-experiment.json` and its scoped direct review. Core
regression, inventory and lint checks passed. Its weak-argument follow-up is in
`unit-3-observer-weak-cache-experiment.json`: twelve targeted tests pass,
including repeated-function work, eviction and capture-cycle avoidance. Actual
string-domain verification remains outstanding. The earlier closed-definition
cache passed both selected decimal divide conditions in 1,067.06 seconds, with
all nine regenerated certificate/metadata pairs unchanged; that run predates
the weak-argument cache. No workload speed or memory improvement is claimed
from these experiments. This does not change certificate acceptance
or close an original work unit.

`calendar-clauses/` adds native DateOnly, TimeOnly, TimeSpan and Guid contract
connections: eight source contexts, 50 operation aliases, 54 attachments and 14
selected ordinary observations, including four definedness rejections. All 70
standalone Calendar/Temporal pins are preserved. The eight new same-byte checker
cases passed; results are recorded in `unit-4-calendar-clauses-progress.json`. Instant error-outcome
connections and application/native-body proofs remain outstanding.

### String contract adapters (unit 4 component)

`string-clauses/` contains six source contexts for 21 unary/binary string operations
and 67 contract attachments. Full-capacity literals exposed a linear selector
stack issue; large literals now use a balanced address tree. Candidate replay
and scalar/large-literal regressions pass. Both changed construction candidates
pass both unchanged checkers; four identical-byte passing cases are retained.
All 67 source observations pass; see `unit-4-string-clauses-progress.json`. Units 3-8 and
W09 remain open.

### W03 integer/Boolean source operation relations (unit 5 component)

`integer-data/` connects frozen scalar bodies to original W03 result and check
formulas at eight SSA use points. Subject-index conversion and canonical core
names are explicit; other data families remain pending. Candidate replay and
1,068 result-bit observations pass. All three same-byte checker cases pass;
all 80 source result/guard cases pass and final runtime bytes match the checker
pins. See `unit-5-integer-data-progress.json`. No native-body/application
proof or original-unit completion is claimed.

### W03 structural equality use points (unit 5 component)

`structural-data/` connects existing semantic equality definitions to seven
original SSA use points across five native inputs. All 98 result/guard cases
and five same-byte dual-checker cases pass. Exact pending operations and native
!= decomposition are retained. The shared result/point helpers preserve the
prior integer-data pins. No fresh native CanonicalCompare use point is included;
see `unit-5-structural-data-progress.json` for this and the remaining W09 scope.


### Native floating data connection checkpoint

`floating-data/` connects eight floating/conversion definition occurrences and
ten original W03 SSA use points across three C# sources. The seven operations
and scope limits are recorded in `unit-5-floating-data-progress.json`. Existing
scalar bodies and source/control links are preserved; 84 result/guard cases,
three same-byte dual-checker certificates, source replay/mutations, lint,
inventory and formatting pass. Runtime outputs match candidate pins. The scoped
review is in `unit-5-floating-data-review.md`. Decimal and other families remain
pending; this does not complete native-body proofs, original unit 5, or W09.


### Native decimal data connection checkpoint (in progress)

`decimal-data/` connects nine decimal definition occurrences and eleven W03
SSA use points. Result relations observe all 512 storage bits and preserve raw
failure conditions separately from ordered guards. A core Let now shares the
computed result across bit comparisons. Candidate reconstruction, scalar-body
closure matching, comparator/sharing regressions, unchanged six prior
integer/floating pins, final lint, inventory and formatting pass. Final native
runtime and dual checking of shared candidates remain live in
`unit-5-decimal-data-progress.json`; earlier unshared checker passes are
historical evidence. The scoped review is in `unit-5-decimal-data-review.md`.
Original unit 5, units 3-8 and W09 acceptance remain incomplete.
