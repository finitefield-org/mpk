# T02-W01 private contexts and immutable dependency capture

Status: In progress. Entry is T01 completion commit
`fe1b85cb70c7ea390c02469706d09cdc913f973f`. The
[requirements](requirements.json) retain the complete W01 scope: private
contexts, compiler/stdlib/analysis identity, immutable multi-capability capture,
exact module/replacement resolution, archive/Go sums, export metadata and
rejection of escape, stale inputs, network/VCS/generation/toolchain fallback.

The first implementation component is
`go-tools/go2vir/practical_source.go`. It constructs the final
`mpk.go.source_object.v1` complete domain-separated identity using `go_file`
and all seven other frozen source kinds. Original bytes are independently
owned; returned bytes and metadata cannot change the stored buffer. Raw binary
bytes, Unicode path spelling and zero-length buffers retain their exact
identity. Logical paths and the16MiB file/1MiB overlay bounds are checked.
Oversized inputs reject before ownership. Strict import rejects duplicate, extra or missing fields, non-integral
byte counts, Boolean/null confusion, unknown/old kinds and mismatched original
bytes or complete identities.

These source-object records supply no filesystem/capability authorization,
compiler identity, module resolution, selected-body acceptance or application
proof. Those executors remain due in W01 and later T02 work items. Existing
candidate graphs remain at their original epoch; no graph is relabeled or
promoted. The public frontend continues to reject the inactive practical
profile before source reads.

The [selected checks](verification.json) pass with the exact Go1.25.0 Darwin
toolchain: eight root tests and26 subtests, including frozen identity vectors,
arbitrary original bytes/caller alias mutations, metadata forgeries, old-kind
crossing, logical-path and byte boundaries, existing shared canonical/strict
JSON and closed public-profile arguments. `go vet` and changed-file `gofmt`
pass. The first shared-cache setup failure and the successful isolated-cache
configuration are recorded separately. The strict input review also identifies obsolete-kind literals in the initial
negative tests. The final tests directly consume the exact original negative
object referenced by the frozen catalog, with its full fixture hash checked.
The three exact failed diagnostics are preserved through the existing historical
seal policy. All127 rules/four scopes and all previous36 seals remain; twelve
altered-seal controls and all existing fixture checks pass. No production or
directory exemption is added. No historical graph changes. No dependency/toolchain download
or C# test is used. These are component checks, not Linux release acceptance.

No complete W01 or T02 claim is made. Targeted checks cover changed code and
affected consumers; the cumulative T02 gate is deferred to W04 and will use
`./scripts/check-fast.sh --skip-csharp` under the user's explicit instruction.
All169 Go theorem demands remain unelaborated and no practical profile activates.

The [rooted capture checkpoint](verification/rooted-capture/README.md) now adds
actual multi-capability filesystem reads and sealed owned snapshots. It checks
exact names, rooted/nonblocking opens, links/special files, observed namespace/
content changes, aggregate budgets and deterministic identities. The final
Go1.25 selection passes20 root tests/43 subtests; after the native-asset review
fix all8 affected rooted tests/17 subtests plus vet/format pass. The Go1.23
compatibility build separately passes3 root tests/11 subtests and rejects
unavailable rooted capture. Materializing the loader namespace and complete
context/module/compiler/source/export authority remain due; W01 stays open.

The [original module input checkpoint](verification/module-inputs/README.md)
adds exact manifest/version/checksum handling and captured local replacement
mapping. A fresh pinned offline Go module-graph comparison agrees with original
nested-module inputs. The12-root52-subtest selection, vet/format and separate
legacy build pass. Versioned archive/graph/export capture and checked private
context/foundation remain due; this does not complete W01.

The [ordinary Boolean checkpoint](verification/ordinary-boolean/README.md)
adds5 actual definitions and5 universal ordinary laws, exact component import,
and fresh current-checker evidence. Both checkers accept the same zero-axiom
certificate with identical hashes and reject an incorrect ordinary equality
claim. Three targeted Rust integration tests, strict Clippy and format pass;
no C# tests execute. This Boolean basis does not register the complete private
foundation/context or complete W01.

The [original archive checkpoint](verification/module-archives/README.md) now
verifies raw ZIP identity, Go h1, complete namespace/size/CRC rules and source
content linkage through actual captured checksum authority. Shared limits and
private-map reconstruction reject forged/crossed inputs; current offline Go
checksum observations agree on original and tampered files. The review fix and
all targeted checks are recorded. Complete module metadata/graph/context/loader
binding remain due; no W01 completion or profile activation is claimed.

The [versioned module checkpoint](verification/module-snapshots/README.md)
now binds original .mod/.info/.ziphash/.zip inputs and verifies original checksum,
version, root-manifest and legacy no-rewrite consistency. Its9-root34-subtest
selection, current vet/format and separate legacy build pass. Actual selected
module graph, complete context/compiler authority and immutable original-package
source/export loading remain due; W01 is still In progress.

The [actual Go module discovery checkpoint](verification/module-resolution/README.md)
now runs pinned offline Go version/replacement selection and binds preparation
observations to original snapshots, roots, languages and main checksum ownership.
An initial11-root31-subtest selection passes; after a reproduced checksum-owner
omission is fixed, all4 affected binder roots/10 subtests and current legacy,
vet/format checks pass. This initially reads the original mutable namespace;
complete consulted-input capture and immutable authoritative replay remain due,
alongside the complete private context/compiler/foundation. W01 stays In progress.

The [prepared compiler checkpoint](verification/compiler-image/README.md) now
checks a complete original Go1.25.0 image:5,577 files,9 exact command/tool binaries,
original specification/version witnesses and complete component ownership.
Independent original-SDK hashes agree with the retained fresh inventory. Native
image/reauthorized-change controls, the eleven-case build-info matrix, legacy
checks and current vet/format pass; the original fixture-mode failure is retained.
Complete analysis-library/frontend build identity and checked context/registry
binding remain due before immutable production loading; W01 stays In progress.

The [analysis/source build checkpoint](verification/analysis-inputs/README.md)
now binds the exact three original analysis module snapshots and vendored code,
retains169 sealed frontend/vendor build inputs and checks linked binary metadata.
Two independent empty-cache builds and a separately reconstructed third build
produce identical complete binaries. The existing uncompiled TODO byte difference
and initial private-output-alias failure remain explicit. Targeted current/legacy,
compiler-consumer, vet/format and independent original-hash checks pass. Complete
actual ordinary foundation/context/registry binding is the next critical work;
application closure/immutable replay still remain due and W01 stays In progress.

The [actual core/basis checkpoint](verification/core-interfaces/README.md) stages
the two exact frozen core interfaces as an explicit W01 foundation prerequisite;
W03 retains common migrations. All255 frozen design inputs remain unchanged.
The fresh35-declaration zero-axiom ordinary basis proves an arbitrary-Nat legacy
recursion identity and conditional Bool false-elimination. Both current checkers
agree on all18 same-byte cases and exact positive hashes. Core/current/legacy,
strict lint/format and targeted importer tests pass. Full ordinary foundation/
descriptor/context and application closure/replay remain due; W01 stays In progress.

The [actual unbounded Nat checkpoint](verification/natural-foundation/README.md)
adds eight ordinary operations and45 checked laws, with88 total declarations
and zero axioms. Both current independent checkers agree on all15 original byte
streams: five acceptances with exact hashes and ten core type mismatches.
The original insufficient reflexivity-only computation recipe is reproduced;
explicit checked computation laws prove all20 correct observations without a
kernel change. Five scoped Rust integration roots, strict Clippy and format
pass. All255 frozen inputs and the preceding core basis remain unchanged.
Balanced integers/ranges, complete representations/operations/source laws and
full descriptor/private context binding remain due; W01 stays In progress.

The [actual balanced integer checkpoint](verification/integer-foundation/README.md)
adds13 ordinary definitions,42 library laws and566 checked Nat-equality
checkpoints. Both current checkers agree on15 same-byte cases with zero-axiom
positive hashes and eleven core type rejections. Integer equality is the
cross-sum relation on full unbounded Nat pairs, and arithmetic/observations
respect redundant representatives. Four targeted integration roots, strict
Clippy and format pass. The initial insufficient embedding-injection proof and
every original input/diagnostic remain; its initial negative passes are not
counted. Half-open folds, full representations/operations/source foundation and
descriptor/private context binding remain due; W01 stays In progress.

The [actual half-open fold checkpoint](verification/range-foundation/README.md)
adds eight ordinary definitions and31 laws. Bounded/interval preservation needs
only equality over the actual visited prefix, including semantic equivalence
for integer sums. Both current checkers agree on17 same-byte cases. Five scoped
integration roots and a focused stronger typed-control root pass, as do strict
Clippy and format. Original insufficient observation recipes, observer failures
and the replaced weaker control remain recorded. Full carrier/source/operation/
definedness foundation and descriptor/private context binding remain due; W01
stays In progress and the T02 gate stays deferred to W04.

The [actual full-width word checkpoint](verification/word-foundation/README.md)
adds1000 ordinary circuit definitions and144 basic universal laws for all four
widths. Exact current checkers agree on20 final candidates with zero-axiom
positive hashes and17 core type rejections. Actual decoded-certificate terms,
independent finite integer arithmetic and fresh pinned native Go agree on all1640
inputs, including forty runtime-domain failures. Corrected scoped Rust tests,
strict Clippy and format pass; initial helper-name/native-preparation failures
remain. Full Nat/Int interpretation/conservation, source-operation/nominal/state/
program binding and descriptor/private context remain due; W01 stays In progress.

The [actual word/numeric checkpoint](verification/word-numeric/README.md)
adds balanced unbounded Nat values/shared weights, pointwise congruence, actual
ripple addition conservation and conditional balanced-Int addition for all four
widths. Both current checkers agree on26 unchanged candidates: three zero-axiom
acceptances and23 core type rejections, including all four dropped-final-carry
claims. Four scoped Rust roots, strict Clippy and format pass. Actual Boolean
circuits, exact integer arithmetic and fresh pinned native Go agree on288
numeric queries, with210 signed no-overflow cases and78 overflow cases. The
original comparison-consumer subtype assertion and focused reproduction remain;
no proof/checker bytes or limits change to repair it. Complete preceding word
bytes and all255 frozen inputs remain unchanged. Remaining numeric/source/
carrier/program laws and descriptor/private context binding remain due; W01
stays In progress and the cumulative T02 gate stays deferred to W04.

The [actual fixed-shift/multiplication checkpoint](verification/word-arithmetic/README.md)
adds genuine unbounded PowerTwo laws, actual discarded-bit/fixed-left-shift
conservation, multiplier-prefix interpretation and complete actual Mul0..width
conservation with a computed Nat witness. Both unchanged checkers agree on26
final streams, with three zero-axiom acceptances and23 core type rejections.
Actual48-input Boolean-circuit evaluation, fresh pinned native Go and exact
integer arithmetic agree on1440 stages and107 carries. Original construction/
conversion/diagnostic/control failures remain; the focused five-case correction
retains eighteen unchanged typed rejections and all passing production/model
inputs. Scoped current Clippy and format pass. Every frozen artifact bound and
all255 design inputs remain unchanged. Remaining numeric/source/carrier/program
laws and descriptor/private context binding remain due; W01 stays In progress
and the cumulative T02 gate stays deferred to W04.

The [actual word bounds/order/guard checkpoint](verification/word-order/README.md)
adds proved complement/value bounds, actual subtraction conservation, unsigned
comparison and equality correspondence, zero values and actual divisor-guard
positivity for all four widths. Both unchanged checkers agree on31 exact
streams, with three zero-axiom acceptances and28 core type rejections. Four
targeted roots, strict Clippy and format pass. Actual Bool circuits, fresh
pinned native Go and independent integers agree on272 original queries;
additional actual carry/zero/guard bits agree with host meanings. The original
large-tree conversion limit and helper compilation error remain with complete
sources/raw diagnostics; explicit small block folds fix conversion without
changing checker fuel or artifact bounds. Prior numeric/arithmetic bytes and
all255 frozen inputs remain exact. Actual division/remainder and the remaining
foundation/source/context/application work remain due; W01 stays In progress
and the cumulative T02 gate stays deferred to W04.

The [actual unsigned division checkpoint](verification/word-division/README.md)
now proves all120 restoring circuit stages, MSB-prefix interpretation and final
Euclidean conservation/strict remainder bounds for all four widths under the
actual nonzero-divisor guard. Both unchanged cores agree on24 current streams:
two zero-axiom positives and22 core type rejections. Five affected roots, strict
Clippy and Rust/Go format pass. All2198 retained prerequisite declarations match
the exact prior named types/bodies/proofs/metadata. The initial result-bound
mistake, nested-conversion limit and helper-name collision remain with complete
sources/candidates/raw evidence; explicit folds and the established prerequisite
generation order fix them without changing bounds/fuel. Actual Bool circuits,
fresh pinned native Go and independent exact-prefix integers agree on96 queries,
2640 stages, eight zero-divisor failures and34 arbitrary-remainder steps including
ten high bits. Remaining signed/numeric/source/foundation/context/application
work stays due; W01 remains In progress and the cumulative T02 gate stays W04.

The [actual signed multiplication checkpoint](verification/word-signed-product/README.md)
adds an unconditionally computed balanced-Int modular correction for the actual
word product, plus both directions of its exact-result guard and agreement with
actual mathematical Int equality. All four widths include minimum times minus
one without excluding Go wrapping. Both unchanged cores agree on33 streams:
two zero-axiom positives and31 core type rejections. Four scoped roots, strict
Clippy and Rust/Go format pass; all2387 retained named prerequisites and complete
prior arithmetic/unsigned-division byte hashes remain exact. Actual48-input Bool
circuits, two identical fresh pinned native Go outputs and independent integers
agree on1440 stages,21 exact products and27 wrapped products. Nat guard/witness
bodies are checked universally in both cores rather than executed by the finite
Bool model. Signed division and the remaining foundation/source/context work
stay due; W01 remains In progress and the cumulative T02 gate stays W04.

The [signed negation/magnitude checkpoint](verification/word-signed-magnitude/README.md)
now proves actual complement-plus-one carry conservation, actual Neg/AbsBits
zero/divisor-guard preservation, actual signed magnitude correspondence and
computed signed-negation correction with both exact-result guard directions.
Its2163 declarations/49,380 terms/666,154 bytes contain zero axioms and all1795
retained named prerequisites match established arithmetic/order/signed-product
references. Four current affected roots, strict Clippy/Rust/Go format and45
same-byte paired streams pass, including43 actual core type rejections.
All32 original Neg inputs,960 actual stages, four minimum wraps and four zeros
agree with two identical fresh native Go outputs and independent integers.
Original failed construction/observer records remain separately qualified;
no bound, core rule, axiom, fuel, frozen input or public activation changes.
Actual magnitude half-range/signed division and complete foundation/context/
source replay remain due. W01 is In progress and the full T02 gate stays W04.

The [signed half-bound checkpoint](verification/word-signed-bounds/README.md)
now proves balanced half/lower-complement/top definitions, actual unsigned
value decomposition, computed magnitude gap and unconditional half bound,
strict nonnegative bounds and actual threshold/sign/minimum correspondence.
Its2683 declarations/58,000 terms/818,605 bytes have zero axioms and preserve
all2163 prior named declarations plus exact complete signed-magnitude bytes.
Four affected roots, strict Clippy/Rust/Go format and60 same-byte paired
streams pass, including58 actual core type rejections. All32 original inputs
agree on96 actual source/Neg/AbsBits words and96 new Top applications with
two identical fresh native Go outputs and independent integer counterparts.
Four minima keep the magnitude high bit; no source input is excluded.
Original failed compilation/source/raw diagnostics remain. Signed division/
remainder and complete foundation/context/source replay remain due; W01 is
In progress and the cumulative T02 gate stays deferred to W04.

The [direct-zero checkpoint](verification/word-zero/README.md) now proves actual
zero/nonzero/sign laws with 274 declarations and all 209 original named
prerequisites exact. The [compact signed-division prerequisite checkpoint](verification/word-signed-division/README.md)
combines all required unsigned division, signed magnitude/half/minimum and actual
signed operator definitions in 3,483 declarations/85,883 terms/1,255,288 bytes,
with zero axioms. The original closed union needed 4,173 declarations; 12 exact
same-type proved zero substitutions leave 613 slots under the unchanged limit.
Every mapped type/body/proof/level/family matches its original reference, and
complete original generators retain their bytes. Each component has four
affected roots, strict Clippy/Rust format and a same-byte paired matrix:
43 zero streams with 41 type rejections and 33 prerequisite streams with 30.
All 128 original zero/signed rows agree with two identical fresh native Go runs
and independent integers, including eight zero-divisor faults and eight
minimum/-one observations. The accepted altered SignedDiv definition explicitly
shows that the prerequisite component does not yet prove signed division.
Actual signed quotient/remainder laws and all remaining foundation/context/source
replay work stay due. W01 is In progress; the cumulative T02 gate stays W04.

The [actual signed-refinement checkpoint](verification/word-signed-refinement/README.md)
now proves actual SignedDiv/SignedRem values against the mathematically signed
actual magnitude quotient/remainder with ordinary computed modular corrections.
The actual nonzero divisor guard yields both magnitude and signed mathematical
reconstruction and the actual unsigned remainder bound. Explicit source operator
definition theorems bind those results; replacing either signed operator by zero
now rejects at every width. All 3,483 prior named prerequisites and 166 additional
original Int proof dependencies keep exact metadata. The 3,897-declaration/
94,497-term/1,390,896-byte certificate has zero axioms and fits unchanged limits.
Four affected roots, strict selected lint/format, 62 same-byte paired streams
with 60 actual type rejections and 160 original-input native/model/integer
observations pass. Original computed-Xor proof failure/source/raw diagnostics
remain; explicit Bool-to-Nat equality transport repairs the recipe. Universal
quotient/half bounds and zero/minimum correction classification remain due,
alongside the full remaining source/foundation/context/application work.
W01 is In progress and the cumulative T02 gate stays deferred to W04.

The [actual signed-division bound checkpoint](verification/word-signed-division-bounds/README.md)
now proves these previously outstanding range and classification laws. The
actual magnitude quotient is at most half the modulus and the remainder is
strictly below half. The actual remainder correction is zero. The actual
quotient correction is exactly the bit of the actual minimum-dividend/
negative-unit-divisor predicate, whose divisor condition is proved equivalent
in both directions to signed -1. This gives classified source reconstruction
and exact reconstruction under a separate optional no-overflow guard. The
minimum/-one operation stays source-defined, with actual quotient word value H
and signed remainder zero.

Its 4,093 declarations/99,499 terms/1,465,556 bytes have zero axioms and fit all
unchanged limits, leaving three declaration slots. All 3,897 prior named
declarations and complete old generator bytes remain exact; eight additional
original half proofs and 188 new declarations provide the new laws. Four
affected roots, strict selected Rust/Go lint/format, 70 same-byte streams with
68 actual type rejections and 96 original full-input native/model/integer
comparisons pass. Both actual source results execute for 88 defined pairs;
eight source faults and eight minimum/-one rows stay present. The finite Bool
model does not execute Nat/Int/predicate/theorem bodies. Both unchanged cores
check the universal proofs separately.

The original native wrapper stopped after successful build and two identical
runs because of a copied expected unique-ID count. Exact original source/raw
outputs and the observed diagnostic remain. Metadata recovery corrects only
128 to 96, preserves unavailable elapsed times as null and qualifies original
control-flow evidence; a new complete prepared-image recheck supplies its own
timestamp. Native build/observations are not repeated. Remaining numeric/source/
carrier/state/program/bounded 56-family/context/registry/application replay work
and all 169 application demands remain open. W01 stays In progress, C# tests
stay skipped and the cumulative T02 gate stays deferred to W04.

The [actual conversion checkpoint](verification/word-conversions/README.md)
now proves all32 signed/unsigned conversions among8/16/32/64-bit words. Actual
lower/higher cube values compute the truncation quotient and exact radix ratio;
computed extension masks give exact zero/sign extension balances. Both target
integer interpretations have actual-result modular refinements, and signed/
unsigned widening preserves its respective mathematical value exactly. Actual
SignedTo64 count guards identify the source sign and provide exact nonnegative
full64 count values; unsigned64 high-bit counts remain source-defined.

Its1,882 declarations/31,495 terms/483,095 bytes have zero axioms under unchanged
limits. All888 retained original named identities and complete old arithmetic/
order raw bytes remain exact;994 declarations are new. Four affected roots,
strict selected lint/format and172 same-byte streams pass, including170 actual
type rejections. All256 original full conversion rows, plus their actual To64/
count guards, agree with fresh typed Go/math-big and independent Python integers.
There are64 real negative-count panics and192 defined counts. The Bool model
executes actual circuits with unchanged ceilings; Nat/Int/quotient/ratio/mask/
theorem and variable-shift bodies do not execute there. Exactness flags identify
type-level theorem applicability, not an iff over finite observations.

Original compiler/generator/fuel/type failures, complete failed candidates and
raw/direct prefix diagnostics remain. Shared scalar proofs resolve declaration
overflow without dropping any converter; small cube/definition/top equalities
mapped through Value/Nat/Not resolve computed-major proof issues. The helper is
ignored from creation. No core/fuel/axiom/limit/frozen-input change occurs.
Variable/signed/right-shift numeric/source laws and all full foundation/context/
registry/source/application work stay due. All169 application demands remain
unelaborated, W01 is In progress, C# tests stay skipped and the cumulative T02
gate stays deferred to W04.

The [fixed-right and full64 count checkpoint](verification/word-shifts/README.md)
adds ordinary power/PopHigh proofs, all18 unsigned/signed fixed-right amount
bindings with computed low-block remainders/bounds, and balanced high-count
arithmetic/zero reflection. The component has3105 declarations,77415 terms,
1100109 bytes and zero axioms, within unchanged caps. It retains2669 exact
original named declarations (the2665 planned root dependencies plus four
existing Builder seed definitions), including all1882 conversion declarations.
Original converter and signed-division composer raw pins remain exact.

Both unchanged cores inspect103 complete same-byte streams: two accepted
positives and101 genuine type rejections. All504 original full shift rows,
actual source stages/count guards and4212 fixed-right observations match typed
Go and Python integers; there are24 real negative-count panics,480 defined
counts and309 defined high counts. The combined scoped run completes its
control/model roots, then exposes the omitted four seed definitions in its
separate count assertion. Only that assertion/report count is corrected and
the affected root rerun. Raw failures/sources and the original observer subtype
assumption are retained; no source/JSON/directory exclusions or core/fuel/axiom/
limit changes occur. The low-prefix bound and actual all3 variable-shift stage/
saturation/signed-Int laws remain due. W01/T02-T11 and all169 application demands
stay active; C# tests stay skipped and the cumulative T02 gate stays W04.

The [full-count variable-shift checkpoint](verification/word-shift-count-bounds/README.md)
now proves actual source left/unsigned-right/signed-right stages and final
zero/sign fill for all unsigned64-bit counts, actual discard/remainder witnesses
and strict bounds. Actual signed left wrapping correction and signed right floor
Int conservation are proved through the established real integer projections.
The4053-declaration/101285-term/1454895-byte zero-axiom component fits unchanged
caps/fuel. All3105 previous metadata identities and the old full conversion raw
pin remain exact. Both unchanged cores inspect85 complete same-byte streams,
including83 genuine source/arithmetic/premise/interpretation type rejections.
The finite counterparts reuse504 exact original Bool/native rows and verify
2586 stage observations with full counts; no huge powers or new theorem bodies
execute. Original pointwise-selection type failure, ignored diagnostic path
failure and private wrapper compile error remain with exact source/raw records.
No checker/axiom/fuel/cap/frozen-input changes occur. Full source/program/
foundation/context/registry and all169 application demands remain due; W01 and
T02-T11 remain active, C# tests stay skipped and the cumulative gate stays W04.

The [sealed original namespace checkpoint](verification/sealed-namespace/README.md)
retains complete visited native directory names/types and independent snapshot
allocations. Read-only capability/path/role lookup distinguishes observed
absence from present-but-unread and unvisited inputs, without original filesystem
access or fallback. The original collector/snapshot byte alias is reproduced
and retained; seal now owns fresh bytes and namespace arrays. All34 affected Go
roots/101 subtests, seven legacy roots/eleven subtests, current vet/format and17
independent complete hash preimages pass. Two fresh-cache frontend builds from170
sealed inputs agree byte-for-byte. All255 frozen inputs and the preceding shift
certificate remain exact. Current requirements now point to these actual capture
and full64 shift checkpoints while preserving every original requirement. Full
foundation/private context and authoritative original application source/export/
selected-closure materialization/replay remain due; all169 demands and T02-T11
stay active. C# tests stay skipped and the cumulative T02 gate stays W04.

The [actual signed-order checkpoint](verification/word-signed-order/README.md)
proves all8/16/32/64-bit source signed Less/LessEqual circuits equal the actual
balanced unbounded Int comparisons. The3457-declaration/71954-term/996816-byte
zero-axiom component retains every3437 original order identity and rechecks
the genuine strict-gap dependency closure. All three affected Rust roots,
selected Clippy/format/ignored-helper listing and29 same-byte paired streams
pass, including27 genuine type rejections for altered source operations/sign
addresses, claims and real bound premises. All96 original signed comparisons
match retained actual Bool/native results; every1640 original result and40 real
native runtime failures remain checked. Initial computed-major proof failure,
direct diagnosis and all observer schema/format failures retain full inputs/raw
output; corrected observers reuse completed native/core streams. All255 frozen
members and preceding complete order/shift bytes remain exact. Full foundation/
source/program/context and all169 demands/T02-T11 remain open; C# tests stay
skipped and the cumulative T02 gate stays W04.

The [actual signed-equality checkpoint](verification/word-signed-equality/README.md)
connects all8/16/32/64-bit source Equal/NotEqual to actual balanced unbounded
Int.Equal and proves both Int.Equivalent directions. The real balanced all-bit
equality tree yields actual top-sign coherence in3/4/5/6 checked steps, and
EqualNatural discharges the scalar coherence callback. All3457 previous signed-
order identities and its full raw bytes remain exact. The3489-declaration/
72690-term/1007869-byte zero-axiom component fits all unchanged caps/fuel.
Three affected Rust roots, selected Clippy/format and46 same-byte paired streams
pass, including44 genuine type rejections (38 check/six lambda). Fresh typed Go
signed/unsigned equality matches all96 original full queries and old decoded
Bool outcomes; every1640 original result and40 native runtime failures remain
checked. The new observer builds from an empty cache, produces identical output
twice and passes vet/format on the complete verified5577-file Go1.25 image.
All255 frozen members remain exact. Full56-family/source/program/context and
authoritative immutable application replay/all169 demands/T02-T11 remain due;
W01 remains In progress. C# tests stay skipped and the cumulative gate stays W04.

The [archive return-ownership checkpoint](verification/archive-ownership/README.md)
reproduces and fixes first-read/cache-hit aliasing of verified member maps and
original raw ZIP bytes. Returned values and the session cache now own separate
complete buffers/maps. All four original regressions and full original inputs
remain recorded. The14 affected Go roots/45 subtests and current vet/format pass;
Go1.23 performs compile/link only. Two actual fresh-cache frontend builds from170
sealed inputs agree byte-for-byte. All255 frozen inputs remain exact. All related
actual handles are terminal; the final receipts preserve their initial epochs
and source/result bindings for strict changed-input and Git blob audits. Full
context/immutable application replay and all169 demands remain due.

The [ordinary heterogeneous-product checkpoint](verification/product-foundation/README.md)
adds generic construction, projections, pointwise observations, updates, maps
and typed validity preservation. The complete78-declaration zero-axiom basis
passes both cores; all50 new actual semantic negatives and exact ownership pass
their selected checks. The expanded prototype stops at the fixed producer term
guard at arity60 after0..59. The complete0..64-field type-sharing matrix passes
Rust with93600 leaf obligations; an independent comparison preserves all2210
original goal types. The expanded Go observer finishes all112 generated inputs
and records the five over-budget original shapes as unavailable. The complete
shared Go matrix now passes all65 inputs with matching full hashes. The
consolidated three boundary roots and Clippy/format pass;52 complete current
inputs reuse exact prior Go executions.
The ordinary value laws grant no source operation,
Go evaluation-order, invocation, storage or program correspondence authority.
W01 remains In progress; C# tests stay skipped and the T02 gate stays W04.

The [full-original assembly capacity observation](verification/product-foundation/foundation-assembly-analysis/README.md)
retains all23 mathematical component inputs and their12249 distinct names, with
no same-name type/body/proof/level/family conflicts. The complete full64 shift
and product combination actually passes both kernels at exactly4096 declarations,
101705 terms and1428859 bytes, preserving every original node modulo exact
sharing. A valid alternate named proof passes both kernels but strict assembly
rejects its changed identity; a false full-composition claim rejects in both.
This diagnostic leaves zero declaration slots and does not complete production
root derivation/full56-family source/private context or any of the169 demands.
The complete library exceeds one artifact's fixed bounds; full original inventory
and compiler-derived complete VC dependency closure remain separate requirements.
The expanded observer is terminal; the main three-root wrapper is consolidated
without changing any source captured by the shared observer. The shared Go
matrix now passes every input. The final product receipt binds120 current
same-byte streams across product/consumer/assembly checks:69 zero-axiom positives
and51 genuine type mismatches. All actual handles are terminal. Strict
changed-input and staged/committed-blob audits cover the final current tree. C# tests stay skipped and the T02 gate stays W04.


The [closed original syntax checkpoint](verification/source-syntax/README.md)
adds private parsing from sealed original go_file buffers and physical source-ID/
byte origins, retaining complete original declarations and unsupported bodies.
CR-bearing raw strings/comments, forged line directives, all materialized
comments/nodes and capability/path ambiguity are checked without source stubs
or filesystem fallback. Twelve affected roots/99 subtests, current vet/format,
Go1.23 compile-only and two actual fresh-cache171-input frontend builds pass.
All81 original source cases,2090 complete AST nodes and1710 original native
function-node obligations are preserved; syntax validity is distinguished from
three original Go type errors. Original failures/full source epochs remain, including the actual returned-source
cache alias regression and previous11-root receipt. First/cache source views
now own separate original byte storage; AST remains untrusted compiler data.
These compiler locators grant no type/profile/source-proof authority. Original
complete package build/file selection/type checking/export/compiler binding and
immutable application/selected-closure replay remain due; R06 and W01 stay open.


The [sealed source package checkpoint](verification/source-packages/README.md)
adds fixed-target Go build/file selection and complete source-only Go typing
from original captured module roots/verified versioned archives. All81 original
sources retain77 typed/3 type-error/1 syntax-error outcomes. Relative/absolute
replacements, nominal/alias identity, unused-body errors, archive/source linkage
and closed import failures are checked without original-source/mod rewriting
or typed-package adapters. All24 affected roots/201 subtests, current vet/format,
Go1.23 compile-only and two173-input fresh frontend builds pass. Original failed
source epochs and fresh actual native diagnostics remain explicit. Missing
stdlib/export input stays not captured. Authoritative immutable native graph/
package/export replay and complete original compiler/stdlib/context/foundation
ownership remain due; R06/W01 stay open and no profile/source proof activates.

The [original SDK source/export checkpoint](verification/standard-sources/README.md)
captures the complete original prepared Go source inventory and checks178
whole application/SDK packages through sealed vendor/internal imports. All179
native packages and178 complete79390348-byte compiler archives retain original
file/import/compiled/root/build-ID correspondence. The full1472407-node source
image and an unselected HTTP handler survive without filesystem fallback. All51
affected roots/325 subtests, current vet/format and Go1.23 compile-only pass;
two fresh175-input frontend builds agree. Every available original byte epoch,
including binary assets/archive buffers, remains losslessly serialized and
independently scanned. Three export failures retain precise limitations: original
archive payloads from the first two were not exposed before cleanup and remain
unavailable; later fresh observations are not substituted. All255 frozen members
remain exact. Full original export/compiler ownership, authoritative immutable
native graph/package/export/selected-closure replay and the full foundation/
private context/all169 demands/T02-T11 remain due. R06/W01 stay open; C# tests
stay skipped and the cumulative T02 gate stays W04.

The [original byte-image/native replay checkpoint](verification/replay-image/README.md)
preserves original absolute/relative/versioned replacement paths and full
source bodies in owned images, with complete namespace/not-captured records,
shared frozen budgets and the full5577-file compiler copy. Native export
preparation uses the same original offline module cache. Two fresh offline
kernel-readonly executions reproduce all4 module objects/all182 package
objects except only the derived export locator and all181 original archives/
79406918 bytes. The unselected HTTP handler/panic initializer remain original
and are not executed. Actual readonly-input/hidden-SDK/missing-source controls
reject. Local Linux/amd64 runs are x86-64-emulated on Linux/aarch64; they grant
no native Linux release claim. All22 affected roots/177 subtests, current vet/
format, Go1.23 compile-only and two176-input frontend builds pass. Independent
audits preserve11334 source preimages, the whole copied compiler/image bytes
and all255 frozen members. Complete production kernel/context/consulted namespace/
selected-closure bindings/full56-all6-19 source/program foundation/private context/
all169 demands/T02-T11 remain due. R06/W01 stay open; C# tests stay skipped and
the T02 gate stays W04.


The [original namespace/owned view checkpoint](verification/namespace-view/README.md)
rejects contradictory complete directory views across original root aliases,
including native type, role, bytes and observed-parent disagreements. The
actual initial failure and both original inputs remain byte-exact. Compatible
aliases and unvisited nested refinement preserve separate source identities.
Owned rooted reader transactions validate the complete5590-file image against
6441 reconciled original namespace nodes; failures stay sticky and live views
retain image lifetime. Rooted cleanup preserves outside link-target bytes/mode.
All30 affected roots/192 subtests, current vet/format, Go1.23 compile-only and
two180-input fresh frontend builds pass. A real Linux producer validates the
full5577-file compiler and original whole source image; two independent kernel
replays reproduce all4 original modules/all182 packages except derived export
locator/all181 complete79406918-byte archives. Independent audits check22668
source preimages,131 additional complete payloads, every materialized file and
all255 frozen members. All6 Linux special-permission controls pass after reproducing the original
failures. Four unmaterialized Darwin controls retain explicit backing limits.
One test-only formatter alignment preserves exact production bytes; both
affected roots/current vet/format pass after correction. Prior byte-identical kernel controls remain preserved
without repetition. R06/W01/private context and complete general consulted
namespace/selected-closure/source-program foundation/all169/T02-T11 remain due.
C# tests stay skipped and the cumulative T02 gate stays W04.


The [restored original image consumer checkpoint](verification/image-consumer/README.md)
restores all5674 original source/export objects through the verified rooted
view, preserving separate source/export snapshot digests and all original
namespaces. Source and original compiler export caches share frozen budgets.
The actual Linux producer removes every original fixture application/module/SDK
root and clears donor snapshot buffers before rebinding all4 modules, typing
all181 complete packages/1472495 nodes and binding all181 full original archives.
Collection and immutable image consumption use the same complete module rules;
no filesystem/process fallback supplies consumer data. Actual checksum-source
and parsed-map cache aliases in both consumers are reproduced and fixed by
independent returned copies. All original failures, raw inputs and precise
initial unexposed-observation limitations remain. All59 affected roots/290
subtests, current vet/format, Go1.23 compile-only and two182-input fresh frontend
builds pass; four Darwin special-mode cases retain explicit backing limits.
Two fresh readonly kernel replays reproduce every original module/package
object except derived export locator and all181 complete79406936-byte archives.
Independent reconstruction checks22668 source preimages/327 additional complete
payloads, the full5771-file image/7411 original namespace nodes, both snapshot
frames and all255 frozen members. R06/W01/private context/general consulted
namespace/selected-closure/full56-all6-19 foundation/all169/T02-T11 remain due.
C# tests stay skipped and the cumulative T02 gate stays W04.
