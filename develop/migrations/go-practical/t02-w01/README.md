# T02-W01 private contexts and immutable dependency capture

Current scope: [Go and Rust only](../../../specs/LANGUAGE_SUPPORT_SCOPE_V1.md).
Earlier checkpoint text below records its original epoch. Removed-language
work and historical gate flags do not apply to current execution. The current
cumulative T02 gate is final W04: `./scripts/check-fast.sh`, with no
language-specific skip option. Release gates are the root README's local
Go/Rust Linux gates.

The [ordered capture session](verification/capture-session/README.md) now
owns the two held root sets through their complete export-owner binding. Its
full original Go capture and affected Linux tests pass; launcher authority,
private context and source/program proofs remain open.

The [request selection boundary](verification/request-selection/README.md)
validates and hashes the frozen raw Go function/method selectors. It does not
activate the profile or replace later original-source and alias resolution.

The [private candidate context](verification/private-context/README.md) binds
the frozen Go/Rust configuration and delegates retained profiles to the actual
revision-4 validators. Its foundation reference is still a candidate, not a
checked complete foundation or installed profile.

The [Go candidate producer](verification/go-context-producer/README.md) now
emits the same complete request bytes consumed by Rust. The producer alone
does not supply launcher or source authority.

The [request/source binding](verification/request-source-binding/README.md)
now resolves that request against the original physical declarations while
both source and export roots remain held. The complete live application and
SDK case passes, including the test-only compiler owner accepting the selected
function and rejecting a different original function. Launcher authority and
source/program proofs remain open.

The [Go source-selection handoff](verification/go-source-selection/README.md)
derives those physical roots from the Go candidate request and retains the
matching original compiler inputs. Its complete live capture also passes the
independent Rust check; this remains a private candidate path.

The [candidate-bound module graph](verification/candidate-module-graph/README.md)
now retains the same Go package owner and independently reconstructs its offline
module data and graph in Rust while the candidate and both root sets are held.
Launcher authority and source/program proofs remain open.

The [same-root value components](verification/rooted-value-components/README.md)
derive checked entry, call and value mathematics from one held owner. Complete
call-free, error-contract and annotated-call captures pass.

The [checked error value relations](verification/checked-error-value-relations/README.md)
retain the original contract and graph while checking nil and every concrete
error row. The full live error-contract application passes; the 70-case
regression remains in progress. Source execution and complete foundation
remain open.

The [error-free value relations](verification/error-free-value-relations/README.md)
cover complete original source domains without inventing an `error` value.
The private CLI checks the complete graph-bound certificate; live-root
authority, source execution and the complete foundation remain open.

The [complete source-overlay capture](verification/source-overlay-live/README.md)
retains an actual result/input postcondition with the full application and
SDK. The current entry and call certificates do not prove that postcondition.

The [complete cross-package capture](verification/cross-package-live/README.md)
also checks an actual call into the original Go 1.22 dependency while preserving
the whole application and SDK. Both checkers accept its entry and call-interface
equations; application and callee execution proofs remain open.

The [same-root checked components](verification/rooted-checked-components/README.md)
construct entry and call mathematics from one held original owner and reject
crossed graphs or functions. They do not establish launcher authority or a
program proof.

The [checked candidate call interfaces](verification/checked-candidate-invocations/README.md)
check every original call and retain compact hashes for exact reconstruction.
They establish mathematical interfaces, while source execution and callee
proofs remain open.

The [candidate-bound call interfaces](verification/candidate-invocations/README.md)
retain the original contracts and graph, including types referenced only by
annotations. The complete affected invocation regression passes, including the
70-error case; callee execution and program proof remain open.

The [checked candidate entry parts](verification/checked-candidate-entries/README.md)
cover all original contract bodies and require zero-axiom Rust kernel acceptance
for every completed part. A fresh Go recheck also passed for both wide parts;
source execution and the complete foundation remain separate obligations.

The [candidate-bound entry equations](verification/candidate-entries/README.md)
retain the original graph and contract owner while deriving the entry catalogue.
Both checkers accept the same live entry-equation bytes; source execution and
application proof remain separate obligations.

The [candidate-bound contracts](verification/candidate-contract-language/README.md)
retain the original candidate and graph while deriving source contracts and
checking selected and transitive Go language versions. This constructs
candidates only; application proofs and the checked foundation remain open.

The [root-held graph binding](verification/rooted-candidate-graph/README.md)
keeps the full candidate selection and verified graph together. A complete
different source/export owner with the same function name is rejected while
the original root handles remain held.

Status: In progress. Entry is T01 completion commit
`fe1b85cb70c7ea390c02469706d09cdc913f973f`. The
[requirements](requirements.json) retain the complete W01 scope: private
contexts, compiler/stdlib/analysis identity, immutable multi-capability capture,
exact module/replacement resolution, archive/Go sums, export metadata and
rejection of escape, stale inputs, network/VCS/generation/toolchain fallback.

The [installed compiler checkpoint](verification/installed-compiler/README.md)
is in progress. Its private owner derives the registered Go SDK from the
build-pinned installed successor release and binds original compiler/export
inputs to the selected bytes. The complete Linux SDK and 182-package source
owner pass both the test-only sealed-snapshot check and an actual four-bundle
installed-root check under local Linux user-mode emulation. Wrong-host,
crossed-Darwin, missing-namespace, changed-SDK and alternate-registry controls
reject. This does not establish native Linux release acceptance, private context,
source/program proofs or W01 completion.

The [analysis build-info checkpoint](verification/analysis-build-info/README.md)
adds independent Rust validation of the actual registered Go frontend's host,
compiler, module dependencies and settings before installed SDK binding. Go and
Rust agree on all 38 native cases, including the exact registered executable;
affected compiler/ELF/SDK and CLI tests, lint and formats pass. The registered
source epoch and the complete current prelaunch source tree differ, so this does
not prove that the binary was built from current sources. Updated four-bundle
installed-root checks pass under local Linux user-mode emulation; native Linux
release acceptance, context and source/program proof remain due.

The [analysis vendor checkpoint](verification/analysis-vendor/README.md)
independently derives the complete original 29-package set from the owned
manifest. Go and Rust agree on all 36 cases, including exact module ownership,
import-path edges and the unchanged 32,768-package limit; the complete original
manifest remains in the maximum-size control. The new Rust test, eight affected
source/type-key tests, scoped lint and formats pass. Actual analysis archive/
source-to-binary authority, context and source/program proofs remain due.

The [analysis archive checkpoint](verification/analysis-archive/README.md)
checks all 1,751 original ZIP members and their correspondence with 133 supplied
vendor files, retaining all 213 original captured sources and the original TODO
difference. Go/Rust agree on five full-tree controls and a separate changed-ZIP
control. Omitting one of several
files from a package can still pass, so this is not namespace completeness or
source/binary authority. Affected module-data and vendor tests, scoped lint and
formats pass; W01 and the practical profile remain open/inactive.

The [analysis namespace checkpoint](verification/analysis-namespace/README.md)
derives vendor files from every recorded entry in the source snapshot and binds
the manifest and archives to that same snapshot. Go and Rust agree on six
observations, including rejection of unread files/directories and acceptance
of a coherently pruned snapshot. Completeness is relative to the supplied
snapshot; the original root still requires independent authentication. Affected
tests, scoped lint and formats pass, with W01 and the profile still open/inactive.

The [analysis budget checkpoint](verification/analysis-budget/README.md) shares
the existing 131,072-entry limit between recorded directories and the three
analysis ZIP namespaces. Go and Rust accept the exact limit and reject one over;
all original source objects remain. This does not authenticate the root or
account for other archives in a larger capture.

The [analysis module checkpoint](verification/analysis-module/README.md) binds
the frontend manifest and sums, cached `.mod` files and ZIP hashes to the same
observed snapshot and three original ZIPs. Go and Rust agree on 20 cases while
retaining the full original source set. At this checkpoint, cached `.info`,
original-root and source-to-binary authority were still due.

The [analysis info checkpoint](verification/analysis-info/README.md) adds
same-snapshot `.info` version and metadata validation for all three archives.
Go and Rust agree on 36 original/adversarial metadata observations and five
owner controls. Root/VCS authority and source-to-binary proof remain due.

The [general module info checkpoint](verification/module-info/README.md) uses
the same metadata validator for captured application dependencies. Fourteen
Go/Rust cases retain the complete original versioned application/SDK owner and
one Root contract; the existing local and versioned owners and module-data
controls also pass. Capture authority and program proofs remain due.

The [analysis source checkpoint](verification/analysis-source/README.md)
independently parses all 65 frontend and 126 vendor Go sources from the
observed snapshot and checks unmodeled native inputs and embed comments. Go
and Rust agree on 20 original/adversarial cases. Root authentication,
source-to-binary binding and application proofs remain due.

The [analysis build-list checkpoint](verification/analysis-build-list/README.md)
derives all 201 original build inputs, with Go, Rust and the actual builds in
agreement. Two independent offline builds yield identical binaries, but they
differ from the existing registered frontend. Root and registered binary
authority, context and program proofs remain due.

The [held-root comparison](verification/source-root-matching/README.md)
independently checks all original analysis snapshot bytes and native directory
entries against retained Linux directory handles in two passes. It does not yet
authorize launcher-selected roots or bind them to the earlier capture epoch.
The [fixture replay](verification/source-root-replay/README.md) verifies the
complete retained original frontend without depending on the evolving checkout.
The [live application comparison](verification/source-root-live/README.md)
opens the original application, dependency and SDK roots before capture and
compares 5,516 captured objects. The later export root and launcher authority
are still separate obligations.

The [capture/compiler binding checkpoint](verification/capture-binding/README.md)
connects the independently matched application, dependency and SDK roots to
the same complete compiler/export owner. It retains all 5,697 original objects
and rejects coherently resealed source or namespace changes, a missing required
root and an unmatched export object reclassified as source. The restored test
roots do not authorize launcher paths or authenticate the separate export root;
W01 and the practical profile remain open/inactive.

The [registered analysis source checkpoint](verification/registered-analysis/README.md)
restores the full earlier registered source epoch and derives all 184 build
inputs independently in Go and Rust. Both complete offline release-flag builds
match the existing registered frontend byte-for-byte. This historical epoch
is distinct from the newer analysis source tree; production launcher authority
and practical source/program proofs remain due.

The [matched analysis-root owner](verification/analysis-root-binding/README.md)
retains the independently parsed analysis source owner with the exact matched
root snapshot. The full historical and later source generations, plus coherent
source and namespace changes, are tested without authorizing launcher roots or
claiming source/binary proof.

The [same-recipe source generation comparison](verification/analysis-generation-rebuild/README.md)
compares every captured input in the registered and later analysis generations.
Two full builds of each epoch use the same SDK and release flags; their
cross-epoch binaries differ. This does not attribute a specific source change
or authorize a practical profile.

The [mode-aware PGO input checkpoint](verification/analysis-pgo/README.md)
preserves the existing PGO-off input universe and captures the exact profile
only for automatic selection. All 196 original objects and 184 build inputs
remain, while auto adds the actual 293,923-byte SDK profile. Separate ignored
entry controls preserve original inputs, and actual auto/off builds demonstrate
the differing PGO metadata. Trusted build-mode/launcher binding and W01 remain
open; the practical profile remains inactive and the final W04 gate is due. The
[filesystem lookup follow-up](verification/analysis-pgo-casefold/README.md)
retains both case-sensitive and case-folded observations and independently
checks the actual resolved profile against held roots.

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
`./scripts/check-fast.sh` under the current Go/Rust-only scope.
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

The [complete original observation checkpoint](verification/native-observation/README.md)
requires actual privately owned controlled Go module/package observations before
original module/source/export/image consumption. A live Go-owned full-image
baseline compares every complete native field and archive byte from two
independent readonly executions with fresh caches. All4 module objects/all182
package objects except generated Export locator and181 archives/79406936 bytes
per run agree, also under an independent comparator. Diagnostics, unknown fields,
unmodelled Root, membership, cache escape, archive mutations and lifetime/close
controls reject. These are preparation/data correspondence checks; production
kernel/context/complete general consulted namespace and selected closure remain
due. A genuine original1-native/1-source package without unsafe now binds using
actual intrinsic membership. A sole compiler-guard diagnostic compatibility
failure retains its exact old source/result; after all original handles terminate,
only that prior literal is restored and its exact guard/current frontend builds
pass. Unaffected complete source/control-flow observations are retained.
Two fresh184-input builds agree; current legacy compile-only/vet/format pass.
Independent reconstruction checks33643 complete source preimages across six
epochs,453 extra complete payloads, both snapshots/full image/compiler inventory
and all255 frozen members. All13 original drivers/six containers are terminal.
Full56-all6-19 source/program foundation/private descriptor/registry/context/
retained policies/all169 demands/T02-T11 remain required. R06/W01 stay open;
no public profile/source/program proof activates. C# tests stay skipped; the
cumulative T02 gate remains W04: ./scripts/check-fast.sh --skip-csharp.

The [owned original package checkpoint](verification/original-packages/README.md)
now combines the actual original module/source/export/image consumers. It
reconstructs complete source types and binds controlled Go observations and
whole original archives with independent metadata/byte ownership and lifetime.
Eight affected roots/thirty-five subtests, vet/format and zero-test Go1.23 compilation
pass. The existing four-module/181-package/1,472,495-node original fixture now
uses this owner after retiring original roots and buffers. Two fresh pinned
development builds agree; their new bytes are not registered under a previous
immutable frontend identity. Original SDK/setup failures and full actual buffers
remain separate. Current scope is Go/Rust only; R06/W01/full ordinary foundation/
private context/general namespace/selected closure/all169/T02-T11 remain due.
The current cumulative T02 gate stays W04: `./scripts/check-fast.sh`.

The [ordinary indexed sequence checkpoint](verification/sequence-foundation/README.md)
adds16 actual definitions and36 universal laws for length/prefix/read/update/map/
offset/append and element validity, including unbounded Nat induction for the
right-append index bound. Both unchanged checkers agree on63 whole inputs: two
zero-axiom acceptances and61 real type rejections. Three affected Rust roots,
strict Clippy and format pass; all131 prior Nat/product identities remain exact.
This is a mathematical prerequisite; all frozen Go carrier representations and
source/word/domain/storage/program correspondence remain required. R01/W01/full
private context/namespace/selected closure/all169/T02-T11 remain open. The
cumulative T02 gate stays W04: `./scripts/check-fast.sh`.

The [ordinary finite-vector checkpoint](verification/finite-vector-foundation/README.md)
connects all1..256 arbitrary-element Boolean-addressed slots to explicit-length
Nat-indexed sequence observations and validity. Its full413-member zero-axiom
component preserves all183 original sequence identities. Both unchanged cores
agree on202 whole streams: two acceptances and200 genuine type mismatches.
Three affected Rust tests, strict Clippy and format pass. Generic one-level
proofs and explicit equalities retain the fixed capacities within1024 fuel;
original failed epochs remain separate. Full64-bit word/count/source, nominal
array/padding/empty-dummy/storage/program correspondence and complete private
context remain required. R01/R06/W01 and T02-T11 remain open. The cumulative
T02 gate stays W04: `./scripts/check-fast.sh`.

The [full64 word-index checkpoint](verification/word-index-foundation/README.md)
proves every depth0..8 actual unsigned64 range/high-bit condition and exact
finite-slot observation, while preserving the complete64-bit length/index.
Unbounded Nat induction proves index<count<=capacity, including count256.
The complete1005-member zero-axiom component preserves786 unique old identities
and all950 preliminary declarations; the original4053-member shift bytes remain
exact. Four affected optimized Rust roots, strict Clippy/format and both
unchanged independent cores agree on346 original full streams:11 acceptances
and335 genuine type mismatches. Original debug failure/interruption, its precise
unexposed-trial limit and all actual available byte epochs remain separate.
Full signed/source/nominal/padding/empty-dummy/storage/program correspondence,
complete ordinary foundation/private context/native namespace/selected closure,
R01/R06/W01/all169/T02-T11 remain due. The cumulative T02 gate stays W04:
`./scripts/check-fast.sh`.

The [original string constant checkpoint](verification/string-constants/README.md)
adds a private physical-locator observation over the owned complete original Go
packages. It retains complete source/expression/native bytes and converts full64
length/256 full8-bit data slots with zero padding;257 bytes remains complete and
returns a distinct modeled-domain error. Actual folded/named/alias/selector,
CRLF/all-byte/boundary/invalid-query/damaged/lifetime cases and the complete
four-module HTTP/replacement image pass. Ten affected roots/77 subtests, current
vet/format, zero-test legacy compilation and both pairs of fresh development/
captured-original builds pass. The186-input closure preserves every prior185
input. Original source/output/driver epochs and the initial mistaken alias
expectation remain; separately missing earlier module diagnostics are explicit.
These are native data facts; ordinary source evaluation/curried carrier/nominal
identity/full source-program/private context/kernel/general namespace/selected
closure/all169/T02-T11 remain due. R01/R06/W01 stay open and Go/Rust scope,
frozen limits/published registry remain unchanged. The cumulative T02 gate stays
W04: `./scripts/check-fast.sh`.

The [ordinary string carrier checkpoint](verification/string-foundation/README.md)
implements the exact frozen fullWord64/256 curried Word8 product and actual
constructor/field/bit/full64 sequence/read/zero/validity laws. All1005 prior
identities remain exact; the1053-declaration zero-axiom component adds18
definitions/30 laws. Five affected Rust roots, strict Clippy/three-file format
and both unchanged independent cores cover81 full inputs:22 acceptances and59
genuine type mismatches. Actual19 original Go source/expression/native-value
observations produce18 complete2112-bit certificates and one distinct257-byte
domain error; no nominal/source/context identity is inferred. Complete source/
candidate/output epochs, implicit-zero/projected-cube/import/style failures and
final unchanged semantic outcomes remain explicit. Nonzero literal whole Valid,
source evaluation/nominal identity/signed-domain/full operations/array/storage/
state/program/private context/native kernel/namespace/selected closure/all169/
T02-T11 remain due. Go/Rust scope and all frozen limits remain unchanged.
R01/R06/W01 stay open; the cumulative T02 gate stays W04: `./scripts/check-fast.sh`.
The [whole literal string validity checkpoint](verification/string-validity/README.md)
now proves full unsigned64 length<=256 and universally quantified canonical
whole-byte zero padding for literal carriers. Its1109-declaration zero-axiom
component preserves all1053 original identities; full256 data preserves all3166
old declarations/2112 bit proofs. Seven affected Rust roots, strict Clippy/four-
file format and both unchanged independent cores pass120 complete streams:
38 zero-axiom acceptances and82 genuine type mismatches. All19 original Go
observations yield18 whole-valid carriers and one distinct257-byte domain error;
14 additional binary boundary lengths pass. Malformed carriers reject after
removing all2048 old byte observations and repairing actual64 length assertions,
with their real carrier/length prefix still type-correct. Both original source/
driver epochs and all candidate/output bytes remain. Source/constant evaluation,
nominal identity/signed-domain/full operations/array/storage/state/program/private
context/native kernel/namespace/selected closure/all169/T02-T11 remain due.
Go/Rust scope, frozen limits and R01/R06/W01 stay unchanged. The cumulative
T02 gate stays W04: `./scripts/check-fast.sh`.

The [canonical string normalization checkpoint](verification/string-normalization/README.md)
adds six definitions/28 ordinary laws for full-word length, whole-byte padding,
logical observable content equivalence and actual guarded byte/sequence reads.
Its1143-declaration zero-axiom component preserves all1109 old identities and
exact generic bytes. Both unchanged cores agree on65 complete streams:two
zero-axiom acceptances and63 genuine type mismatches. Four certificate roots
pass in a command retaining one failed finite observer. The observer alone is
corrected and all54 identical raw-carrier cases pass separately;31 old inputs/
30 available outputs, full failures/source epochs and fixed ceilings remain.
Production/core-consumer bytes stay exact; final Clippy/five-file format pass.
Full compare/substring/concat compatibility/source preservation, original Go
source/evaluation/nominal/signed-domain/state/program/full foundation/private
context/native kernel/namespace/selected closure/all169/T02-T11 remain due.
Go/Rust scope and frozen limits stay unchanged. R01/R06/W01 remain open; the
cumulative T02 gate stays W04: `./scripts/check-fast.sh`.

The [full-byte string equality checkpoint](verification/string-equality/README.md)
adds actual ByteEqual/String Equal/NotEqual with reflection/completion to real
full64 size equality and every observable byte bit. Its1215-declaration zero-
axiom component retains all1143 old identities and exact bytes. Six affected
Rust roots, strict Clippy/five-file format and both unchanged cores pass123
complete streams:two zero-axiom acceptances and121 genuine type mismatches.
All59 conclusions/47 real premises/15 typed wrong definitions with actual
counterexamples are checked. Actual definition observation covers all65536 byte
pairs and75 whole raw/normalized cases. Full data/source epochs remain; neither
finite observation nor dirty-padding counterexamples grant source authority.
Original Go source/evaluation/nominal/signed-domain binding, all four bytewise
order comparisons/prefix/suffix/substring/concat/source preservation/full
foundation/private context/native namespace/selected closure/all169/T02-T11
remain due. Go/Rust scope and limits stay unchanged; R01/R06/W01 remain open.
The cumulative T02 gate stays W04: `./scripts/check-fast.sh`.

The [unsigned-byte string order checkpoint](verification/string-order/README.md)
adds actual end/byte ranks and four comparisons. Its1,706 declarations/52,523
terms/679,227 bytes retain all1,215 prior string identities and exact old
StringEquality/full WordOrder bytes, adding26 definitions/190 ordinary laws.
First-difference reflection/completion carries a real address, equality at every
smaller index and a strict rank difference. Full64-size guards distinguish end
from NUL; actual Word8 unsigned bridges, pointwise respects/normalization,
rank-vector total equality and reflexivity are checked without axioms, function
extensionality, core or limit changes. Six affected roots and both unchanged
cores agree on339 complete inputs:two zero-axiom acceptances and337 genuine
mismatches, including190 altered conclusions,122 typed-witness domain changes
and25 typed wrong definitions with actual canonical-padding counterexamples.
Finite actual-DAG observations cover65,536 bytepairs and75 full stringcases,
all four operations,both directions,raw/normalized; they supply no source proof.
A receiver-only Clippy correction regenerates the identical full certificate;
current strict Clippy/all six changed-file format pass. Ten full source epochs/
3,779 preimages retain14 executions and all five original failures, including
both original rejected Boolean candidates and their actual16-case diagnostic.
All255 frozen inputs,186 Go build inputs and127/four/39 scanner counts stay exact,
with no new exclusion or language. The bounded bridge to existing full-size
String.Equality and original source/evaluation/nominal/signed-domain binding,
prefix/suffix/substring/concat/source preservation, private foundation/context,
native kernel/general namespace/selected closure/all169/T02-T11 remain due.
R01/R06/W01 stay open; the cumulative T02 gate remains W04: `./scripts/check-fast.sh`.

The [bounded order/equality checkpoint](verification/string-order-bounded/README.md)
now proves rank equality exactly matches existing full-size String.Equality
under both actual upper witnesses. A strict bound precedes full64 length-to-slot
conversion; end0 versus a present-byte successor recovers length equality and
actual unsigned value/byte reflection recovers all observable bits. Its1,729
declarations/53,121 terms/687,164 bytes retain all1,706 prior identities and exact
old order bytes, adding23 ordinary laws without new definitions/axioms/core/
extensionality/limits. Neither-strict/both-weak equality, weak antisymmetry,
trichotomy and equality/strict/weak consistency use the actual four operators.
Five affected roots, strict target Clippy/five-file format and both unchanged
cores pass62 complete candidates:two zero-axiom acceptances/60 type mismatches,
with23 changed conclusions/37 typed-witness changes. Actual definition evaluation
observes75 whole raw/normalized pairs and five out-of-domain full64 countermodels
showing why bounds matter; these supply no admitted-source counterexample.
Three complete source epochs/1,143 preimages and seven checks preserve the
original trichotomy construction Linkage and all five root failure outputs;
removing the reversed Eq chain repairs construction before any new candidate is
claimed. All255 frozen inputs,186 Go inputs and127/four/39 scanner counts stay
exact. Source/type/evaluation/nominal/signed-domain binding,prefix/suffix/substring/
concat/source preservation/private context/full foundation/native kernel/general
namespace/selected closure/all169/T02-T11 remain due. R01/R06/W01 stay open;
the cumulative T02 gate remains W04: `./scripts/check-fast.sh`.

The [guarded substring checkpoint](verification/string-slice/README.md) now
constructs the complete frozen result with actual signed64 guard equivalence,
full-width subtraction, checked non-wrapping source addition and real bounds
before address conversion. Length, byte content, canonical padding, validity,
weak content equivalence and normalization are proved. Its 2,142 declarations,
67,192 terms and 865,248 bytes retain all 1,729 prior identities and exact old
producer bytes; six definitions and 53 ordinary laws change no axiom, core rule,
extensionality, capacity or limit. Source capacity is a profile precondition.
Five affected Rust roots, strict Clippy, ten-file format and both unchanged cores
pass 125 complete inputs: two zero-axiom acceptances and 123 type mismatches
(53 conclusions, 66 typed witness domains, four typed wrong definitions with
actual countermodels). Actual-DAG execution observes 78 whole raw/normalized
outputs and 307 signed guards in 113,263,317 steps. Output-only replays keep
complete 256-slot inputs/all64 endpoint bits and unchanged observer ceilings.
Eleven full source epochs/4,251 preimages retain 13 checks and all seven original
failures with available candidates and outputs. All 255 frozen inputs, 186 Go
inputs and 127/four/39 scanner counts remain exact, without new exclusions.
Original Go source/type/evaluation/nominal binding and source preservation,
checked prefix/suffix, bounded concat, private context/full foundation/native
kernel/general namespace/selected closure/all169/T02–T11 remain due. Go and Rust
remain the only source languages. R01/R06/W01 stay open; the cumulative T02 gate
remains W04: `./scripts/check-fast.sh`.

The [bounded concatenation checkpoint](verification/string-concat/README.md)
adds the complete frozen result, actual64-bit addition/carry and right-index
subtraction, with the full length-sum capacity obligation kept separate from
Go panic definedness. Length conservation, no carry, input/result bounds,
full-word byte equations, canonical padding/validity and observable equivalence/
normalization preservation are proved. Its 2,195 declarations, 69,528 terms and
896,032 bytes retain all 2,142 prior identities and exact old producer bytes;
seven definitions and 46 ordinary laws change no axiom/core/extensionality/limit.
Five affected Rust roots across identical source bytes, strict Clippy, eight-file
format and both unchanged cores pass 117 actual inputs: two zero-axiom acceptances
and 115 type mismatches (46 conclusions, 62 typed witness domains, seven typed
wrong definitions with actual countermodels). Passed roots are retained without
unnecessary repeat. Actual-DAG execution covers 100 whole outputs in four
normalization modes, 28 profile rejections and eight full64 carry cases in
536,136,409 steps, with complete inputs and unchanged observer ceilings.
Seven source epochs/2,758 preimages retain ten checks and all three original
failures. All 255 frozen inputs, 186 Go inputs and 127/four/39 scanner counts
remain exact. Original source/type/evaluation/nominal binding and preservation,
checked prefix/suffix, private context/full foundation/native kernel/general
namespace/selected closure/all169/T02–T11 remain due. Go and Rust remain the only
source languages. R01/R06/W01 stay open; the cumulative T02 gate remains W04:
`./scripts/check-fast.sh`.

The [checked prefix/suffix checkpoint](verification/string-affix/README.md)
adds full-index reflection/completeness, actual64-bit source position equations,
normalization/weak-content preservation and empty/self/too-long/concat laws.
Its2,254 declarations/71,079 terms/916,686 bytes preserve all2,195 previous
identities and exact896,032 old producer bytes; ten definitions and49 ordinary
laws change no axiom/core/extensionality/cap/limit. Five affected Rust roots,
strict target Clippy/nine-file format and both unchanged cores pass128 full
inputs:two zero-axiom acceptances and126 type mismatches. Actual-DAG tests
check216 whole predicates,504 full64 positions and six wide guards in619,708,955
steps. Nine source epochs/3,605 preimages retain13 executions and three
original failures. All255 frozen/186 Go inputs and127/four/39 scanner counts
remain exact. Original source/type/evaluation/nominal/library-symbol adapters and
preservation, full foundation/context/native kernel/general namespace/selected
closure/all169/T02–T11 remain due. Go and Rust remain the only source languages.
R01/R06/W01 stay open; the cumulative T02 gate remains W04:
`./scripts/check-fast.sh`.

The [fixed-array checkpoint](verification/array-foundation/README.md) adds
all frozen empty/positive array shapes,full64 reads and signed/unsigned guards,
zero/copy/padding/normalization/update validity,visible equality and heterogeneous
element-relation preservation. Its2,688 declarations/80,853 terms/1,033,685 bytes
retain all2,254 prior identities and exact916,686 old affix bytes;131 definitions
and303 ordinary laws add no axiom/core/extensionality/root/cap/limit change.
Five affected Rust roots,strict Clippy/eight-file format and both unchanged cores
pass794 complete inputs:two zero-axiom acceptances and792 genuine type
mismatches (303 conclusions,452 typed witness domains,37 typed wrong
definitions with actual countermodels). Actual-DAG evaluation checks132 whole
outputs/396 full64 reads/nine guard pairs/one empty dummy in34,700,456 steps.
Nine source epochs/3,679 preimages retain14 checks and five original
failures;two original full failed candidates also reject independently in Go.
All255 frozen/186 Go inputs and127/four/39 scanner counts remain exact,without
new exclusions. Go and Rust remain the only source languages. Full source/type/
evaluation/nominal and array-store/aliasing/state/program correspondence,complete
private foundation/context/native Linux/general namespace/selected closure/all169/
T02–T11 remain due. R01/R06/W01 stay open;the cumulative T02 gate remains W04:
`./scripts/check-fast.sh`.

The [array-comparison checkpoint](verification/array-equality/README.md) adds
actual element-comparator folds, reflection/completeness, inequality/refutation,
full 64 read reflection, conditional algebra and pointwise/normalization/copy/
relation preservation for every positive depth 0..8 and canonical empty values.
Its 2,981 declarations/89,121 terms/1,132,748 bytes retain all 2,688 prior identities
and exact 1,033,685 old array bytes;47 definitions and 246 ordinary laws change no
axiom/core/extensionality/root/cap/limit. The closed byte instantiation uses the
actual existing eight-bit comparator and directly binds its `!=` wrapper.
Five affected Rust roots, strict Clippy/eleven-file format and both unchanged
cores pass 783 whole inputs:two zero-axiom acceptances and 781 type
mismatches (246 conclusions,488 typed witness domains,47 typed wrong
definitions with actual countermodels). Pinned original Go comparisons cover
all 257 lengths and 1,028 whole input pairs; actual-DAG execution matches2,584
operator results in 536,538,673 steps, including raw and normalized operands.
Eight native compiler type rejections include zero-length noncomparable arrays.
The whole SDK archive/all 14,496 original files retain exact bytes. Thirteen
source epochs/5,454 preimages retain 19 checks and six original failures;
the initial accepted ByteNotEqual corruption and stopped failed suite remain
separate evidence. Only affected checks rerun after the nine missing wrapper
laws are added. All255 frozen/186 frontend inputs and 127/four/39/51 scanner
counts remain exact, without new exclusions. Go and Rust remain the only source
languages. Full source/type/evaluation/nominal preservation, storage/aliasing/
state/program, private foundation/context/native Linux/general namespace/selected
closure/all 169/T02–T11 remain due. R01/R06/W01 stay open; the cumulative T02 gate
remains W04: `./scripts/check-fast.sh`.

The whole original SDK archive uses seven reversible text chunks with exact decoded bytes. The original failed input review and its three compressed-binary token matches remain reported; decoded SDK/source inputs have no findings, and no scanner exception is added.

The [slice-descriptor checkpoint](verification/slice-descriptor/README.md)
implements the exact frozen six-field representation: nil Bool, Nat owner,
16 full Word8/Word64 path cells and full Word64 offset/length/capacity.
Construction, projection, copy, reconstruction, field equality and metadata
updates preserve complete values and owner/path references. Full signed and
unsigned index/two/three-bound guards match actual Nat/Int interpretations;
reslicing uses capacity, nil has no index, and address addition is unbounded.
HeaderValid binds actual bounds and canonical nil metadata, with typed live
storage/NoAccess/source/permission correspondence explicitly still required.
The 2,290-declaration/69,289-term/896,669-byte component adds 35 definitions and
113 ordinary laws, preserving all 2,142 prior identities/full 865,248 parent
bytes without axiom/core/extensionality/root/cap/limit changes. Five affected
Rust roots, strict Clippy/six-file format and both unchanged cores pass 186
complete inputs: two zero-axiom acceptances and 184 genuine type mismatches
(113 conclusions, 43 typed witness domains, 28 runtime definitions with actual
countermodels). Original Go supplies 400 complete inputs/2,400 bounds results,
16 descriptor cases and ten shared-view observations. Actual-DAG execution
checks 1,264 complete outputs and all 2,400 guards in 60,486,975 steps, including
all 256 raw path-kind byte values and complete high index/offset bits.
Five source epochs/2,073 preimages retain 11 executions/three original fuel
failures; all three original complete candidates also reject in Go. The
separate original fixture-path capture failure precedes Cargo and retains its
full changed preimages. No original output is invented. Frozen/source/scanner
identities remain exact without new exclusions. Go and Rust remain the only
source languages. Full source/type/nominal preservation, typed live owners/
views/aliasing/permissions/lifetime/state/program, private foundation/context/
native Linux/general namespace/selected closure/all 169/T02–T11 remain due.
R01/R06/W01 stay open; the cumulative T02 gate remains W04:
`./scripts/check-fast.sh`.

The [storage-view checkpoint](verification/storage-lens/README.md) adds
ordinary typed current-parent read/update composition, identity/association,
heterogeneous field views, all array depths 0 through 8, and disjoint frame
composition. Full Word64/Nat indices and real shape/range/relation/validity
premises remain explicit; recursive element observations avoid function
extensionality. Its 2,902 declarations/87,423 terms/1,115,975 bytes preserve all
2,688 prior identities/full 1,033,685 producer bytes and reuse the existing 16
product-relation declarations. The lens namespace adds 17 definitions and
181 laws, with zero axioms and unchanged core/limits/private registration.
Five affected Rust roots, strict target Clippy/11-file format and both cores
pass 448 full inputs: two zero-axiom acceptances and 446 type mismatches
(181 conclusions, 249 selected real premise domains, 16 typed wrong definitions with
actual countermodels). Original Go supplies 203 complete buffer pairs/1,015
root snapshots; actual ordinary definitions match 1,060 roots/406 alias reads
in 67,909,080 aggregate steps, including nine nested-array splits and paths
through length 16. Eleven source epochs/4,848 preimages retain 17 executions
and four Rust failures; two complete failed candidates also reject in Go.
The original native JSON-tag failure retains all original sources/actual
outputs; its overwritten first binary is unavailable and not reconstructed.
The exact current executable is retained. No scanner exclusion is added.
Go and Rust remain the only source languages. Actual source/nominal binding,
typed path synthesis/live owners/shared stores/aliasing/permissions/lifetime/
state/program, full foundation/private registry/context and native Linux/
general namespace/selected closure/all 169/T02–T11 remain due. R01/R06/W01
stay open; the cumulative T02 gate remains W04: `./scripts/check-fast.sh`.

The [original storage-source checkpoint](verification/storage-source/README.md)
adds native typed storage-route extraction from exact physical spans in owned
original packages. Full ordered type closures, physical named/alias/imported
declarations, promoted field paths and full-width integer index expressions
remain bound to complete original source bytes. Slice indexing explicitly
pivots to its descriptor; it grants no owner/path/offset/permission proof.
Anonymous Go type equality shares shapes without conflating source declarations.
All native child edges are checked, including zero arrays; geometry256/64/16
and the final artifact4096 limit remain unchanged. Native observation nodes use
the existing4,194,304 source-syntax budget, not an incorrect count of semantic
type declarations. Global-U/Plain/weight/final-depth and canonical source IDs
remain separate. Seven affected Go roots/62 subtests, two exact original-input
frontend builds (188 inputs), inactive public boundaries, vet/three-file format
and Go1.23.2 compile pass. Nine source epochs/2,241 preimages retain24 executions;
five exact lossless archives preserve all original native inputs/outputs.
Existing Rust448-case view evidence is retained without unnecessary reruns.
Go and Rust remain the only source languages. Independent Rust source/nominal/
path import, ordinary source/route/lens correspondence, shared live owners/
permissions/lifetimes/state/program, full foundation/context/registry and all
native Linux/general namespace/selected closure/all169/T02–T11 remain due.
R01/R06/W01 remain open; T02 whole gate stays W04: `./scripts/check-fast.sh`.

The [original-source type-key checkpoint](verification/type-keys/README.md)
adds a private Go producer and independent Rust component importer. Exact owned
original source bytes, full active package sets, real go.mod objects, physical
named/alias TypeSpec spans, raw tag bytes, embedding, pinned UnicodeLu export
scopes, distinct native int/uint names and alias-transparent semantic children
are retained in new private v2 key preimages. All255 frozen old draft inputs
remain exact. Five actual original Go streams import in Rust with identical
canonical bytes/hashes; nine affected Go roots/67 subtests and five Rust roots,
actual189-input paired frontend builds, inactive boundaries, strict lint,
changed-file format and legacy compile pass. Ten epochs/6,953 preimages retain
23 actual checks, two original check failures, four original prelaunch races,
and six lossless original archives. The current driver serializes preimages.
These are internally consistent candidate components only; synthetic64-field/
depth16 roots deliberately do not gain source-typing authority. Full original
source/nominal/alias/capture proofs, U/Plain/weight/final-depth/counts, ordinary
source/carrier/route/lens correspondence, live stores/permissions/lifetime/state/
program, complete foundation/context/registry and all native Linux/general
namespace/selected closure/all169/T02–T11 remain due. Go/Rust only; R01/R06/W01
stay open and the T02 whole gate remains W04: `./scripts/check-fast.sh`.

The [independent Go lexer checkpoint](verification/go-lex/README.md) adds a
Rust physical tokenizer over owned original Go buffers, including pinned Unicode,
all number/operator/literal forms, comments/directives and exact semicolon rules.
All5853 real Go1.25 reference cases agree:5714 accepted streams contain7,702,601
exact token tuples and139 cases reject in both. The default2919-case edge corpus
and ownership controls pass; a separate4,194,307-token Go/Rust boundary has the
same whole stream hash. Token count follows owned byte length plus two, not the
unchanged AST-node budget. Two Go and three Rust roots, strict lint and scoped
formats pass. Eight epochs/5662 preimages retain15 checks and all35,114 original
files/565,549,694 bytes in lossless native and deduplicated Rust transports. The
whole pinned SDK, frozen255 design inputs, existing189 frontend inputs and prior
448-case ordinary/core evidence remain exact; unaffected suites are retained.
Lexing gives no parsing, typing, source/nominal/capture, source/carrier/path,
live-store/permission/lifetime/program or profile authority. Those complete
requirements, full U/foundation/context/registry and all native Linux/general
namespace/selected closure/all169/T02–T11 remain due. Go/Rust only; R01/R06/W01
stay open and the T02 whole gate remains W04: `./scripts/check-fast.sh`.

The [independent Go syntax checkpoint](verification/go-parse/README.md) adds
Rust parsing of full source declarations, types/generics, expressions and function
bodies/statements into owned physical syntax arenas. Every2996 real Go1.25
reference decision agrees:2969 accept/27 reject, with5,731,905 exact canonical
AST nodes across50 kinds. Default62-case, ownership/raw-byte/untyped-boundary and
0/1000/20000/100000-depth tests pass. Two Go and three Rust roots, strict lint,
scoped formats and the actual433.43-second full original-source comparison pass.
Twelve epochs/8577 preimages retain20 executions, four original failures and all
12,165 original files/1,268,912,231 bytes, with literal decoded byte/mode audits.
The select-case colon bug is fixed without discarding original failures. Frozen
255 inputs, complete SDK,189 frontend inputs and ordinary448-case proof basis
remain exact and unaffected suites are retained. Syntax supplies no name/type/
nominal/capture/version/context or source/carrier/store/program authority. Those
full requirements, U/foundation/registry, nativeLinux/generalnamespace/selected
closure/all169/T02–T11 remain due. Go/Rust only; R01/R06/W01 stay open and the T02
whole gate stays W04: `./scripts/check-fast.sh`.

The [original declaration syntax checkpoint](verification/source-declarations/README.md)
adds independent Rust binding of all candidate package clauses and exact physical
TypeSpec/name/alias membership, retaining complete immutable parsed sources.
All five actual Go-produced streams keep identical canonical bytes/hashes; eight
affected Rust roots, strict lint and scoped format pass. Sixteen fully rehashed
source controls reject, while six constructed physical/local/Unicode/untyped
boundary cases document syntax-only acceptance. Two input epochs/1488 preimages
retain four checks and all236 original files/842726 bytes with literal byte/mode
comparison. Parser/lexer, Go frontend189 inputs, frozen255 and ordinary448-stream
basis remain unchanged; unaffected evidence is retained. Type-expression graph
denotation, full scope/type/constant/nominal/capture/context and storage/program
requirements remain due, together with full56 foundation/registry, nativeLinux/
generalnamespace/selectedclosure/all169/T02–T11. Go/Rust only; R01/R06/W01 stay
open and the T02 whole gate stays final W04: `./scripts/check-fast.sh`.

The [independent lexical scope checkpoint](verification/source-scopes/README.md)
retains actual package/file/function/block/implicit-clause/type-parameter
bindings, visibility starts, import aliases/default names/dot exports and shadow
blockers from complete owned syntax. All117 real Go lexical observations match
original definition source/position and binding kind. One Go and three Rust roots,
strict Clippy, scoped vet and changed-file formats pass; only affected roots
rerun after fixes. Nine epochs/6510 preimages retain21 executions/four original
failures and all22433 original files/174595501 bytes with literal byte/mode audit.
Unresolved import metadata never guesses a name or exposes an uncertain outer
binding. Existing parser/lexer/importer behavior, frozen255, frontend189 and
ordinary448-case basis remain unchanged and are retained. Constant evaluation,
full type-expression/nominal/alias/module/capture/context and ordinary source/
storage/program requirements remain due, together with full56 foundation/
registry, nativeLinux/generalnamespace/selectedclosure/all169/T02–T11. Go/Rust
only; R01/R06/W01 stay open and the T02 whole gate stays W04: `./scripts/check-fast.sh`.

The [independent scalar constant checkpoint](verification/source-constants/README.md)
evaluates real original scalar syntax with multiword exact integers, pinned
compiler precision and typed representability, preserved scalar/nominal keys,
byte strings/runes/bools and grouped/inferred/imported constants. Repeated specs
use their actual current lexical point and iota; inherited annotations and name
shadowing remain exact. All4832 actual Go scalar values/types match;281 real
rejections agree and four valid outside-reader cases stay explicitly unmodeled.
One Go/two new Rust/three affected scope roots, strict lint/scoped vet/formats
pass. Runtime-pinned native rerun preserves10235 full original files exactly;
SDK14496, frontend189, frozen255 and ordinary448 basis remain unchanged. Seven
epochs/5063 preimages retain18 executions/three original failures and all30978
original files/66409565 bytes with literal byte/mode audit. Full type/array/
nominal/source-evaluation/capture/context and ordinary storage/program proofs,
resource closure, full56 foundation/registry and nativeLinux/generalnamespace/
selectedclosure/all169/T02–T11 remain due. Go/Rust only; R01/R06/W01 stay open
and the T02 whole gate stays W04: `./scripts/check-fast.sh`.

The [original source type denotation checkpoint](verification/source-types/README.md)
derives private v2 scalar/nominal/alias/array/slice/struct graphs from actual
owned type syntax and scalar constant lengths. Ordered grouped/embedded alias
fields, package visibility and exact raw tags agree with seven new and five
retained native streams. Normalized underlying types erase intermediate nominal
spelling while original declaration/alias traces remain. Larger immutable
source scopes handle omitted unused imports only with exact source/package
matching. Four new Rust roots, seven fully rehashed body controls, two context
substitutions, eight existing importer roots and scoped lint/vet/formats pass.
Seven epochs/5288 preimages retain17 checks/three original failures and all11339
original files/88718325 bytes with literal byte/mode audit; missing partial
outputs remain unreconstructed. SDK14496, frontend189, frozen255 and previous
scalar/scope/parser/lexer/ordinary448 evidence remain unchanged. Complete source
typing/evaluation/carrier/module/capture/context/resource and storage/program
proofs, full56 foundation/registry and nativeLinux/generalnamespace/selected
closure/all169/T02–T11 remain due. Go/Rust only; R01/R06/W01 stay open, public
practical profile remains inactive and the T02 whole gate stays W04:
`./scripts/check-fast.sh`.

The [original signature and binding-annotation checkpoint](verification/source-signatures/README.md)
retains ordered receiver/input/result and declared-variable types with exact
original Field/name/type nodes, grouped sharing, unnamed/blank slots and local/
imported aliases. Supported27 signatures and250 declared bindings match real Go
keys and physical origins. Four valid signatures/three value types remain out
of domain, five inferred variables require expression typing and one native
function-type name remains type-only metadata. Local nominal receivers, fixed
result arity0–16,70-input sharing, variadic-variable slice normalization and
annotation/body separation retain their distinct checks. Five new Rust roots,
four affected type roots and scoped lint/vet/formats pass. Eleven epochs/8344
preimages preserve24 executions/three failures and all33373 original files/
262995192 bytes with literal byte/mode audit; missing partial outputs are not
recreated. SDK14496/frontend189/frozen255/ordinary448 basis remains exact.
Expression/initializer/body/assignment/call/state typing, source evaluation/
carrier/capture/context/resource and storage/program proofs, full56 foundation/
registry and nativeLinux/generalnamespace/selectedclosure/all169/T02–T11 remain
due. Go/Rust only; R01/R06/W01 stay open, public practical profile stays inactive
and the T02 whole gate stays W04: `./scripts/check-fast.sh`.

The [independent scalar typing checkpoint](verification/scalar-typing/README.md)
derives original scalar expression and shared/inferred/short initializer types
with canonical nominal identity, real default/expected contexts, exact constant
versus runtime rules and scalar0–16 static call-header linkage. Contextual shifts
and builtin min/max internal defaulting remain distinct; single-value and
result-list APIs preserve arity. All2775 exact positive type/value/addressability
outcomes and2205 actual Go rejections agree; five valid outside-reader cases stay
explicit. Three new Rust/two affected constant roots,20000-node flat contextual
work and scoped lint/vet/formats pass. Nine epochs/6862 preimages preserve18
checks/three failures and all49052 original files/126193706 bytes with literal
byte/mode audit, including both original false partial acceptances. SDK14496/
frontend189/frozen255/ordinary448 basis remains exact. Aggregate/method/range/
interface/body/assignment/call-state typing and source evaluation/carrier/
capture/context/resource/storage/program proofs, full56 foundation/registry and
nativeLinux/generalnamespace/selectedclosure/all169/T02–T11 remain due. Go/Rust
only; R01/R06/W01 stay open, public practical profile stays inactive and the T02
whole gate stays W04: `./scripts/check-fast.sh`.

The [original aggregate value typing checkpoint](verification/value-typing/README.md) extends the shared
scalar engine with canonical array/slice/struct assignments and conversions,
actual direct/promoted field visibility/ambiguity, indexing/slicing and original
int index contexts, literal element contexts, array/slice range variables and
aggregate static call headers. Receiver aliases use actual type denotation. Nil origin remains distinct from its actual
destination materialization. All474 exact positive outcomes and463 actual Go
rejections agree on950 final cases; thirteen valid forms stay explicitly
unmodeled. Two new Rust/three affected scalar roots, original4985 scalar cases,
20000-node work and scoped lint/vet/formats pass. The scalar basis now has2776
exact positives/2205 agreeing rejections/four unmodeled float forms. Twenty-six
epochs/20013 preimages retain48 checks/four failures and50550 original files/
154017214 bytes with literal byte/mode audit; incomplete original outputs are
not reconstructed. SDK14496/frontend189/frozen255/ordinary448 basis stays exact.
make/append/copy/method/interface and full body/statement/call-state typing,
source evaluation/carrier/capture/context/resource/storage/program proofs,
full56 foundation/registry and nativeLinux/generalnamespace/selectedclosure/
all169/T02–T11 remain due. Go/Rust only; R01/R06/W01 stay open, public practical
profile remains inactive and the T02 whole gate stays W04:
`./scripts/check-fast.sh`.

The [original slice builtin typing checkpoint](verification/slice-builtins/README.md) adds make/append/copy
with actual original named/element types, size/index contexts, value/spread/
string-byte forms, tuple calls and retained source-node headers in both value
and scalar API projections. All510 exact positive type observations and1143
original rejections agree on1659 cases; three valid outside-key forms and three
actual Go1.25 checker panics stay separate. Four same-source compiler controls
confirm tuple/string append-spread compilation and preserve three compiler
internal errors; type-info agreement supplies no compiled-source admission.
Two new Rust/three scalar/two aggregate roots and scoped lint/vet/formats pass;
all4985 scalar/950 aggregate outcomes and20000-node work remain exact. Ten epochs/
7723 preimages retain19 scoped executions/three failures and24139 original files/
80622279 bytes with literal byte/mode audit. SDK14496/frontend189/frozen255/
ordinary448 basis stays exact. Complete method/interface/body/statement/call-state
typing, source evaluation/carrier/capture/context/resource/storage/program proofs,
full56 foundation/registry and nativeLinux/generalnamespace/selectedclosure/
all169/T02–T11 remain due. Go/Rust only; R01/R06/W01 remain open, public practical
profile stays inactive and the T02 whole gate stays W04: `./scripts/check-fast.sh`.

The [original method/error typing checkpoint](verification/method-typing/README.md) resolves actual value
methods and immediately called method expressions with canonical receiver
identity, aliases, original parameter/result types, visibility, field/method
shadowing and repeated embedding paths. Error() string signature dependencies
remain distinct from actual calls; pointer/value and interface-dispatch cases
are explicit. All143 exact positive type/provenance outcomes and300 original
rejections agree on461 cases;18 valid outside-reader forms stay separate. Real
Go selections agree on69 caller/callee/body/type records,23 promoted paths,
18 method expressions and8 opaque declarations. Three new Rust/seven affected
scalar/aggregate/builtin roots,70-input/16-result controls,200-call flat work,
original4985/950/1659 outcomes and scoped lint/vet/formats pass. Nine epochs/
6990 preimages retain22 scoped checks and31709 original files/98995757 bytes
with literal byte/mode audit. SDK14496/frontend189/frozen255/ordinary448 basis
stays exact. Promotion/interface type facts do not relax frozen v1 admission.
Complete body/statement/call-state/source-evaluation/carrier/capture/context/
resource/storage/program proofs, full56 foundation/registry and nativeLinux/
generalnamespace/selectedclosure/all169/T02–T11 remain due. Go/Rust only;
R01/R06/W01 stay open, public profile remains inactive and the T02 whole gate
stays W04: `./scripts/check-fast.sh`.

The [original assignment typing checkpoint](verification/assignment-typing/README.md) checks actual assignment/short-declaration/compound/incdec syntax,
writable targets, complete parallel/tuple lists, contextual constants/shifts,
blank destinations and original new/reused bindings. All475 exact positives and
951 actual rejections agree on1433 Go1.25 cases;7 valid outside-reader forms stay
separate.903 ordered slots/215 new bindings/72 blanks/17 original static call
headers agree independently. Three new Rust/ten affected old roots,200-variable
parallel/short lists,16/17-result boundaries,20000-node contextual work and scoped
lint/vet/formats pass. Five epochs/3902 preimages retain14 executions/one malformed
deep-fixture failure and24739 original files/77332904 bytes with literal byte/mode
audit. SDK14496/frontend189/frozen255/ordinary448 basis stays exact. Return/control/
whole-body checking, actual evaluation/store/call-state/program proofs, full56
foundation/registry and nativeLinux/generalnamespace/selectedclosure/all169/
T02–T11 remain due. Go/Rust only; R01/R06/W01 remain open, public profile stays
inactive and the full T02 gate stays finalW04: `./scripts/check-fast.sh`.

The [original return typing checkpoint](verification/return-typing/README.md) binds actual return nodes through original AST edges to the nearest
function declaration/literal and checks ordered results, tuple forwarding,
constant/shift contexts and named/blank/shadowed implicit returns. All150 exact
positive type/provenance outcomes and470 rejections agree on630 Go1.25 cases;
10 valid forms stay outside the reader.2214 signature slots/448 result slots/
205 named variables/39 implicit returns/8 nested owners/4 methods/17 call records
agree independently. Three new Rust/thirteen affected old roots,70-input/16-and17-
result controls,20000-node work and scoped lint/vet/formats pass. Nine epochs/
7062 preimages retain20 executions/two initial failures and24054 original files/
78163074 bytes with literal byte/mode audit. The initial unused-import package
rejection remains separate from partial return typing; both original corpora
remain. SDK14496/frontend189/frozen255/ordinary448 basis stays exact. Actual
condition/control/whole-body and evaluation/state/program proofs, full56
foundation/registry and nativeLinux/generalnamespace/selectedclosure/all169/
T02–T11 remain due. Go/Rust only; R01/R06/W01 stay open, the public profile stays
inactive and the whole T02 gate stays finalW04: `./scripts/check-fast.sh`.

The [original function-body typing checkpoint](verification/body-typing/README.md) checks every supported original statement, including unreachable syntax,
local/blank/repeated const declarations, conditions/switch duplicates, array/slice
ranges, actual branch targets, unused locals and Go syntactic final-return rules.
All174 exact positive outcomes and299 rejections agree on495 Go1.25 cases;22 valid
reader gaps remain explicit.1665 signature slots/1972 statements/442 expression
uses/113 branches/14 const specs and16 members/57 locals/28 syntactically
terminating bodies agree independently. Four new Rust/sixteen old value-statement/
two constant/four source-type roots,1001-statement/70-input/20000-node controls
and scoped lint/vet/formats pass. Four epochs/3305 preimages retain17 passed
checks and32576 original files/92971348 bytes with literal byte/mode audit.
SDK14496/frontend189/frozen255/ordinary448 basis stays exact. Const len/cap of
arrays remain required v1 reader gaps; syntactic termination proves no runtime
halting. Other bodies/packages/selected closure, source evaluation/state/program
proofs, full56 foundation/registry and nativeLinux/generalnamespace/selectedclosure/
all169/T02–T11 remain due. Go/Rust only; R01/R06/W01 stay open, the public profile
stays inactive and the whole T02 gate stays finalW04: `./scripts/check-fast.sh`.

The [original array-constant dependency checkpoint](verification/aggregate-constants/README.md)
closes the required len/cap gaps through actual declared/inferred/alias/return/
import type inference, current repeated-ConstSpec iota/scope and compiler variable
reads. Flat shared dependency work rejects cross-reader cycles. All345 exact
positive values/types and56 rejections agree on406 Go1.25 cases;5 frozen-domain
boundary controls remain outside. Four new/38 current affected Rust roots,
512-link/four-thread controls and scoped Clippy/vet/formats pass. Original495 body
cases now give176/299/20; original5117 constant cases4833/281/3, with all original
corpus bytes retained.8 epochs/6656 preimages preserve24 checks/one initial
visibility failure and15 archives/34406 files/93144308 bytes pass literal byte/mode
audit. Two scope/signature input-inventory followup reruns agree exactly with the
initial maps. SDK14496/frontend189/frozen255/ordinary448 basis stays exact.
Whole-v1/package/selected closure and actual source evaluation/state/program
proofs, full56 foundation/registry and nativeLinux/generalnamespace/all169/T02–T11
remain due. Go/Rust only; R01/R06/W01 stay open, the public profile stays inactive,
and the T02 whole gate stays finalW04: `./scripts/check-fast.sh`.

The [original source dependency metadata checkpoint](verification/source-closure/README.md)
uses intact owned native packages to retain exact roots, original function/type/
value-spec dispositions,528 physical calls,1101 body/type/constant/signature/
rejected edges and23 issues with root paths. All31 selected cases (13 positive
metadata/18 diagnostic),512-function/70-parameter/70-root controls, inherited
constant shadows, Error signature/body distinction and source ownership pass.
The15 crossed-owner controls and81 original native source outcomes remain exact;
unselected dependency type errors reject. Current Go formats/vet pass.15 epochs/
12735 preimages preserve25 checks/two original fixture/assertion failures and9
archives/55282 files/494736364 bytes compare literally. SDK14496/frozen255/old
frontend189/ordinary448 basis remains; one new private Go production file is
explicit. Independent Rust closure validation, contracts/Root/Plain/closed error,
complete capture/context/source execution/state/program proofs, full56 foundation/
registry and nativeLinux/generalnamespace/all169/T02–T11 remain due. Go/Rust only;
R01/R06/W01 stay open, the public profile stays inactive, and the T02 whole gate
stays finalW04: `./scripts/check-fast.sh`.

The current190 captured frontend inputs additionally rebuild twice from independent
empty caches with identical8592898-byte outputs (SHA256
`589e82c3c73b95df654501cc71776d9a1a3095371a0ee11170ac2dff4a816eea`).
This is a local Darwin/arm64 build observation; native Linux acceptance remains due.

The [independent original-source selection checkpoint](verification/source-selection/README.md)
reconstructs selected bodies/static calls/type/constant/signature dependencies in
Rust from owned original buffers and physical roots, without native AST/type/call
facts. Go now explicitly inventories all unselected nested declarations. All33
native cases pass;15 independent positive graphs match9795 declaration rows,
516 calls/1071 dependencies, and18 independent negatives reject. Three new and
11 affected old Rust roots,512 helpers/70 parameters/70 roots, scoped Clippy/vet/
formats pass.21 epochs/17902 preimages retain34 executions/three original
failures;11 archives/23775 files/232605508 bytes pass literal byte/mode audit.
The190 frontend inputs rebuild twice to identical8592898-byte outputs with
SHA256 `764e57a7db286f4a76fc41bb1b15e92cf23a57fc8d6400f4fec84712afc02d9b`.
SDK14496/frozen255/ordinary448 stay exact. Capture/wire/contract authority,
complete v1 admission and source runtime/state/program proofs, full56 foundation/
registry and nativeLinux/all169/T02–T11 remain due. Go/Rust only; R01/R06/W01
remain open, public profile stays inactive, and the T02 full gate remains
finalW04: `./scripts/check-fast.sh`.

The [independent function-directive checkpoint](verification/source-directives/README.md)
rejects reached function documentation directives in both body and signature
roles, including Error compatibility, from original bytes alone. Physical
comment groups, inserted semicolons, CRLF/internal CR and //line retain original
spans/paths; unselected functions stay type-check-only. All51 current native
cases and33 unchanged original Rust cases pass:39 exact positive graphs/45
rejections, with11 exact directive spans/paths. Four affected Rust roots,512
helpers/70 parameters/70 roots and scoped Clippy/vet/formats pass.12 epochs/
10229 preimages retain16 executions/three failures;6 archives/22619 files/
254849781 bytes compare literally. The explicit original-corpus inventory
followup reruns all roots and both174-file passing maps agree byte for byte.
SDK14496/frontend190/frozen255/ordinary448 stay exact. Capture/wire/contract/full
v1 admission and source runtime/state/program proofs, full56 foundation/private
registry and nativeLinux/all169/T02–T11 remain due. Go/Rust only; R01/R06/W01
stay open, public profile stays inactive and the T02 full gate stays finalW04:
`./scripts/check-fast.sh`.

The [original error-boxing checkpoint](verification/error-boxings/README.md)
records original concrete-to-error operands with separate tuple-result/range-
element projections and derives selected-body type unions without caller
compiler facts. Both Go/Rust walks now honor unevaluated constant-array ranges.
All46 new original cases match125 complete body sites/unions;128 emitted value
sites are a subset of129 compiler sites, with the compile-only ConstLen site
explicitly delegated. Two new/24 affected old Rust roots,70-argument/16-result/
512-helper/70-root controls and Clippy/vet/formats pass. All51 old native closure
cases pass within the earlier mixed runs, whose new output-directory failures
remain overall failures.15 epochs/12854 preimages retain26 executions/three
failures;14 archives/54753 files/385437972 bytes compare literally.190 frontend
inputs rebuild twice to identical8592898-byte outputs, SHA256
`cf8dc06b5f000f8256f481c82135713e89a93876e420014485b4893846f97121`.
SDK14496/frozen255/ordinary448 remain exact. Entry-case constraints/full global U,
Root/Plain/payload/cycle/weight admission and finite Error dispatch, complete
capture/contracts/source runtime/state/program proofs, full56 foundation/private
registry and nativeLinux/all169/T02–T11 remain due. Go/Rust only; R01/R06/W01 stay
open, public profile stays inactive, and the T02 full gate stays finalW04:
`./scripts/check-fast.sh`.

The [source value-domain checkpoint](verification/source-domains/README.md)
binds complete explicit error entry paths to original selected input types and
combines their cases with the selected-body union. Static expanded-cycle,
Root/Plain, depth and per-value weight checks now use that single union, with
original owner IDs and array-length/slice-capacity distinctions. Scoped source
checks pass; application/stdlib role authority, raw overlay/context binding,
finite Error dispatch, runtime/store/frame/program proofs and R01/R06/W01
completion remain due. Go/Rust only; the T02 full gate remains final W04.

The [finite Error source-selection checkpoint](verification/error-dispatch/README.md)
combines explicit entry cases and original body boxings in a growing worklist.
Every possible Error target body contributes dependencies and further cases;
70 explicit targets and70 successive growth rounds are checked without a small
case/iteration cap. Original bound receiver expressions are also included as
executed dependencies in Go metadata. Scoped source checks pass; receiver-once/
nil/callee-contract runtime proofs and complete capture/overlay/context authority
remain due. R01/R06/W01 stay open, Go/Rust only, and the T02 gate stays final W04.

The [original source-overlay checkpoint](verification/source-overlays/README.md)
reads exact captured sidecar bytes, preflights all70 closed expression heads and
binds original package/function/value-receiver selectors, complete named entry
error paths and exact physical loops. It connects explicit entry overlays to
finite source selection without creating capture-dependent binding/contract IDs.
Thirteen actual source/sidecar pairs,70 input slots and source budgets are checked.
Expression typing, full logical/header namespaces and whole-module optional
sidecar/default contract reconstruction remain due; R01/R06/W01 stay open.
Go/Rust only, with the T02 full gate still deferred to final W04.

The [original contract-expression checkpoint](verification/contract-expressions/README.md)
adds internal type derivation for all70 source heads, actual lexical header
bindings, fresh logical binders and complete entry/return/header heap snapshots.
Contract-only source types retain the same Root/Plain checks and global U;
typing does not discharge partial-operation safety. Six new/20 unique affected
old Rust roots and4 original native functions/10 headers/58 exact bindings pass,
with scoped Clippy/vet/formats and complete original-input records. Canonical
capture-dependent scope/contract/context IDs and whole-module callee/default
contracts remain due, together with all runtime/source/program proofs and
R01/R06/W01 completion. Go/Rust only; the T02 full gate stays final W04.

The [original function-contract module checkpoint](verification/contract-modules/README.md)
keeps selected roots, reached callees and optional captured sidecars distinct.
Every body receives its actual annotation or an explicit partial/true/all-same-U
default with no raw overlay ID. Callee annotations extend finite Error closure;
generated invariants/range ranks remain unproved candidates and missing total
for ranks remain pending. Six new/31 affected old Rust roots and17 current Go
cases pass, including70 annotation-driven additions and512 helpers. Original
bytes/modes and the initial source epoch remain retained. Canonical scope-set/
contract/context authority, complete source/runtime/program proofs and
R01/R06/W01 completion remain due. Go/Rust only; T02 full gate stays final W04.


## Complete original-source value relations

The [source-value checkpoint](verification/source-value-relations/README.md)
derives closed observations from every original same-U source type. Words,
array padding, all string bytes, complete slice descriptors/ghost fields,
ordered structs and every error payload remain. Equivalence, equality lifts,
zero, whole-field assembly/projections and copy/nil/box/active transfers are
ordinary zero-axiom proofs. All prior source-specific identities remain;
unchanged call-site libraries retain their own complete old proofs. No source
case/root/helper/parameter cap or function extensionality is introduced.

Both cores agree on97 distinct full streams:89 mathematical acceptances and8
genuine type mismatches. Final83 normal rows cover5 small,71 large and7
heterogeneous rows;5 original pilot rows remain separate. Independent checks
cover10684 closed definitions/37603 type/field statements/
10538 whole-error statements/22013 actual encoded leaf changes
and identical-zero observations. Complete tag observations remain. A coherent
constant byte view is valid mathematics but fails exact source import and the
independent identity-body oracle.

The missing-projection, whole-product fuel, nil-only function-type and first
control fixture failures retain actual inputs/outputs. The adapter audit
checks100 source-type choices; only nil-only error changes. Fixed isolated
sources pass the affected small/first-control checks, four complete-library/
boundary byte replays and literal final worktree binding. The initial control
suite's two passed roots remain; only its failed first root reruns. Scoped
Clippy/format pass. Maxima2156 declarations/33228 terms/
806470 bytes fit unchanged caps. All34 source epochs/
24819 original source rows/379 Go prelaunch rows
remain. Native verification is never restarted after an observation timeout.

The remaining active multi-language roadmap routing is retired. Complete old
plan bytes and the intervening retirement notice are preserved as historical
records; current routing leads only to Go/Rust. No other-language test or
workflow runs.

These relations are not Go comparison, source/input validity or program
correspondence. Full validity/execution/callee/result/effect/frame/context,
full56/private registry/nativeLinux/all169/T02-T11 remain due. Prior historical
scope findings and semantic binary handling remain for the final strict gate.
Go/Rust only; R01/R06/W01 stay open, the practical profile stays inactive and
the cumulative T02 gate remains final W04: `./scripts/check-fast.sh`.


## Original-source bounded string adapters

The [string checkpoint](verification/source-strings/README.md) preserves every
original source type/U and adds builtin/nominal string adapters, whole
roundtrips, original zero/validity and constructor/guarded byte observations.
The source string adapter checkpoint preserves all original source types and complete error universes in eight normal rows: nil-only, entry, heterogeneous, all70 cases, and every builtin/nominal string type in a fresh four-case original-Go unit. Whole source/math roundtrips, source-zero equality and validity, full64-bit length/all256 byte-slot observations, constructors and guarded byte observation laws are ordinary checked proofs. Both unchanged full cores agree on13 streams: 9 zero-axiom mathematical acceptances and4 genuine type rejections. A coherent constant-length adapter is accepted as mathematics but fails exact source import and the independent original-body oracle. Independent checks cover480 original source recipe members and50688 actual length/data-bit observations. Maxima are859 declarations/17188 terms/256542 bytes under unchanged caps.

These are private representation adapters. They do not establish Go len result/int index typing, source expression evaluation, arbitrary runtime-input validity, source error dispatch safety/effects/frame, canonical capture/context authority or profile activation. The full56 foundation, private registry, nativeLinux/all169 and remaining T02-T11 work stay due. Go/Rust only; R01/R06/W01 remain open. The cumulative T02 gate stays final W04: ./scripts/check-fast.sh.


## Original-source integer string indices

The [index checkpoint](verification/source-string-indices/README.md) retains every original source type/U and prior string identity.

The original-source string index catalogue binds every eligible builtin/nominal integer width and sign to full unsigned zero extension, actual balanced-Int interpretation and guarded source-uint8 reads. Definedness consumes the original strong string validity, actual nonnegative sign and complete mathematical index<size; no high bits are truncated. All original source types/U and every prior source-string declaration identity remain. Normal131 rows cover all71 integer types in the70-case unit,4 heterogeneous types and all56 pairs of4 string types with14 integer types in the new original-Go unit. Both unchanged full cores agree on135 complete streams:132 zero-axiom mathematical acceptances and3 genuine type mismatches. Independent comparison checks2911 new complete definitions/statements and20313 original source recipe members, with1622720 full64 conversion-bit and172072 full-byte bit observations. A coherent constant read is accepted as mathematics but fails exact source import and the independent body oracle. Original Go observations independently match all13 operand types,4 string types,4 full cases and the extra original numeric error case; aliases retain identity. Maxima1247 declarations/23275 terms/365232 bytes fit unchanged caps.

These are private typed representation/definedness adapters, not ordinary proof of the original IndexExpr evaluation, operand-once ordering, signed Go len result, arbitrary runtime-input validity, source effects/frame or canonical capture/context. The full56 foundation/private registry/nativeLinux/all169 and remaining T02-T11 requirements remain due. Go/Rust only; R01/R06/W01 remain open and the practical profile is inactive. The cumulative T02 gate remains final W04: ./scripts/check-fast.sh.


## Original-source Go int string lengths

The [length checkpoint](verification/source-string-lengths/README.md) preserves every original source type/U and prior string identity.

The original-source string length catalogue returns the actual source builtin int carrier, preserving the entire64-bit header and balanced signed Int interpretation. It neither requires source uint8 nor fabricates a missing source int. Real string length-fit/validity proofs imply a clear sign bit and whole Int equality to the natural length, using unbounded Nat order and a balanced2^63 threshold without truncation, bounded length unrolling, axioms or changed caps. Every original source type/U and prior source-string declaration identity remains. Normal12 rows cover all4 builtin/nominal string types in each complete original0/4/70-error-case scope. Both unchanged full cores agree on16 streams:13 zero-axiom mathematical acceptances and3 genuine type mismatches. The accepted extra theorem still fails exact source import and the independent export-set oracle. Independent comparison checks204 source members,168 generic public statements,240 closed polynomial checkpoints and1188 original source recipe members, with289536 full64-bit observations. Original Go independently confirms4 dynamic int-returning len calls and all12 rows' original identities/U. Maxima1319 declarations/24753 terms/395672 bytes fit the unchanged4096/131072/2097152/1024 caps.

This private typed representation/length bridge does not establish original CallExpr evaluation, operand-once execution, arbitrary runtime-input validity, source effects/frame or canonical capture/context/program correspondence. Complete56 foundation coverage, all169 demands, native Linux release gates and remaining T02-T11 work remain due. R01/R06/W01 remain open and the practical profile is inactive. The cumulative T02 gate remains final W04: ./scripts/check-fast.sh. Go/Rust only; no removed-language test or release prerequisite.


## Original-source string operation bindings

The [operation checkpoint](verification/source-string-operations/README.md) binds every selected original string operation to its actual type-specific mathematical recipe.

The catalogue now derives string len/index sites from every original checked selected body and binds each private certificate to the complete original source buffers, package/root set, all source types/U, physical operation and operand nodes, actual result type and every contained ordinary call/callee. It distinguishes predeclared len from an ordinary function named len, arrays/slices and typed-int constant lengths. A single captured operand product has exact projections, actual Length/Read results, unchanged validity/index guards and full signed integer consequences. Error-free units retain all original source carriers/zeros without a fabricated source error key; runtime literal indexing retains the actual materialized string key. The error-bearing predecessor row remains byte-identical. Normal25 rows cover11 Length and14 Index instances across complete0/70-case units and separate actual Length/At/Literal selections. Both unchanged full cores agree on28 complete streams:26 zero-axiom mathematical acceptances and2 genuine type mismatches. Cross-site exact import rejects; the extra theorem is valid mathematics but fails exact import and the independent export-set oracle. Independent comparison reconstructs384 site-specific members and3066 original source recipe members, plus the three generic Pair/Uncurry/PairApply declarations in every Index row. Original Go independently confirms all24 physical operation sites, including2 typed-int constant lengths, actual operand/result types, caller and child-callee origins, with full original error universes0/70. Maxima1346 declarations/24859 terms/401578 bytes fit unchanged4096/131072/2097152/1024 caps.

This is original operation/type binding and captured-value mathematics. It does not establish operand expression execution, operand-once sequencing, Go panic behavior, callee execution/effects/frame, arbitrary runtime-input validity or canonical source/IR/program/capture/context correspondence. Capture positions are argument positions, not a claim that all Go expression evaluations occur left to right. Constant-length sites remain inventoried with their actual int result; runtime-row elaboration for them remains due. All56 foundation coverage, all169 demands, native Linux release gates and remaining T02-T11 work remain due. R01/R06/W01 stay open and the practical profile stays inactive. The cumulative T02 gate is deferred to final W04: ./scripts/check-fast.sh. Go/Rust only; no other-language implementation, test or release prerequisite is added.


## Independent concrete original-Go execution

The [source execution checkpoint](verification/source-execution/README.md) executes actual selected originals and records concrete values, source faults and finite observation prefixes.

An independent untrusted observer now executes actual original selected Go entries and their checked original callees from owned syntax and the complete source-derived domain. It materializes exact constants, reads actual local bindings, preserves nominal/full-width values, short circuits expressions, executes wrapping/signed arithmetic and shifts, strings/raw bytes/rune conversions, ordered result tuples, value arrays/structs, alias-preserving slices and array views, two-phase/deferred-location assignments, initialized locals/named results, branches/switches/for/range control, ordinary calls and closed error dispatch. Caller roots, callee bodies and methods come from the original checked selection; no caller evaluator hook, native type graph or native result supplies an execution. Source faults, explicit finite observation exhaustion and unavailable/host-capacity strategies remain distinct. Local array views retain their actual storage across writes and return; new declaration/Go1.25 loop-variable cells do not overwrite prior captured cells. This is one source-permitted evaluation/allocation choice, not a universal Go order or allocator theorem.

All127 complete original-Go cases agree on return values, source faults and post-call argument observations:117 normal returns and10 source faults (5 index bounds,1 division by zero,1 slice bounds,1 make bounds,1 negative shift,1 nil interface call). All70 original concrete error types execute their own original Error body. The202 real source invocations retain unique invocation IDs and matched entry/return IDs on normal runs; a shared-state multi-result forwarding call executes once. Full64-bit/nominal values, all raw bytes, invalid UTF-8 conversions,257-byte strings, huge/negative indices and shifts, overlap copy, aliasing, array views, short-circuiting, parallel assignment and source control remain represented. Maximum observed execution length is151 steps; observation budgets0/1/7/19 stop the actual infinite source loop without becoming Go faults or semantic/kernel caps. An independent read of the complete native archive authenticates every original application/module/observer/vector byte and the actual native stdout. The source packet contains all3 original packages, original files plus module metadata and only the original bool seed; native type facts/results do not enter its constructor. The existing source-selection regressions execute3 affected roots, including the full original33-case corpus, and scoped Clippy/Rust/Go format pass.

Concrete observer agreement is not an ordinary checked source semantics, source/IR equivalence, operation theorem, operand-once theorem, callee precondition/result/effect/frame theorem, universal interfering-expression order result, allocation-strategy theorem, arbitrary runtime-input validity, loop termination or canonical capture/context/program proof. The independent source interpreter and future IR/ordinary semantics remain separate. The concrete execution trace must still be bound to the actual ordinary operation roots and all source/program theorem demands. Bounded-model validity is separate from Go execution (the257-byte source string is executed without asserting its256-byte model validity). Full56-family assembly, all169 demands, remaining private descriptor/registry/context/capture work, native Linux release gates and T02-T11 remain due. R01/R06/W01 stay open; the practical profile stays inactive. The cumulative T02 gate is deferred to final W04: ./scripts/check-fast.sh. Go/Rust only; no other-language code, test or release dependency is added.


## Original-source joint invocation interfaces

The [joint invocation checkpoint](verification/source-invocations/README.md) retains every original call, expression group, full signature and possible target.

Every static, method and closed-error call is derived from the complete original checked Go body closure. Original argument expression groups retain all tuple projections into the complete materialized parameter list, including receivers, source/parameter types and untyped nil. Results, actual callee origins, receiver paths and the complete global error target universe remain source-owned. A private ordinary interface couples one unbounded Nat invocation identity, complete evaluated arguments, before state, all results and post state. State and invoke remain universally quantified. CaptureCall has one completion binder and one invoke application occurrence; all five joint-field laws, every result projection, full result packing and the zero-result unit law retain their checked ordinary proofs.

All32 original call interfaces retain every source type/U and all65 input positions/16 heterogeneous results/void calls/method values and expressions. One original expression forwards all16 result positions. Actual Go independently matches all32 callers, physical sites, full signatures, 154 argument groups,184 value projections, nil markers and possible targets in complete zero/70-case units. All32 exact source-site imports and every original predecessor identity are checked without repeating kernel work. An independent oracle reconstructs5760 source recipe members and1276 invocation definitions, law statements and proofs. The complete original named declaration union is checked in two whole certificates by both unchanged real kernels, with zero axioms and identical hashes/counts. Independent full AST comparison checks81420 original type/body/proof and constructor/recursor ownership identities, with no missing or extra declarations. Cross-site import rejects; both kernels genuinely reject the stale-post law, while an accepted extra theorem fails source import and the export-set oracle. Whole-union maxima3492 declarations/129452 terms/1310358 bytes and per-row maxima2946/125036/1210450 fit unchanged caps. This establishes every original ordinary declaration obligation; it does not claim all32 separate original certificate wrappers were individually accepted.

This is a typed joint data interface. An ordinary proof must still bind invoke to the actual original Go callee. Operand execution/once sequencing, callee preconditions/results/effects/frame, state/storage/alias semantics, and source/IR/program/canonical capture/context correspondence remain due. Full56-family assembly, all169 demands, private descriptor/registry integration, native Linux release gates and T02-T11 remain due. R01/R06/W01 stay open; the practical profile stays inactive. The cumulative T02 gate is deferred to final W04: ./scripts/check-fast.sh. Go/Rust only; no other-language implementation, test or release dependency is added.


## Owned original-source input handoff

The [source-input checkpoint](verification/source-inputs/README.md) connects the intact native package owner to the strict independent source reader.

The intact original-package owner now produces source-only analysis inputs in production code. Every non-intrinsic original export package contributes its complete active original Go source set and exact module manifest, including unselected files and bodies. Returned bytes and objects are independently owned, remain intact after owner/image/preparation retirement, and cannot mutate the original universe. A private identity couples the complete image/compiler/main/requested-package provenance to the source component. The existing native source-unit test helper delegates to this producer. The Rust source-only importer requires the complete package/source set and the fixed logical Bool seed, rejecting native declarations/type/alias/underlying hints. Actual source types are independently derived without a new source type/argument/result/case/root limit.

Actual pinned original Go compilation and immutable original-package reconstruction pass for both unchanged zero/70-error-case units and a real local replacement selected package whose own module differs from the main module. The replacement retains its unselected channel/callback and init-panic bodies; no application initializer executes. The complete production input bytes for both original units equal the prior source-only packets. Independent source analysis derives25/95 actual types, all65 argument positions,16 results and complete zero/70-case universes. All32 entire original proof certificate byte streams replay literally, preserving the previously checked complete named-union proof evidence. The current32 native call/caller/group/signature observations also equal the prior complete observations. All four new Rust roots pass across the original three successful roots and the isolated corrected control rerun. Coherent generic-valid native uint16 data rejects at the source-only boundary; incomplete package roots already reject at the shared Reference boundary, and missing/changed original bytes reject. Closed/incomplete Go owners and mutations of every returned source byte/object reject or remain isolated. Scoped strict Clippy, Go vet and changed-file Rust/Go format checks pass. Unchanged kernels and large prior proof matrices are not repeated.

These are privately owned source-only candidates and provenance identities, not authenticated installed capture contexts, source admission, callee execution/precondition/result/effect/frame proofs, operand-once/order proofs or source/IR/program correspondence. Complete context/descriptor/private registry/capture and foundation assembly, all169 theorem demands, native Linux release gates and remaining T02-T11 work remain due. R01/R06/W01 stay open and the practical profile stays inactive. The cumulative T02 gate remains final W04: ./scripts/check-fast.sh. Go/Rust only; no removed-language implementation/test/release prerequisite is added.


## Execution histories and source-owned entry environments

The [ordinary history component](verification/execution-histories/README.md)
proves all-prefix safety and partial-correctness laws over unbounded Nat
histories, retaining every transition edge and requiring a separate normal-run
witness. It remains generic mathematics; closed Go/IR transition binding stays
due.

The [source entry component](verification/source-entries/README.md) derives all
168 original function entry layouts and shares them between the actual source
interpreter and ordinary initialization equations. Actual named inputs receive
their corresponding argument, named results receive source-derived zero values,
and locals remain absent until declaration. Blank/unnamed positions create no
cell, while all argument/result positions and original binding identities remain.
The complete source/type/zero-or-70-case domains retain65 inputs and16 results.
All three Rust roots, complete native comparison,127-case interpreter comparison,
independent row/part reconstruction, scoped Clippy and format checks pass.
The whole wide mathematical union retains all4,388 declarations in two complete
parts, each containing the entire original source/type/U foundation. Both
unchanged full checkers agree on all13 actual streams:8 zero-axiom acceptances
with identical hashes/counts and5 genuine type rejections. All original failed
and interrupted attempts remain retained separately from successful checks.

These are entry-value equations, not a complete program certificate, source/IR
correspondence or storage/frame/callee/entry-validity proofs. Full56-family
assembly, all169 demands, private context/capture/descriptor/registry integration,
native Linux gates and remaining T02-T11 remain due. R01/R06/W01 stay open,
the practical profile stays inactive, and the cumulative T02 gate stays final
W04: `./scripts/check-fast.sh`. Go and Rust remain the only source languages.


## Owned source and contract handoff

The [contract-input checkpoint](verification/contract-inputs/README.md) joins
the original source-only packet and every captured raw sidecar under one owned
package universe. The strict independent consumer uses that exact source and
annotation set, retains unselected annotations and derives explicit defaults
when no raw sidecar exists. Complete saved-inventory integrity, original raw
identities, copy/lifetime ownership and frozen byte/count bounds are checked.
All14 affected Rust roots and6 actual Go roots pass, with independent complete
original-byte/hash/native-membership inspection and scoped lint/vet/formats.
The original failed collection epoch and all failed/final raw snapshots remain
losslessly preserved. Self-hashed input provenance does not authenticate a
launcher or installed context. Full foundation/program/context work and
R01/R06/W01 remain open; the cumulative T02 gate remains final W04.


## Original file language versions

The [source-language component](verification/source-languages/README.md)
derives exact module requirements and per-file effective languages from the
complete original bytes. Owned contract derivation checks every selected and
transitive body against Go1.22-Go1.25 while retaining older unused files.
All150 pinned Go parser/types comparisons,11 affected Rust roots,2 native
Go roots, strict Clippy/vet and changed-file formats pass. Both complete
zero/70-error units and all32 original call-proof bytes remain unchanged.
Full original data and failed attempts are losslessly retained. Complete
module/build/context/source-IR/program authority remains due; R01/R06/W01
stay open and the practical profile stays inactive. The cumulative T02 gate
remains final W04. Source-language support remains Go/Rust only.


## Owned physical function-root inputs

The [physical-selection handoff](verification/selection-inputs/README.md)
binds original package/source/full function byte intervals to the complete
owned source/sidecar set. Independent parsing supplies the node identities;
contract construction accepts no replacement selectors. Same-name receiver
methods remain distinct; package init functions reject while ordinary methods
named init are admitted. All22 affected Rust roots,8 actual Go roots/2 subtests,
strict lint/vet/formats and legacy compilation pass. Both unchanged complete
zero/70-case source units retain all43 roots each and all32 call-proof bytes.
The exact8192-function boundary passes without shrinking source or proof scope.
Full original files, inputs and actual observations remain losslessly retained.
This is input/contract-candidate linkage; canonical context and program proofs
remain due. R01/R06/W01 stay open, the practical profile stays inactive and the
cumulative T02 gate stays final W04. Go/Rust only.


## Complete original snapshot inputs

The [snapshot/capture component](verification/snapshot-inputs/README.md)
connects all original captured objects and observed native namespace bytes/type
bits to the owned physical source/sidecar selection. The independent reader
reconstructs the exact original snapshot digest and distinguishes observed
absence from uncaptured files/directories. Original Windows/test-excluded Go
files and auxiliary build inputs are now captured before immutable package
replay; the original failed fixture remains intact. All9 affected Rust roots,
15 successful native Go roots/71 subtests, strict lint/vet/formats and legacy
compilation pass across the retained initial and repair suites. Both complete
zero/70-case original units and all32 call-proof bytes remain unchanged.
Private input linkage supplies no installed capture/context or program proof.
R01/R06/W01 stay open; the profile stays inactive and the cumulative T02 gate
remains final W04. Go/Rust only.


## Complete snapshots in another checkout

The [portable snapshot component](verification/portable-snapshots/README.md)
fixes restoration after checkout relocation while preserving original absolute
origins as immutable provenance. All77,162 original files and696,793,808 bytes
restore with exact relative paths and modes. Both actual Rust consumer binaries
are recompiled in the new directory and all9 roots pass from zero pre-restored
snapshot files. Both complete original source units,43 roots each, zero/70-error
universes and all32 proof byte streams remain unchanged. Fifteen focused path,
corruption and link controls pass. Original failures and every consumed helper
epoch remain retained. This host check does not establish native Linux release
execution or canonical capture/context authority. R01/R06/W01 stay open; the
profile stays inactive and the cumulative T02 gate remains final W04. Go/Rust only.


## Independent fixed-target source eligibility

The [build-constraint component](verification/build-constraints/README.md)
checks every claimed active Go source against original Go1.25.0 Linux/amd64
filename and modern/legacy header rules. Complete snapshots retain every excluded
source. Actual Go/Rust results agree on182 boundary cases and all13,853 Go source
candidates across the three unchanged full original snapshots. All18 known OS
and24 architecture names are covered against exact pinned source witnesses.
All15 affected Rust roots and both actual Go oracle roots pass, with strict
Clippy/vet/formats. Four coherent complete rebinding controls reject, while both
43-root original units, zero/70-error universes, all150 language observations
and every byte of32 proof streams remain. Full import/cgo/package membership,
canonical module/compiler/export/context authority and program proofs stay due.
The new test reader also passes both standalone roots after restoring every
original snapshot from zero files in another checkout. Its actual old missing-
snapshot failure stays preserved; final affected Clippy/format checks pass.
R01/R06/W01 remain open; the profile stays inactive and the cumulative T02 gate
remains final W04. Go/Rust only.


## Original package Go files and headers

The [package-file component](verification/package-files/README.md) reconstructs
original Go-file membership, package names and imports/internal-or-external-test/
cgo-disabled groups from the complete immutable namespace. Complete capture
rejects omission of a real unused source even when selected contracts still
derive. All39 native header cases and all2,003 original package directories
agree. All24 affected Rust roots pass, including the full5,853 lexer/2,996
parser corpus, original43-root units, zero/70-error universes and32 exact proof
streams. Strict Clippy/vet/formats pass. Initial fixture-root and generator
failures retain their exact original sources/outputs; only affected checks rerun.
Auxiliary/directive/embed selection and canonical module/compiler/export/context/
root/program authority remain due. R01/R06/W01 stay open, the profile stays
inactive and the cumulative T02 gate remains final W04. Go/Rust only.


## Original auxiliary build inputs

The [auxiliary component](verification/auxiliary-inputs/README.md) reconstructs
all nine original Go auxiliary build-input classes and ignored-other files from
the complete immutable namespace and exact buffers. All178 native cases and
all2,003 original package directories agree. There are318 selected auxiliary
files and1,600 ignored-other inputs across the three full original universes.
Missing/reclassified-buffer controls reject without replacement inputs. All18
affected Rust roots and both Go oracle roots pass, with strict Clippy/vet/formats,
unchanged43-root/zero-or70-error universes and32 exact proof streams. This adds
no other program-language support. Cgo directives/embed and canonical module/
compiler/export/context/launcher/program authority remain due. R01/R06/W01 stay
open; the profile stays inactive and the cumulative T02 gate remains final W04.
Go/Rust only.


## Original cgo-disabled documentation

The [cgo documentation component](verification/cgo-directives/README.md)
validates original import-C comment attachment, text, conditions, arguments,
source-directory safety and native directive forms while cgo stays disabled.
All88 native cases agree, including31 rejections, and all2,003 original package
directories/66 cgo-disabled files remain intact. A complete5,486-object native
owner preserves its excluded cgo source; changing only that source's directive
rejects coherently resealed capture with the identical source selection and two
contract candidates. All21 affected Rust roots and three native Go roots pass,
with strict Clippy/vet/formats, original43-root/zero-or70-error universes and32
exact proof streams. No C program/build hook executes. Embed selection and
canonical module/compiler/export/context/launcher/program authority remain due.
R01/R06/W01 stay open; the profile stays inactive and the cumulative T02 gate
remains final W04. Go/Rust only.


## Original embedded resource capture

The [original resource component](verification/embedded-inputs/README.md)
binds original non-test embed patterns and exact resource identities to actual
pinned Go package metadata. Sealed replay uses only complete captured names and
buffers after physical roots are removed. All54 native cases agree:27 accepted
and27 rejected, including hidden/all/glob/name/module/symlink behavior. Every
successful and failed original input and native output remains complete. A full
5,535-object native owner retains four application assets, arbitrary binary
bytes and independent inventory ownership; missing buffer/content/directory
controls reject. All seven Go roots/28 subtests and13 Rust roots pass, with
strict Clippy/vet/formats and legacy compile-only checks. Rust independently
imports every original resource byte; independent Rust pattern/file selection
remains due. Original43-root/zero-or70-error units and32 proof streams remain
intact. Ten source epochs preserve all consumed originals and exact outputs.
Canonical module/compiler/export/capture/context/launcher/root/program authority,
full56 foundation/all169 demands, native Linux gates and remaining T02-T11
stay due. R01/R06/W01 stay open; the profile stays inactive and the cumulative
T02 gate remains final W04. Go/Rust only.


## Independent original embedded resources

The [independent resource component](verification/embedded-selection/README.md)
reads original Go source pattern bytes and reconstructs production resource
identities from the complete immutable namespace. Required missing buffers or
unobserved directories reject coherently resealed capture with the same original
Root contract. All54 native resource cases,54 pragma cases,73 native glob cases
and all2,003 unchanged original package directories agree. Three actual native
groups keep internal/external test patterns distinct from normal production
resource selection, including invalid and missing test-only patterns. The entire
5,535-object original owner/four assets remains intact. Five native Go roots,
26 affected Rust integrations and one native-glob unit pass, with strict affected
Clippy/vet/formats. All240 Rust/187 Go production sources and the two complete
prior test-source prefixes stay unchanged when adding the group tests. Both43-root
units, zero/70-error universes and32 proof streams remain byte-exact. All22
consumed source epochs and exact outputs stay complete. Canonical module/compiler/
export/capture/context/launcher/root authority, missing module-link target/stat
facts, compiler pragma validation, full56 foundation/all169 source/program proofs,
native Linux gates and remaining T02-T11 stay due. R01/R06/W01 remain open;
the profile stays inactive and the cumulative T02 gate remains final W04.
Go/Rust only.


## Whole original compiler/export input linkage

The [original export component](verification/export-inputs/README.md) binds the
whole actual Go compiler output and every captured archive to original source,
auxiliary/resource and import membership. Original SDK vendor resolution, exact
selected-root/physical-origin correspondence and actual archive bounds/target/
build IDs retain complete bytes. The HTTP/embed owner keeps180 packages/179
exports/5,663 objects/164,865,655 bytes with unchanged unused unsupported body
and unexecuted initialization. All22 affected Rust roots,21 metadata and four
fully resealed archive controls pass. The new native root passes; an output-
directory collision blocks the next ownership test before execution, then the
same test passes in a separate evidence directory. Both attempts stay complete;
the successful export root is not repeated. Strict Clippy/vet/formats pass,
with both43-root units/zero-or70-error universes/all32 proof streams intact.
All ten consumed source epochs and exact outputs remain preserved. Compiler-image/
module/root/context/registry authority, full source/export/program/foundation
proofs, native Linux gates and remaining T02-T11 stay due. R01/R06/W01 remain
open; the practical profile stays inactive and T02's full gate stays final W04.
Go/Rust only.


## Original module observation correspondence

The [module observation component](verification/module-observations/README.md)
retains whole original resolver output/diagnostics and every selected manifest
under the same lock as source/export capture. Rust binds those facts to original
module/Go/replacement tokens, exact-before-wildcard lookup, local longest matching
capabilities, versioned cache spelling and every application package's original
module/source/version/replacement identity. Ignored main and duplicate unused
directives remain intact. Duplicate native fields reject before projection.

The actual local owner retains181 packages/180 exports/5,667 objects/164,737,170
raw bytes. The versioned owner retains two packages/exports/5,490 objects/
85,379,438 raw bytes, original go.sum and complete mod/info/ZIP/ziphash buffers.
The original HTTP/embed owner remains180 packages/179 exports/5,663 objects/
164,865,655 raw bytes. Unused HTTP bodies and initialization remain unexecuted.
All three native roots and25 affected Rust roots pass, including24 resealed
module/package controls over the same original capture and one Root contract.
Existing21 metadata/four archive controls, both43-root units, zero/70-error
universes and32 proof streams stay intact. Strict affected Clippy/vet/formats,
independent inspection and all eleven full source epochs/outputs are retained.

Independent MVS, full modfile/module identity/version/graph/ZIP/sum/root/compiler/
context/descriptor/registry authority, source/export/IR/state/storage/frame/callee/
program proofs, all56 foundation families/all169 demands, native Linux gates and
T02–T11 remain due. R01/R04/R05/R06/W01 stay open and the practical profile stays
inactive. The cumulative T02 gate remains final W04: `./scripts/check-fast.sh`.
Go/Rust only; no premise, source/type/error/proof scope, review rule or budget changes.


## Independent original module data

The [module-data component](verification/module-data/README.md) independently
parses every original main go.sum line and checks canonical module/version/h1
identity, including unused entries and identical duplicates. Whole original ZIP
headers/namespace/content retain Go1.25 ZIP64, prefix, descriptor, directory/CRC,
last-name hashing and Unicode15 folding/filename dispositions. Streaming Hash1,
unchanged500MiB expansion/16MiB manifest-license/131072 namespace limits, cached
manifest/info-version/ziphash and every captured expanded source remain bound.
No member is extracted into a loader filesystem or supplied by a caller adapter.

All14 native Go roots,29 affected Rust integration roots and one whole-scalar
unit pass. All1291 sum/39 ZIP verdicts and accepted inventories agree; all1112064
Unicode scalars and both full protocol boundaries remain. Eleven fully resealed
controls retain the same source selection and one Root contract: eight reject
changed contents/missing data and three valid duplicate/encoding/comment variants
accept. The original first-bit failure changed only DEFLATE encoding; its whole
source/output epoch and independent complete reconstruction remain. A separate
midpoint case changes real member content. Production validation is unchanged.
The initial missing u64 counter annotation failure also remains complete.

Both43-root units, zero/70-error universes and32 proof streams stay exact. Complete
cached dependencies match their whole source archives and Cargo.lock, with no
network fallback or unrelated upgrades. Independent Python ZIP/DEFLATE Hash1
inspection, every original raw module buffer/control identity, all24 input epochs,
strict affected Clippy/vet/formats and exact outputs are retained. No review rule/
seal/exclusion, premise, math rule, kernel cap or source/type/error/proof scope
changes. Host checks confer no native Linux release acceptance.

General MVS/full module/native metadata semantics, checksum database and root/
compiler/capture/context/descriptor/registry/launcher authority, source/IR/state/
storage/frame/callee/program proofs, all56 foundation/all169 demands, native Linux
gates and remaining T02–T11 stay due. R01/R04/R05/R06/W01 remain open; the practical
profile stays inactive. The cumulative T02 gate remains final W04:
`./scripts/check-fast.sh`. Go/Rust only.


## Independent original module directives

The [manifest component](verification/module-manifests/README.md) derives all
ordered require/exclude/replacement/retraction facts, module identity and exact
Go/language/toolchain policy from owned original bytes. Full decimal version
normalization, prerelease/build/path-major rules, duplicate/order retention,
byte syntax, native Unquote behavior and original non-UTF8 compiler-name suffixes
are independent of caller graphs or source adapters. Original retraction strings
retain the pinned parser's intentional non-normalization. Existing original
module/source/export correspondence consumes this reader and exposes all five
actual selected manifests for subsequent graph reconstruction.

All656 main/dependency native verdicts and facts agree:162 accept/494 reject.
The first652 full raw sources/verdicts remain exact; four additional raw toolchain
suffix cases retain both modes. Ten affected Rust roots, three native Go roots
and an additional updated-oracle execution, current strict Clippy/vet/formats
pass. Initial unused test import and redundant Result-match failures retain exact
source/output epochs. Local181-package/5667-object and versioned2-package/5490-
object universes, every source/export/cache/archive and unused HTTP body remain.
No original initialization or unsupported application body is executed.

Independent inspection checks all observations and original raw owner buffers,
prior preimages and unchanged32 proof bytes. Earlier43-root/zero-or70-error/source/
resource/language/foundation/protocol matrices keep full evidence without new
replay claims. Complete consumed input/output epochs and formatter preimages
remain. Explicit-path formatting preserves every actual input; functional
consumers retain their full source/package universes. No scanner rule/seal/
exclusion, source/type/error/proof domain, premise, math rule or kernel cap changes.
Host checks confer no native Linux release acceptance.

General MVS/pruned/unpruned graph reconstruction and complete graph inputs,
compiler/root/capture/context/descriptor/registry/launcher authority, source/IR/
state/storage/frame/callee/program proofs, all56 foundation families/all169
demands, native Linux gates and remaining T02–T11 remain due. R01/R04/R05/R06/W01
stay open; the practical profile stays inactive. The cumulative T02 gate remains
final W04: `./scripts/check-fast.sh`. Go/Rust only.


## Original native module graph inputs

The [graph component](verification/module-graphs/README.md) retains the controlled
original Go1.25 graph/diagnostics and original .mod inventories for initial and
final roots. Unselected old/empty leaves, identity/pruning-state traversal,
cycles, duplicate order, exclusions, exact replacements and source-free virtual
nodes remain intact. Pruned unread targets do not acquire fabricated originals.
Immutable source rebinding and a same-lock original module/source/export graph
handoff preserve the whole181-package/5667-object HTTP owner after retirement.
Native final roots remain observations; independent Rust ParseLax, root expansion,
MVS and a complete transient native input trace remain due.

All five new/15 affected existing native roots, final vet/formats and actual-SDK
Go1.23.2 compile pass. Seventeen complete consumed source epochs and exact outputs,
full pinned SDK originals, failed filename/root-expectation evidence and corrected
same-case raw generated trees/child diagnostics remain. The inconsistent root case
retains Go's readonly rejection; its separate expanded graph grants no owner.
The initial child diagnostics/generated tree serialization gap and superseded
legacy SDK receipt are explicit in the component record. Inspection verifies
raw object/hash linkage and unchanged32 proof streams without replaying those
proofs or unchanged Rust consumers. All previous source/package/error/foundation
universes remain. R01/R04/R05/R06/W01, full56 foundation/all169 source/program
requests, compiler/root/context/registry authority, native Linux gates and T02–T11
stay open. The practical profile stays inactive. The cumulative T02 gate remains
final W04: `./scripts/check-fast.sh`. Go/Rust only.


## Independent original dependency manifest parsing

The [dependency manifest component](verification/graph-manifests/README.md)
adds the independent Rust ParseLax disposition with complete original bytes,
optional unchecked module paths, ignored main-only/unknown blocks, require/
retract/ignore facts and exact lax Go-version behavior. Decimal strings retain
full version widths; syntax and semantic Go-version validity stay separate.
Pruning/implicit go/compiler-availability facts and ordered requirements agree
with original initial/final graph buffers, including old/empty unselected inputs.
The full181-package/5667-object HTTP source/export/module owner remains intact.

All779 native/Rust verdicts and full fact rows agree:376 accept/403 reject,
including every earlier656 source/mode row and123 new cases. Eight original
parser-library files match the complete pinned SDK byte for byte. All11 affected
Rust roots, native oracle, strict Clippy/vet/formats and two strengthened original
requirement comparisons pass. Eleven complete source epochs and exact outputs
remain. Original32 proof streams and earlier43-root/zero-or70-error universes
retain prior evidence without new proof replay. Independent complete graph
transport, root expansion/MVS, transient native input and compiler/root/context/
registry/source/program/foundation authority, native Linux gates and remaining
T02–T11 remain due. R01/R04/R05/R06/W01 stay open; the practical profile stays
inactive. The cumulative T02 gate remains final W04: `./scripts/check-fast.sh`.
Go/Rust only; no scope, premise, math rule, scanner policy or kernel cap changes.


## Original graph cache and byte identities

The [v2 graph input component](verification/graph-inputs/README.md) binds the
actual native module cache to its saved capability/directory during collection
and immutable replay. Its private handoff carries that location and original
version bytes as hex, preserving non-UTF8 toolchain suggestions. Distinct labels
with equal semantic versions receive a deterministic raw-byte inventory tiebreak;
native graph output is not changed. Historical v1 records remain exact.

All five native graph roots, vet/formats and a wrong-original-cache control pass.
The full181-package/5667-object HTTP owner keeps its unused bodies/init and an
original toolchain suffix ending in ff through source/image/owner retirement.
Both implicit/plain and equal-version custom labels remain exact and ordered.
All four input epochs, generated originals, whole native diagnostics and unchanged
SDK evidence remain. Inspection checks v2 hash/cache/namespace/byte correspondence
and unchanged32 proof streams without new proof/Rust replay claims. Independent
Rust graph import/MVS, transient input and compiler/root/context/source/program/
foundation authority, native Linux gates and T02–T11 remain due. R01/R04/R05/R06/W01
remain open; the profile stays inactive. Final W04 owns the T02 cumulative gate:
`./scripts/check-fast.sh`. Go/Rust only; no scope, premise, rule or cap changes.


## Independent original graph input import

The [graph import component](verification/graph-import/README.md) independently
reconstructs initial/final loaded dependency summaries from original main
requirements, excludes, exact replacements, cache/origin/namespace data and
h1-checked original .mod files. Identity/pruning-state traversal retains old lower
and empty versions, cycles, duplicates and unpruning while avoiding invented
pruned sources. Complete v2 inventories, native breadth-first byte output and
observed real-module maxima agree with original module metadata. Arbitrary
virtual version bytes and all original module/source/export/archive data stay
owned. Final roots remain native observations pending root expansion/MVS.

All15 affected Rust roots, full native versioned capture, current affected Clippy,
vet and formats pass. The local owner retains181 packages/5667 objects with its
non-UTF8 toolchain suffix; the new versioned owner retains182 packages/5697
objects, eight selected modules and13 initial/final loaded nodes. Both keep
unused HTTP bodies and unexecuted init after root/image retirement. Twenty
resealed graph controls reject. A separate changed unselected .mod control
reseals object/snapshot/capture/export/module/graph hashes with the same complete
source/package/one-Root-contract universe; unchanged original go.sum rejects.

Initial empty-slice type inference and test err/expect Clippy failures retain
exact inputs/outputs. All12 source epochs, formatter preimages and whole original
SDK/cache inputs remain. Only the new checksum root/changed test lint rerun after
14 runtime roots pass. Inspection checks all original owners, controls and32
unchanged proof byte streams without new kernel replay; earlier43-root/zero-or70-
error universes keep prior evidence. General root expansion/MVS, transient input/
transport-resource, compiler/root/context/registry/source/program/foundation
bindings, native Linux gates and T02–T11 stay due. R01/R04/R05/R06/W01 remain
open; the practical profile stays inactive. The full T02 gate stays final W04:
`./scripts/check-fast.sh`. Go/Rust only; no source/type/error/proof scope, premise,
math rule, review rule/seal/exclusion or kernel cap changes.


The [source-derived root checkpoint](verification/root-expansion/README.md)
implements native bootstrap normalization, iterative pruned root upgrades and
complete transient original manifest retention in preparation and independent
Rust import. All18 native cases,225 version comparisons and84 exact sort
permutations agree with Rust. Eight affected Rust roots and8 native roots pass,
including the fresh182-package/5697-object HTTP owner roundtrip;22 coherently
sealed root/graph claims, the original checksum mutation and a missing transient
manifest reject. Initial duplicate-root and new-test format failures retain all
inputs and outputs among20 epochs. Production/integration Clippy, vet and formats
pass. Full algorithm/source/read/resource and remaining W01/proof authority stay
open; the profile stays inactive. No original source/type/error/proof universe,
review rule, premise, math rule or kernel cap changes. The full T02 gate remains
final W04: `./scripts/check-fast.sh`. Go/Rust only.


The [original observation byte checkpoint](verification/observation-json/README.md)
retains complete native output through private JSON at the existing16MiB stdout/
1MiB stderr budgets. Only two root hex fields receive derived32MiB/2MiB bounds;
common Go/Rust codecs, schemas/domains/hashes and all source/kernel/certificate
limits stay unchanged. All30 original byte/hash witnesses agree. The new actual
HTTP owner retains182 packages/5697 objects and15000 original duplicate edges,
with package586034-byte and graph630780-byte native streams above the former
ceiling. All15 affected Rust roots and7 native roots, Clippy/vet/formats pass.
All16 source epochs retain three setup/coverage/budget-comparison failures;
no original case or universe is narrowed. Full correctness/other resource/read/
compiler/root/context/source/program/foundation and remaining W01 authority stay
open; the profile stays inactive. Final W04 owns `./scripts/check-fast.sh`.
Go/Rust only; no scanner rule, premise, math rule or kernel cap changes.


The [whole compiler input checkpoint](verification/compiler-inputs/README.md)
owns all5577 actual prepared SDK files/163457770 bytes from the existing verified
image after source/image retirement. Rust independently binds full inventories,
component partition/projection, original exports, Go1.25 witnesses and every captured
SDK source byte, with existing bundle budgets separate from unchanged source caps.
One actual compiler file exceeds16MiB; the original HTTP owner still retains182
packages/5697 objects. All10 affected Rust roots and2 native roots, strict Clippy,
vet/formats pass across9 complete epochs. Thirty coherent claims/raw-file controls
and a coherently rehashed unused SDK-source comment mutation reject without changing
application source or its Root contract. Full build-info/source-role and required
compiler/root/context/native-read/source/program/foundation bindings stay open;
the profile remains inactive. All prior source/error/proof scopes and review rules
remain. Final W04 owns the cumulative T02 gate: `./scripts/check-fast.sh`.

The [original executable build-info checkpoint](verification/compiler-build-info/README.md)
independently binds all9 actual owned SDK tools to exact Go1.25/tool/host/settings
facts without executing binaries or trusting producer build-info fields. All187
native observations and12 affected Rust roots pass, with scoped Clippy/vet/formats.
Three fully resealed actual tool-content controls reject at BuildInfo; every original
application/Root contract and the full5577-file SDK/182-package/5697-object owner
remain. All12 epochs retain the initial negative-control selection failure and its
successful repair. Full reader equivalence (including compressed ELF section-name
tables), SDK source roles and remaining compiler/root/context/read/source/program/
foundation/native Linux/T02–T11 work stay due without removing frozen input cases.
W01 remains open and the profile inactive. Prior proof/error/type/source universes,
review policies and all budgets remain. Final W04 owns `./scripts/check-fast.sh`.

The [whole SDK source-role checkpoint](verification/compiler-source-roles/README.md)
independently derives all5480 prepared SDK source roles and715 source directories
from original owned bytes. All11004 raw SDK sources/1017 raw directories and388
boundary probes agree with pinned Go, including a40000-import header and its
original body without an AST-node restriction. All23 selected Rust roots and scoped
native/Clippy/vet/format checks pass;3 coherently rehashed unused role claims reject
at SourceRole after full export reimport. All5577 compiler files and182-package/
5697-object application/Root contract remain. All14 source epochs preserve two
initial visibility compile failures and the repair. Strict standard-module and
remaining source/program/context/installed/native Linux/T02–T11 work stay due;
W01 remains open, the practical profile inactive and final W04 owns the full T02
gate. Every original source/type/error/proof universe, review policy and cap remains.

The [strict standard-module checkpoint](verification/standard-module/README.md)
replaces the SDK index's dependency ParseLax projection with independent general
strict Parse facts and the native std/go/compiler-upper-bound condition. All1427
native observations,20 affected Rust roots and both native roots pass with scoped
Clippy/vet/formats. Four fully rehashed SDK manifest controls preserve the full
5577-file compiler/182-package/5697-object application/Root bytes;3 malformed cases
reject StandardModule and valid complete directives remain accepted after full
export reimport. The13 source epochs retain both initial setup/construction errors
and their repairs. No main/dependency policy, source/type/error/proof universe,
review rule, premise, math rule or limit changes. W01 remains open, the profile
inactive and final T02-W04 owns the full T gate; installed/context/read/source/
program/foundation/native Linux and remaining T02–T11 work stay due.

The [original ELF section checkpoint](verification/compiler-elf/README.md) adds
native class/endian/section/name/compression facts and exact ZLIB/Zstandard prefixes.
All 425 native/Rust observations retain187 prior cases,9 actual SDK tools and3
complete original SDK Zstandard streams, including both 4,320,033-byte expanded tables.
All 8 affected Rust roots, both native roots and scoped Clippy/vet/formats pass.
The 13 source epochs preserve the no_std import compile failure and repair. Original
5577 SDK files,182 packages/5697 objects, Root contract and all prior compiler/
executable/source-role/standard-module/content controls remain. No original source/
type/error/proof universe, scanner rule, premise, math rule or budget changes.
General reader/codec equivalence and remaining installed/context/read/source/
program/foundation/native Linux/T02–T11 work stay due. W01 remains open, the profile
inactive and final T02-W04 owns the full T gate.

Its [build-wiring follow-up](verification/compiler-elf/build-wiring/README.md) fixes
seven independent observer/fuzz locks and complete codec source receipts/mirroring.
All seven full offline/locked graphs and actual pinned local Linux/shared-mirror
observer builds pass with fixed inputs. Original bytes, failures and command outputs
remain; these build checks add no proof or native Linux release acceptance.

The [live export-cache checkpoint](verification/export-root-live/README.md)
opens the actual Go-generated cache before export capture and independently
matches 181 original export objects and their complete observed namespace while
that cache is live. Its exact projection into the complete 5,697-object,
182-package owner and all 5,577 SDK files is independently verified; coherent
export-byte and unread-namespace changes reject. All 16,880 original evidence
files are losslessly retained and restoration is verified. Scoped Go/Rust tests,
vet, Clippy and formats pass. The observation does not yet bind both held source
and export roots into one owner or authorize a launcher. W01 and the Go profile
remain open/inactive; the full T02 gate stays deferred to final W04.

The [combined live-root checkpoint](verification/combined-root-live/README.md)
retains both independently matched source and generated-export roots in one
private owner and checks their complete disjoint coverage of the original
5,697-object export snapshot. Two actual live captures retain 182 packages and
all 5,577 registered SDK files; the final one checks the exact export cache is
gone before binding. Coherent missing, crossed and changed observations reject.
The original failed timeout, full inputs, scoped checks and both live epochs are
retained. These observations do not establish installed or launcher authority,
an application theorem or native Linux release acceptance. W01 remains open and
the cumulative T02 gate remains due at final W04.

The [test-only combined compiler checkpoint](verification/installed-combined/README.md)
also binds both held root sets to the private compiler owner using a complete
independently checked SDK snapshot. The real 182-package capture retains all
5,697 source/export objects and 5,577 SDK files; the test binary and original
outputs are restorable. Scoped normal and target-only test-configuration lint,
format and live checks pass. The test snapshot does not establish installed or
launcher authority, source/program proof or native Linux release acceptance;
W01 remains open and the cumulative T02 gate remains due at final W04.
