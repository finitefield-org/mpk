# Go Practical Subset Implementation Work Plan

Scope amendment (2026-10-20): MPK supports **Go and Rust only**. Follow
[the normative language scope](../specs/LANGUAGE_SUPPORT_SCOPE_V1.md).
Java/C# implementation, resumption and release dependencies are retired.
Future common-artifact integration covers retained Go, practical Go, Rust32
and Rust64. All Go proof, compiler-binding and resource requirements remain.
Earlier checkpoint descriptions below are historical evidence of their own
inputs; they do not preserve removed-language product support.


Status: T01 passed its historical 594-test gate. The current scope supports
Go/Rust only; T02-W01 remains in progress. The
[completion audit](../migrations/go-practical/t01-w05/completion-audit.json)
and implementation ledger own the current status.

The W01 [installed compiler checkpoint](../migrations/go-practical/t02-w01/verification/installed-compiler/README.md)
is in progress. A private SDK owner uses the build-pinned registries and the
existing installed-root loader. A test-only sealed copy of the complete registered
Linux SDK binds the original 182-package/5697-object owner; a rehashed wrong-host
claim and a crossed complete Darwin owner reject. The same complete owner now
binds through an actual four-bundle installed root under local Linux user-mode
emulation, with modified SDK, missing namespace and alternate-registry controls.
This is not native Linux release acceptance, and no practical profile is active.

The W01 [analysis build-info checkpoint](../migrations/go-practical/t02-w01/verification/analysis-build-info/README.md)
independently checks the registered Go frontend's compiler, host, main module,
analysis dependencies and build settings. Native Go and Rust agree on 38 cases,
and the affected SDK/ELF/CLI checks, lint and formats pass. The registered source
epoch differs from the current source tree; build-info agreement does not prove
source-to-binary correspondence. Updated actual four-bundle installed-root checks
pass under local Linux user-mode emulation, with no practical context or profile
activation and no native Linux release acceptance.

The W01 [analysis vendor checkpoint](../migrations/go-practical/t02-w01/verification/analysis-vendor/README.md)
independently parses the complete original analysis dependency manifest. Go and
Rust agree on 36 cases, including all 29 original packages and the unchanged
32,768-package boundary. Source/archive and source-to-binary authority remain
open; the practical profile is inactive.

The W01 [analysis archive checkpoint](../migrations/go-practical/t02-w01/verification/analysis-archive/README.md)
checks all 1,751 members in three original analysis ZIPs against 133 supplied
vendor files, preserving the original TODO difference and 213 captured sources.
Go/Rust controls agree. The supplied list does not establish namespace
completeness, source syntax or source-to-binary authority; W01 remains open.

The W01 [analysis namespace checkpoint](../migrations/go-practical/t02-w01/verification/analysis-namespace/README.md)
derives the vendor list from recorded directory entries and requires the
manifest and archives from the same snapshot. Go/Rust agree on six observations;
unread entries reject, while a coherently pruned snapshot still passes relative
to itself. Authentication of the original root and W01 remain open.

The W01 [analysis budget checkpoint](../migrations/go-practical/t02-w01/verification/analysis-budget/README.md)
uses the unchanged shared 131,072-entry bound for recorded directories and all
three analysis ZIP namespaces. Go and Rust agree at the limit and one over.
Other capture archives and original-root authority remain open.

The W01 [analysis module checkpoint](../migrations/go-practical/t02-w01/verification/analysis-module/README.md)
binds the original frontend manifest and sums to all three cached module
manifests, ZIP hashes and archives. Go and Rust agree on 20 observations.
At that checkpoint, cached `.info`, original-root and source-to-binary authority
were still open.

The W01 [analysis info checkpoint](../migrations/go-practical/t02-w01/verification/analysis-info/README.md)
validates the three cached `.info` records against Go JSON and time observations
within the same snapshot. Original-root/VCS and source-to-binary authority remain
open; the Go practical profile is inactive.

The W01 [original ELF section checkpoint](../migrations/go-practical/t02-w01/verification/compiler-elf/README.md)
adds native 32/64-bit/both-endian section/name/compression handling and exact ZLIB/
Zstandard data prefixes. All 425 native/Rust cases retain187 prior cases,9 actual
SDK tools and all 3 complete original SDK Zstandard streams; both 4,320,033-byte
expanded tables pass. All 8 affected Rust roots and scoped native/lint/formats pass.
The complete 5577-file/182-package/5697-object SDK/source/Root owner and prior controls
remain. General reader/codec equivalence and remaining W01 authority/proof work
stay open; the practical profile remains inactive.

The ELF checkpoint's [independent build wiring](../migrations/go-practical/t02-w01/verification/compiler-elf/build-wiring/README.md)
retains seven complete locked graphs and the complete private codec source package.
Actual pinned local Linux and owned-mirror observer builds pass with unchanged
inputs. Emulated observer compilation provides no native Linux release acceptance.

The W01 [strict standard-module checkpoint](../migrations/go-practical/t02-w01/verification/standard-module/README.md)
replaces dependency ParseLax in the owned SDK index with independent general strict
Parse facts and the separate native std/go/compiler-upper-bound condition. All1427
complete native cases,20 affected Rust roots and both native roots pass with scoped
lint/formats. Four fully rehashed actual SDK manifest controls preserve all5577
compiler files and182-package/5697-object application/Root bytes:3 malformed rows
reject and valid full directives remain accepted. Main/dependency policies, limits
and prior proof/error/type/source universes stay unchanged; W01 remains open.

The W01 [whole original SDK source-role checkpoint](../migrations/go-practical/t02-w01/verification/compiler-source-roles/README.md)
independently classifies all5480 prepared SDK sources and715 source package
directories inside the complete5577-file compiler owner. Every one of11004 raw
SDK sources and388 boundary probes agrees with pinned Go, including a complete
40000-import header/body without an AST-node restriction. All23 affected Rust
roots and scoped native/lint/format checks pass;3 fully rehashed unused-role claims
reject while retaining all original bytes,182 packages/5697 objects and the Root
contract. Strict standard-module semantics and remaining W01 authority/proof work
stay open; the profile remains inactive.

The W01 [original executable build-info checkpoint](../migrations/go-practical/t02-w01/verification/compiler-build-info/README.md)
independently reads all9 actual owned SDK tools and binds exact Go1.25/tool/host/
settings facts. All187 native byte/layout/grammar observations agree;12 affected
Rust roots and scoped Clippy/vet/formats pass. Three fully resealed actual tool
mutations reject while preserving the whole5577-file SDK and182-package/5697-object
application/Root contract. Full reader equivalence (including compressed ELF
section-name tables), SDK source roles and remaining W01 authority/proof work
stay open; no frozen input scope is removed.

The W01 [whole original compiler checkpoint](../migrations/go-practical/t02-w01/verification/compiler-inputs/README.md)
retains every actual prepared SDK byte/mode after image retirement and independently
binds its complete inventories/components to original export inputs and all captured
SDK sources. The actual5577-file/163457770-byte compiler includes a file above16MiB
without changing source budgets. All10 affected Rust roots and2 native roots pass;
30 coherent claims and a rehashed unused-SDK-source control reject. Independent
binary build-info/source roles and remaining W01 authority/proof work stay open.

The W01 [original observation byte checkpoint](../migrations/go-practical/t02-w01/verification/observation-json/README.md)
closes the private root hex-field mismatch against unchanged16MiB stdout/1MiB
stderr budgets. All30 complete byte/hash witnesses agree across Go/Rust; a full
182-package/5697-object native owner crosses the former JSON ceiling in actual
package/graph outputs. All15 affected Rust roots,7 native roots and targeted lint/
format checks pass. Other resource/read/correctness and W01 authority/proof work
remain open; common JSON and all kernel/certificate limits stay unchanged.

The W01 [pruned root expansion checkpoint](../migrations/go-practical/t02-w01/verification/root-expansion/README.md)
now derives final roots from original main/dependency manifests, including native
bootstrap normalization and transient expansion inputs. All18 native expansion
cases,225 version comparisons and84 exact sort permutations agree with Rust.
Eight affected Rust roots and8 native roots pass, including a fresh whole HTTP
owner roundtrip. General correctness/read/resource and remaining W01 authority/
proof obligations stay open; the practical profile remains inactive.

The W01 [independent graph input checkpoint](../migrations/go-practical/t02-w01/verification/graph-import/README.md)
reconstructs original initial/final loaded summaries, exact cache/replacement/
checksum inputs, native graph bytes and observed selected maxima. All15 affected
Rust roots and a complete182-package native owner pass;21 coherent claim/content
controls reject. General root expansion/MVS and remaining W01 authority/proof
obligations stay open.

The W01 [v2 graph input checkpoint](../migrations/go-practical/t02-w01/verification/graph-inputs/README.md)
binds the original native cache capability, preserves exact version bytes and
orders distinct equal-version labels deterministically. All five native graph
roots pass, including a full HTTP owner with a non-UTF8 toolchain suffix.
Independent Rust graph import/MVS and remaining W01 obligations stay open.

The W01 [dependency manifest checkpoint](../migrations/go-practical/t02-w01/verification/graph-manifests/README.md)
adds an independent original ParseLax reader and exact version/pruning facts.
All779 native/Rust facts agree, preserving all656 earlier original mode cases;
11 affected Rust roots and the native oracle pass. Full graph transport, root
expansion/MVS and the remaining source/context/foundation work stay open.

The W01 [original graph checkpoint](../migrations/go-practical/t02-w01/verification/module-graphs/README.md)
retains native initial/final graph inputs, old/empty unselected manifests,
pruning, replacements and whole original owner handoffs. Five new and15 affected
native roots pass. Independent Rust ParseLax/root expansion/MVS, complete
transient input authority and all remaining W01 proof/context obligations remain
open; native final roots remain observations and the practical profile is inactive.

The W01 [ordinary Nat checkpoint](../migrations/go-practical/t02-w01/verification/natural-foundation/README.md)
adds eight unbounded arithmetic operations and45 ordinary checked laws,
including general results for arbitrary naturals. Both current checkers agree on15
same-byte cases; the complete ordinary foundation/private context and application
source proofs remain in progress.

The W01 [balanced integer checkpoint](../migrations/go-practical/t02-w01/verification/integer-foundation/README.md)
adds13 ordinary definitions and42 library laws, with checked Nat-equality
checkpoints. Both current checkers agree on15 same-byte cases. Half-open folds,
complete representations/operations/source foundation and private context binding
remain in progress.

The W01 [half-open fold checkpoint](../migrations/go-practical/t02-w01/verification/range-foundation/README.md)
adds eight definitions and31 checked laws with only the actual bounded-prefix
premise. Both current checkers agree on17 same-byte cases. Full carrier/source/
operation/definedness foundation and private context binding remain in progress.

The latest W01 [full-width word checkpoint](../migrations/go-practical/t02-w01/verification/word-foundation/README.md)
adds1000 ordinary definitions and144 basic laws for8/16/32/64-bit circuits. Both
current checkers agree on20 candidates, and all1640 actual-DAG/integer/native-Go
observations agree. Full numeric/source binding and private context remain open.

Historical design-stage notes: implementation was authorized 2026-10-16.
At the following checkpoint T01-W01, W02 and W03 were complete.
W03's [completion audit](../migrations/go-practical/t01-w03/completion-audit.json)
pins reviewed implementation `9d4e6fb7855a66d9fff309e6dbd9143652ee6e21`.
Its ordinary representations,
full64 arithmetic/Nat sums, complete bytes, descriptor/store/construction laws,
actual branch guards and coupled bound proofs have retained both-checker and
original-Go evidence. The separate private dependent Nat.induct prototype
resolves general invariant/abrupt-control feasibility and preserves production
sources and the old Nat/Boolean rules. Fourteen sequential upper/boundary
recipes now reproduce exact certificates and accepted hashes with identified
hardware resources. W04 owns the exact additional interface, semantic limits,
resource budgets, schema identities, migrations and production executors.
Complete production source/operation/application binding belongs to T02–T06.
W04 is In progress: [draft requirements, owners, surfaces and candidate limits](../migrations/go-practical/t01-w04/README.md)
are recorded. Root/error/empty-slice rules now have 49 original-Go vectors
and native observations with 35 evidence checks. A 56-family logical operation
table and evaluation/contract/loop rules add 54 native cases with 52 evidence
checks. Private order/general Sort0 strong induction adds 39 both-checker
proof/interface/regression cases with a proposed fixed Bool type selector.
An ordinary arbitrary-Nat rank+1 finite-stop schema adds 13 both-checker
candidate/regression cases, including zero-rank exits without decrease. Its
invariant/decrease obligations apply only to continuing backedges; full Go
source totality still requires separate bindings and terminating calls.
The closed overlay draft adds 70 source/normalized expression heads, 24 vectors
and intact native Go type/scope observations. A scheduling clarification keeps
invariants/ranks before condition evaluation under LC02; abstract proof bytes
are unchanged. Normative identities, independent import and complete
definedness/source/proof reconstruction remain due.
Canonical owner/typed-view work adds 15 native/model observations and ten
ordinary coupling claims in unchanged production checkers, with five accepted
zero-axiom proofs. Typed child views share their parent's store; escaped arrays
retain aliasing across exact source scopes. General path/exposure/lifetime/live
counter and source proofs remain due.
The [derived lifetime/accounting draft](../migrations/go-practical/t01-w04/storage-lifetimes/README.md)
adds complete modeled roots for variables, entry arguments, evaluation/target
temporaries, joint results and closed error payloads. It preserves independent
entry/post observations, derives escape translations and reuses retired slots.
Twelve native/model observations, 24 targeted tests and two accepted ordinary
masked-frame/past-namespace statements support the design; three altered
proofs reject. Source/root completeness and allocator/translation correctness
remain unproved, and no actual Go GC or finite-carrier guarantee is claimed.
The [closed source/closure draft](../migrations/go-practical/t01-w04/source-admission/README.md)
assigns all 56 pinned AST nodes and 18 builtins explicit dispositions. Its
81 intact original-source fixtures yield 31 unproved candidates, 46 candidate
rejections and four original Go failures. Type-check-only functions, concrete
error method signatures and selected body dependencies stay distinct; static
call cycles and ambient state retain transitive paths. Complete independent
capture/import, typed/effect predicates and source/proof binding remain due.
The [numeric/carrier draft](../migrations/go-practical/t01-w04/structural-limits/README.md)
adds64 explicit counter candidates and192 isolated boundary vectors, with
85 original native layout sources and complete current/past accounting.
The unused-HTTP capture measures1,417,549 syntax nodes, so practical Go's
candidate becomes4,194,304 while the predecessor's1,000,000 stays unchanged.
Abstract2Mi-value/8Mi-store/32Mi-total units charge full bounded shapes,
copies, roots and observations; source/allocator/finite-ID proofs remain due.
The [identity/DAG draft](../migrations/go-practical/t01-w04/identity-dag/README.md)
proposes101 unreserved identities/41 domains and closes ten source-identity
record shapes. A26-node/seven-root graph and44 Python/Rust/Go hash vectors
exercise acyclic source/module/package/declaration/type/capture/loop linkage.
All143 predecessor paths have candidate migration routes; full root/union/
predicate/symbol migration review and independent source/registry proofs remain.
The [context protocol draft](../migrations/go-practical/t01-w04/context-protocol/README.md)
closes six private registry/entry/context/request/Go-parameter/selection roots.
All four retained profiles delegate to actual unchanged Rust validators;
both Rust32/64 targets remain valid. Nullable retained-profile foundations
and an exact private Go reference prevent profile mixing. The152 vectors,
31 actual legacy-policy queries and20 targeted tests cover full typed
context linkage and rehashed-registry attacks. Complete foundation/limits,
source resolution and downstream schema/migration proofs remain required.
The [foundation/limits package draft](../migrations/go-practical/t01-w04/foundation-limits/README.md)
closes three more packaging roots with64 counter/unit/outcome records, all56
logical operations in eight recipe families and the exact joint core proposal.
Its descriptor binds original member bytes through an acyclic limits-to-
definitions-to-descriptor-to-context chain. The102 configuration vectors,
21 Python/Rust/Go preimages and20 targeted tests preserve both private
generations and four existing profile policies. The recipe inventory remains
explicitly incomplete ordinary kernel/source foundation work.
The [closed carrier expansion](../migrations/go-practical/t01-w04/closed-carriers/README.md)
now derives42/43 concrete instances from39 captured/fixture types. Complete
carrier/zero expansions and46/47 ordinary theorems pass both unchanged private
checkers with zero axioms; six altered definitions/claims reject. Aliases,
nominal identity, full word/byte shapes, global-U-dependent Plain and ordered
result arity remain explicit. Owner labels stay Nat ghosts, with finite-id,
input validity, complete source/operation bindings and normalization proofs
still required. Twenty targeted tests cover the concrete expansion boundary.
The [closed contract wire draft](../migrations/go-practical/t01-w04/contract-wire/README.md)
closes three roots and all70 source/normalized expression arms with exact
nominal type IDs, lexical bindings, de Bruijn depths and observation namespaces.
Twenty targeted tests,24 source cases and97 Python/Rust/Go canonical preimages
cover the private model. Original overlay bytes precede capture-dependent
identities. Independent production capture/import, source and definedness
proofs remain due; typed normalization is not a proof verdict.
The [concrete operation-signature draft](../migrations/go-practical/t01-w04/operation-signatures/README.md)
covers all 56 families with concrete source identities, state/control/region
interfaces, static-call binding requirements and complete required rule
members. Its 224 vectors, 24 selected tests and 56 three-observer hash preimages
exercise signature integrity. Compound operators inherit their actual checks;
assignment values retain a separate arity from function results. Executable
VIR, source/SSA/provenance and ordinary operation proofs remain required.
The [typed function-body draft](../migrations/go-practical/t01-w04/vir-body/README.md)
adds explicit SSA/state/region flow, static call/post provenance, normal-only
continuations and deferred typed-location access sites. All56 operation
components,three body candidates,20 graph vectors and21 selected tests cover
the private structural boundary. Sixty-nine full-limit Python/Rust/Go hash
vectors retain depth256. Complete shared VIR/source/dynamic-invocation and
ordinary semantic proofs remain required; no application claim is established.
The [shared VIR/contract module draft](../migrations/go-practical/t01-w04/shared-vir/README.md)
now closes the five-profile envelopes and Go whole-capture body/contract/scope
sets, including multiple actual overlays and unannotated callees. Seven modules,
73 cases and22 selected tests check exact context dispatch, source-buffer
authority, acyclic dependency closure and module counters. Sixty-two canonical
preimages agree across Python/Rust/Go. Actual retained importer/type/WP execution
preserves the complete typed projection and serialized VC data in five target
contexts. These fixture comparisons and typed Go models supply no general
source-equivalence or ordinary proof. Downstream artifact roots/migrations
and the full T02–T11 implementation remain required.
The [source-map/frontend-manifest draft](../migrations/go-practical/t01-w04/capture-artifacts/README.md)
adds exact common envelopes and Go raw input inventories. Full function/node/
terminator locator coverage, actual captured-byte hashes, selected roots and
capture/build/module/package references stay linked. Eight artifact triples,
187 vectors,24 targeted tests and19 Python/Rust/Go preimages exercise the
boundary. Actual unchanged public validators preserve full old map/manifest
values in five retained target contexts and retain independently authorized
synthetic origins. Source/AST/ordinary proofs, registered practical frontend/
release provenance and certificate-manifest/VC/policy/API/envelope/release
closure remain required; the private external reference fixture is uninstalled.
The [VC/certificate-source draft](../migrations/go-practical/t01-w04/proof-artifacts/README.md)
adds exact source-linked common VC/certificate references and actual retained
VC/skeleton regeneration,with flat term arrays preserving depth768. Eight
bundles,181 cases and25 targeted tests retain full native statement values in
five target contexts. Go has19 closed theorem-demand families,169 concrete
requests and56 local fragments retaining416 member/access demands. All260
Python/Rust/Go hash inputs agree. Go ordinary proposition/statement expansion
and final skeleton/program assembly remain unfrozen; skeleton requests stay
pending and no caller context can promote them. No application proof is claimed.
The [ordinary word component](../migrations/go-practical/t01-w04/word-foundation/README.md)
now supplies full8/16/32/64-bit circuits and source-typed value/definedness
specializations for787 operations over13 distinct word types. Five ordinary
sets pass both unchanged private checkers with zero axioms;16 false/ill-typed
controls reject. All1,640 actual ordinary-AST/Go/integer observations agree.
Complete Nat/source correctness proofs and Go VC elaboration remain required;
this component does not promote the preceding169 theorem requests.
The [numeric addition bridge](../migrations/go-practical/t01-w04/word-numeric-bridge/README.md)
now connects the actual ripple result/carries to genuine Nat conservation and
conditional signed cross sums at all four widths,including13 exact typed Value
constants. Both checkers accept all four sets with zero axioms. Fourteen altered
statements reject by type mismatch; two dropped-carry candidates remain distinct
conversion-fuel failures. All288 actual ripple/Go/big-integer observations agree.
Remaining word/Nat/Int/contract/source proofs and complete Go elaboration stay open.
The [ordinary arithmetic component](../migrations/go-practical/t01-w04/ordinary-arithmetic/README.md)
adds unbounded Nat algebra,balanced-pair Int algebra and representative-safe
comparisons,and half-open logical quantifiers/sums. Two ordinary sets pass both
checkers with zero axioms; eight altered candidates reject by type mismatch.
All27 existing closed logical heads now construct typed values and unproved
Boolean statements. All149 Go/big-integer observations agree with an independent
oracle;139 also agree with actual ordinary-AST execution,while ten large cases
remain pending at that interpreter's private unary-loop ceiling. Sixteen
targeted tests preserve false-clause,source and full-Go proof boundaries.
The word-to-general-arithmetic connection,partial/source expressions and
independent elaborator correctness remain required. All169 complete Go
requests stay unelaborated; W04 remains open.
The [word/arithmetic addition connection](../migrations/go-practical/t01-w04/word-arithmetic/README.md)
now proves pointwise successor-function interpretations at every cube depth
and connects actual8/16/32/64-bit ripple results to genuine Nat carry equations
and conditional semantic Int equality. All four sets pass both checkers at
zero axioms;16 altered candidates reject by type mismatch. Thirteen exact
typed Values retain their source identities and actual guards. All96 native
Go/integer observations and24 selected actual denotation executions agree.
Generic checked application/conditional lemmas solve the initial32/64-bit
conversion-fuel failure without changing checker budgets. Other word/source
operations,full ordinary/Go proofs and W04 freeze remain required.

The [common frontend envelope draft](../migrations/go-practical/t01-w04/frontend-envelope/README.md)
binds pre-lowering requests to independent snapshots,context,selection and
release references. It validates retained success/diagnostic projections with
unchanged public APIs and preserves five independent old-context baselines.
New Go diagnostic codes and object/byte/function spans remain model data;
protocol acceptance supplies no registered-run or source-proof authority.
Policy/API/release roots,complete ordinary Go assembly and reviewed migrations
remain required before W04 can freeze. W05 still owns the cumulative T01 gate.
Shared/downstream wire tables, remaining variant/source foundation,
limits, schemas/hash identities and migration review remain open.
T01 is incomplete and no practical Go profile is active.

The [policy evidence draft](../migrations/go-practical/t01-w04/policy-api/README.md)
now binds scan/evidence/reproduction/receipt v4 to complete source graphs and
real source-free certificate bytes. Two retained Rust certificates reach both
zero-axiom checkers after common-manifest rebinding. Four observed assembly
errors remain errors; unrelated valid certificates cannot discharge Go's169
unelaborated demands. Six retained API traces cover seven operations and72
atomic failures. New API/AI/release roots,full ordinary Go assembly,reviewed
migrations and native release gates remain required before activation.

The [implementation ledger](go-practical-implementation-traceability-ledger.md)
owns detailed status, source/checker identities and local verification.
Implementation now targets Go/Rust without another language's release
prerequisite. T01 passed its historical W05 gate; T02's cumulative gate remains
due in W04. Scope and semantic decisions are owned by
[the design](09_go_practical_subset_design.md).

The [common API contract design](../migrations/go-practical/t01-w04/api-contracts/README.md)
defines request/response/session v4 for all33 declared routes,exact snapshots,
atomic preparation/commit and source-derived ordinary readiness. Its6800
configuration/transport cases and96 model exchanges remain separate from102
fresh actual v2 operations. All169 Go demands remain unelaborated. APIv4
execution is assigned to T07-W01; source imports migrate in T02-W03.
AI/release roots,complete ordinary-program signatures and the final reviewed
consumer migration table remain W04 design work.

The [AI request/explanation design](../migrations/go-practical/t01-w04/ai-explanation/README.md)
closes the v4 wire and redaction/batching rules. All169 current Go demands
remain pending across complete32-property batches. Local statuses,source
bindings and full theory/axiom summaries come from policy; provider prose
cannot change them. English/Japanese bundles and40 independent hashes remain
separate from the original consumer tests at that archived source epoch. No
provider is contacted. Compiled v4 integration for the four required Go/Rust
target contexts remains T07 work. Release roots,ordinary-program/materialized-policy signatures
and exhaustive consumer migration review remain W04 work; W05 owns publication
and the cumulative T01 gate.

The [release metadata design](../migrations/go-practical/t01-w04/release-contracts/README.md)
closes seven root shapes and the acyclic build/image/hash/readiness contract.
At that archived checkpoint, old validators covered4 languages/5 target tuples;
current implementation supports Go/Rust only. Metadata-only
projections cannot run as new-protocol binaries or authorize activation. New
IDs rebind component scope/content refs without mutating old inventories.
229 cases and13 independent Rust/Go hashes pass. Actual new-family builders,
byte handles,native gates and atomic activation/rollback remain T07 work.
W04 still needs final ordinary-program/materialized-policy and host/budget
closure plus exhaustive consumer migration/axiom-policy/source-epoch review.

## 1. Entry, ordering, and execution

Go practical implementation proceeds independently under the Go/Rust-only
scope. The C# and Java work plans are retired. Each Go facility must be
validated through Go's own gates; prior receipts attest only to their original
source and registry identities. Production phases remain serial.

Execute T01 through T11 serially, and W01 through the last W serially within
each T. Each W has a concrete review boundary. Multiple requested work items
can be carried out in order; passing a work-item gate does not require a new
user confirmation. Do not begin the next T before the preceding T's gate
passes. A failed feasibility or proof gate requires a recorded correction to
the affected design/freeze; it must not be bypassed with an assumption.

| Stage | Scope | Priority | Result |
| --- | --- | --- | --- |
| GO-PRAC-T01 | Feasibility, profile v1 and shared-contract freeze | Prerequisite | Frozen inactive package and evidence |
| GO-PRAC-T02 | Dependency capture and selected-function closure | Prerequisite | Private existing-module integration |
| GO-PRAC-T03 | Types, constants, and ordinary syntax | 1 | Private domain-value verification |
| GO-PRAC-T04 | Multi-result calls and closed errors | 2 | Private success/failure verification |
| GO-PRAC-T05 | Bounded byte strings | 3 | Private identifier/text verification |
| GO-PRAC-T06 | Slices, construction, and structured loops | 4 | Private collection verification |
| GO-PRAC-T07 | Policy integration and first release | 1–4 | Public practical v1 |
| GO-PRAC-T08 | Profile v2 and successor-contract freeze | Prerequisite | Frozen inactive v2 package |
| GO-PRAC-T09 | Bounded maps | 5 | Private membership/update verification |
| GO-PRAC-T10 | Pointers, receivers, and state transitions | 6 | Private state-transition verification |
| GO-PRAC-T11 | Second release and final regression closure | 5–6 | Public practical v2 |

At T01-W01 create `go-practical-implementation-traceability-ledger.md` beside
this plan. It records each W's status, predecessor, reviewed commit, exact
inputs and hashes, specification rows, implementation/test owners, commands,
results, outstanding findings, and deferred checks. The ledger becomes the
implementation status authority; the unchecked work list below is a plan.

## 2. Common completion rule and verification routing

Every implementation W includes the code or freeze change, affected tests,
source/contract/VIR/VC/proof evidence where applicable, documentation, and
review fixes. A semantic definition certificate is not an application proof.
Each feature-stage final W needs a nontrivial source-to-certificate example,
all requested claims discharged, both-checker acceptance, and a deliberately
incorrect case that fails at the intended boundary.

Run targeted checks during W items. The final W of every T additionally runs
`./scripts/check-fast.sh` for that T's cumulative changes and cannot complete
until it passes. Do not reproduce that whole gate during intermediate W
items. Record why the selected tests cover the change and explicitly record
the full gate as deferred until the final W. T07 and T11 also run the complete
local native Linux release gates specified by the root README, with two-pass
installed-release evidence. All execution stays local; no workflow files or
GitHub Actions are introduced or used.

The fast gate now runs every remaining workspace test. The previous C#
test deferral is historical; the removed implementation has no tests to skip.
Native Linux release acceptance remains a separate required gate.

## 3. GO-PRAC-T01 — Feasibility and v1 freeze

Entry: the user-authorized pre-C# order, the last installed four-language
release, and an audited clean source baseline. Practical C# completion is not
required. T01-W01 must distinguish historical release evidence from the
current source state; T01-W05 still requires its cumulative whole gate.

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T01-W01 | Record the pre-C# schedule amendment, exact clean source baseline and last installed-release receipt with their distinct scopes, new ledger, current consumer inventory, and fixed-profile regression corpus. Pin the proposed target/compiler and supported effective-language-version table. |
| T01-W02 | Probe original-package type checking with unused unsupported functions, external modules, replacements, module metadata, build tags, initializers, and reachable global effects. Produce a capture/verification-closure matrix and exact offline snapshot contract. |
| T01-W03 | Demonstrate ordinary encodings for nominal values, multi-results/errors, bounded bytes, slice descriptors/stores and abrupt loop control. Both checkers must accept representative conditional lemmas, not only definitions. Measure the design's v1 workload points and reject unsound projections. |
| T01-W04 | Freeze v1 source admission/rejection, operation/check table, closure/effect rules, contract expressions, exact numeric limits/budgets, all changed schema/hash identities, producer/consumer migrations, and positive/negative vectors. Assign every requirement a production executor. |
| T01-W05 | Publish the reviewed inactive specification package and manifest entries, review source-model binding and proof feasibility, run targeted freeze checks and the full T gate. Keep public selection and the installed registry unchanged. |

W04's exact specification owners are `GO_PRACTICAL_PROFILE_V1.md`,
`GO_PRACTICAL_FOUNDATION_V1.md`, `GO_PRACTICAL_SHARED_ARTIFACTS_V1.md`,
`CORE_NAT_INDUCTION_V1.md` and `CORE_BOOL_TYPE_CASES_V1.md`. W05 publishes
these inactive normative references and their conformance manifest entries.
Kernel feasibility is measured independently of the C# design.

Exit: a measured, implementable frozen v1 scope with zero unresolved semantic
questions. Any proposed workload above a viable bound is explicitly excluded
or redesigned before freeze, rather than advertised as supported.

W04 freezes semantic predicates,ordinary statement/program signatures,exact
limits and the producer/consumer contract. General compiler/elaborator
correctness,all operation/definedness/control/frame lemmas and complete
application proofs remain mandatory in their assigned T02–T06 gates. Their
unfinished implementation proofs alone do not extend W04; an undefined
semantic rule or unmeasured feasibility problem still prevents design freeze.
W04 closes the AI/release roots,ordinary-program signature/budget contract and
all183 consumer dispositions in its completion audit. W05 publication and the
cumulative T01 gate must pass before T02 starts. This changes no implementation
or release acceptance condition and does not activate the private profile.

## 4. GO-PRAC-T02 — Captured modules and function selection

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T02-W01 | Implement private profile/context contracts and immutable dependency capture. Verify exact module resolution, replacement roots, source/export metadata and compiler identities; reject path escape, missing snapshots, stale checksums, and forbidden network/toolchain fallback. |
| T02-W02 | Implement original-package loading and the two closure algorithms. Record every declaration's disposition and root-to-rejection path. Prove by tests that unused handlers are allowed while reachable unsupported helpers, globals, callbacks and cycles reject. |
| T02-W03 | Implement the frozen common artifact migration, nominal cross-package identity, overlay linkage and independent consumer validation. Exercise retained Go fixed, practical Go, Rust32 and Rust64 contexts and reject mixed versions or caller-supplied adapters. |
| T02-W04 | Run a scalar policy end to end inside a captured multi-package application with an unused external dependency. Verify conditional scope at function entry, both-checker proof evidence, deterministic replay, old Go behavior, and the full T gate. |

Exit: dependency support and selection work through real private consumers.
A source-extraction script or a manufactured stub package is insufficient.

Implementation ordering amendment (2026-10-19): stage the two already frozen
Sort0 core interfaces in T02-W01 as prerequisites for actual checked ordinary
foundation definitions. The [record](../migrations/go-practical/t02-w01/core-foundation-ordering.json)
changes execution order within T02; all frozen specification members/semantics
remain unchanged. T02-W03 keeps the common artifact/importer migration and T02-W04
keeps the cumulative gate. Public Go practical activation remains T07 work.

## 5. GO-PRAC-T03 — Domain types and ordinary syntax

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T03-W01 | Implement target-sized integers, nominal primitive/array types, aliases, cross-package structs/constants, grouped/inferred constants and `iota`. Test target width, representability and identity confusion. |
| T03-W02 | Implement zero-filled literals, local value updates, compound assignment, increments, parameter local copies, `else if`, expression switches and expanded short-circuit contexts with faithful source origins. Test source evaluation and case ordering. |
| T03-W03 | Connect typed operations, VC generation and proof recipes. Add switch/amount/aggregate examples and negatives for overflow-as-integer reasoning and duplicated target evaluation. |
| T03-W04 | Close source-to-certificate coverage for the priority-1 matrix, compare pinned Go observations, check both verifiers and regressions, review findings, and pass the full T gate. |

## 6. GO-PRAC-T04 — Multiple results and closed errors

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T04-W01 | Implement zero-to-16-result products and projections, single call execution, joint contract results, tuple/parallel assignment, `:=` shadowing, blank destinations, forwarding and normalized named returns. |
| T04-W02 | Implement the closed non-pointer error universe and exact nil-interface cases. Treat `Error()` as a signature dependency when unused; reject open dispatch, unlisted dynamic types and pointer errors. |
| T04-W03 | Generate branch-specific result obligations and checked call composition. Verify `(Amount, bool)` and `(Decision, error)` examples, including a valid nonzero first result on failure and incorrect tuple/result binding. |
| T04-W04 | Complete differential observations, independent import negatives, source-free application proofs and the full T gate. Verify that proof failure is not reported as a compiler rejection or success. |

## 7. GO-PRAC-T05 — Bounded byte strings

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T05-W01 | Implement the frozen byte carrier, literals/zero values and operations: equality/order, length, indexing, substring and bounded concatenation. Preserve arbitrary bytes and exact nominal types. |
| T05-W02 | Connect checked representation/binding laws, safety and result-bound obligations, plus exact-symbol prefix/suffix adapters. Verify the library binding rather than trusting package names. |
| T05-W03 | Verify identifier-validation and selected `Error()` functions. Exercise NUL, invalid UTF-8, multibyte strings, endpoint bounds and failed concatenation-capacity claims. |
| T05-W04 | Close both-checker application proofs, measured capacity/budget cases, old-profile and affected-consumer tests, review and full T gate. |

## 8. GO-PRAC-T06 — Slices and structured loops

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T06-W01 | Implement read-only descriptors/stores, nil/empty distinction, length/capacity, index and slice expressions, array copy/view behavior, and alias-preserving import. Include observable data beyond logical length. |
| T06-W02 | Implement fresh bounded construction, local alias updates, overlap-correct copy, explicit-capacity make and non-reallocating append. Prove region separation and construction bounds; reject hidden input writes. |
| T06-W03 | Implement array/slice range, condition-only and nested loops, break/continue and early return, with effective-language-version bindings and exact source contract attachment. |
| T06-W04 | Generate and discharge initialization, preservation, exit, postcondition and totality obligations. Add counterexamples for copied-list alias semantics, wrong continue/post behavior and omitted return paths. |
| T06-W05 | Verify the design's `Quote` and preallocated-filter examples end to end, including mathematical-sum binding, all failure paths and no input mutation. Measure actual proof/checker costs, close review and pass the full T gate. |

Exit: the first product milestone works privately in an existing module.
There are no pending required application theorems behind a “supported” label.

## 9. GO-PRAC-T07 — Practical v1 public release

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T07-W01 | Complete the practical-Go strategy, CLI/API and evidence routes for all v1 obligations. Display exact entry bounds, selected functions and remaining obligations; reject old-strategy/context substitution. |
| T07-W02 | Build registered bundles, installation inventory and replay corpus. Demonstrate unchanged application builds without MPK dependencies and preserve fixed-Go semantic behavior under the common artifact migration. |
| T07-W03 | Rehearse atomic activation, wrong-version rejection and rollback from reviewed bundles. Run installed native Linux isolation and release checks, with two matching complete receipts. |
| T07-W04 | Apply the reviewed v1 registry/consumer activation atomically, finish documentation and regression review, pass the full T gate and required final release checks, and record the exact shipped scope. |

Exit: v1 is a public immutable profile with a useful, completely checked
business-function corpus. Missing Linux evidence prevents release completion;
a local host that cannot run the gate does not substitute a simulated receipt.

## 10. GO-PRAC-T08 — v2 feasibility and freeze

Entry: the practical v1 release is complete.

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T08-W01 | Capture v1 as the preservation baseline. Inventory map references, root pointers, permissions, frame observations, typed-nil errors and every affected shared consumer. |
| T08-W02 | Probe Go alias/nil/receiver observations and ordinary map/heap encodings. Both checkers must accept representative lookup-update and state/frame lemmas. Measure the map/root-object workload points and reject ordered-map substitutions. |
| T08-W03 | Freeze the immutable v2 profile, restricted key/value/pointer matrices, effect and ownership rules, contract additions, exact bounds/budgets, adapter rules, schemas/hash changes and vectors. Publish the inactive package and pass the full T gate. |

Exit: v1 remains immutable; no map/pointer form is admitted via a v1 feature
flag. Any shared migration has a complete existing-consumer preservation plan.

## 11. GO-PRAC-T09 — Bounded maps

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T09-W01 | Implement map handles, finite mappings, admitted keys/values, nil/empty states, lookup and comma-ok projection with nominal/type and region validation. |
| T09-W02 | Implement fresh/exclusive map construction, aliased insert/replace/delete and explicit footprint state. Generate cardinality and unrelated-key/frame obligations. |
| T09-W03 | Verify membership and deduplication/update examples. Test nil writes, zero versus missing, alias-visible updates, capacity boundary, and map-range rejection; prove the canonical carrier cannot introduce source ordering. |
| T09-W04 | Close source-to-certificate proofs, performance/resource measurements, independent corruption cases, v1/fixed-Go regressions, review and the full T gate. |

## 12. GO-PRAC-T10 — Pointers and mutable receivers

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T10-W01 | Implement nonzero-sized root objects, pointer/nil identity, local allocation/address taking and dereference. Validate permitted type graphs and reject interior addresses, unsafe paths and unsupported escapes. |
| T10-W02 | Implement direct pointer receivers, local aliases, effect/permission transfer, writable-root exclusivity and automatic address/dereference normalization where admitted. Test nil-safe methods and interfering expression rejection. |
| T10-W03 | Implement `old`, modifies/frame and state invariants, joint result/post-state contracts, and pointer concrete errors with typed-nil preservation. Prove callee effects instead of trusting annotations. |
| T10-W04 | Verify guarded account reservation, both success and unchanged-state failure. Include overlapping root inputs, hidden writes, frame violations, failure after mutation and incorrectly collapsed typed nil. |
| T10-W05 | Close all required application proofs, both-checker costs, independent reconstruction and predecessor regressions, review and the full T gate. |

## 13. GO-PRAC-T11 — Practical v2 public release

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T11-W01 | Complete v2 policy/API/evidence and registered bundles, with precise ownership and caller obligations. Verify fixed-Go and v1 preservation and reject crossed profiles or effects. |
| T11-W02 | Rehearse atomic shared-artifact/profile activation and rollback, run the complete real-module corpus and native Linux two-pass release gates, and close release findings. |
| T11-W03 | Activate the reviewed v2 release, document the exact map/pointer limits and deferred forms, pass the full T and final release gates, and publish the final traceability ledger and receipts. |

Exit: both practical releases have reproducible proof evidence for their
advertised functions, and subsequent development stays within the retained
Go/Rust pipeline.

## 14. Required decisions before normative publication

The design fixes semantics and priority, while these measured implementation
facts are deliberately assigned to their freeze owners:

| Fact to freeze | Owner | Evidence required |
| --- | --- | --- |
| Exact compiler/toolchain and effective-language-version table | T01-W01/W02 | Registered inventory, module/file version and compiler-shape probes |
| v1 bounds and resource budgets | T01-W03/W04 | Representative conditional proofs, both checker receipts and cost measurements |
| v1 wire shapes, hash domains and complete consumer migration | T01-W04/W05 | Closed schemas/vectors, ownership inventory and reviewed migration plan |
| v2 bounds, ownership/effect encoding and budgets | T08-W02/W03 | Map/heap application-shaped lemmas and alias/frame adversarial cases |
| v2 wire shapes and consumer migration | T08-W03 | Closed specification/vector package and predecessor preservation inventory |

Do not fill these entries with guessed digests, speculative success reports,
or an unmeasured “unbounded” model. Their completion is part of implementation
feasibility work, not missing user input or permission to skip a gate.

## 15. Design-only validation record — 2026-10-16

This change adds the design and work plan plus navigation links. No production
code, frozen specification, vector bytes, or registry is edited. Validation
was selected for that documentation-only scope:

- Local Markdown destinations and balanced code fences: passed for the two
  new documents and both edited README files.
- Work-item IDs: 45 unique, sequential W items across T01–T11; passed.
- The embedded Go example: parsed with `gofmt` and built with `go test ./...`
  in a disposable module, with networking/toolchain switching disabled;
  passed under host Go 1.23.2 on darwin/arm64, with no behavioral test files.
- Whitespace/diff review: passed, including checks on the new document files.

The example build establishes syntax and type correctness only. It is not an
MPK proof or a target-Linux release receipt. No implementation T/W was started
or completed, so its full gate is not due. Production tests and
`scripts/check-fast.sh` were not run for this documentation change; they are
assigned to the implementation work items and their stage gates above.
