# CSHARP-03 Practical C# Implementation Milestones and Tasks

Status: current reviewed implementation decomposition, revised on 2026-09-06.
The native `JAVA-03-T10` x86-64 Linux release receipt is accepted and
`CSHARP-03-T01-W01/W02/W03/W04/W05/W06/W07/W08/W09/W10` have completed the entry audit,
consumer inventory, private frontend/toolchain closure proof, Roslyn shape
probes, primitive/string/numeric/codec runtime measurements, the candidate
foundation/specialization/binding/data semantics, and the successor contract/
boundary/transition/identity/limit freeze and publication. T02-W01/W02 then
completed the private successor registry/context foundation, registered
practical foundation, closed specialization engine, and monomorphic value
model. T02-W03 followed with the closed operation/check, construction,
binding, control/pattern, and exception vocabulary. T02-W04 then completed the
context-bound successor source artifacts, source map, manifests, closed tables,
and boundary byte/value linkage. T02-W05 then completed the strict successor
VIR importer, including independent foundation/specialization reconstruction,
closed operation and artifact linkage, structural control/value/ownership/
exception validation, bounded parsing, and the generic-free barrier.
T02-W06 then completed the context-bound successor VC and theorem-skeleton
models, closed ordinary-term routes and later proof ownership, deterministic
limits/hashes, and the proof-empty, theory-empty, zero-axiom ordinary-context
assembly profile without changing Certificate v0 or checker acceptance.
T02-W07 then completed the candidate-only frontend request/result protocol,
schema-ordered source/sidecar inventory, phase-ordered sanitized diagnostics,
artifact-free failure, and complete source-map/manifest/input-set linkage.
T02-W08 then completed the private Go, Rust, scalar-C#, and Java predecessor-
producer adapters, canonical v2 regeneration, retained-limit enforcement,
two-run equality, and four language-specific semantic-difference reports
without adding an installed route or public format selector.
`CSHARP-03-T01-W09-F01` is resolved
by W08's reviewed binary-addressed Boolean-cube and static concrete-transformer
expansion (ledger section 11). Both checkers accept all W09 capacity cases
through each frozen limit plus one without a core change. W10 publishes the
normative but inactive specifications and 700 vectors.
`CSHARP-03-T02-W01/W02/W03/W04/W05/W06/W07/W08/W09` and
`CSHARP-03-T03-W01/W02/W03/W04/W05/W06/W07/W08/W09/W10/W11/W12/W13` are complete, `CSHARP-03-T03-W14` is ready, and every
later implementation work item remains blocked by its serial predecessor.

Source design: `08_csharp_practical_subset_design.md`.

This document is subordinate to the source design, the published
`CSHARP_PRACTICAL_PROFILE_V1.md` and
`CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md` specifications, and their manifested
`csharp-practical-profile-v1.json` package. The T01-W09 private freeze remains
their immutable evidence source. This document
replaces the superseded source-visible-library,
iterator/async, and suspension-stage plan. Its `Wnn` items are the current
independently assignable units, but none is executable until its stated entry
gate and predecessor are complete.

The non-negotiable implementation boundary is:

- application source and build output contain no MPK namespace, package,
  assembly, attribute, interface, base type, generated source, or runtime
  component;
- every user-defined generic declaration, method, or closed use rejects; the
  only constructed-generic source exception is the exact value-type `T?` form;
- only the source design's enumerated MPK-owned semantic templates may produce
  derived closed instances, selected from one registered content-hash-pinned
  foundation bundle and expanded to monomorphic definitions before VIR
  emission;
- iterator, `yield`, async, await, task, and async-iterator forms are rejection
  and upgrade-vector scope, never positive implementation scope;
- the canonical boundary document is an MPK verification-overlay transport,
  not an application runtime protocol; and
- idempotency is optional and may be claimed only for the complete retained
  `Command` and `Context` snapshots under the source design's reflexive-equality
  rules.

## 1. Entry gate, authority, and stop conditions

The only permitted entry edge is:

```text
JAVA-03-T10 -> CSHARP-03-T01-W01
```

`JAVA-03-T10` completed atomic Java activation and the required native x86-64
Linux two-pass release gate on 2026-09-03. This prerequisite is satisfied and
T01-W01/W02/W03/W04/W05/W06/W07/W08/W09/W10 have closed the entry audit, consumer inventory,
private frontend/toolchain closure proof, and Roslyn data, construction,
control, exception, pattern, dependency, generic, iterator, and async-rejection
plus primitive/string/numeric/codec runtime measurements and the foundation/
specialization/binding/data semantic freeze. T01-W09 subsequently found that
W08's cross-result Bool/Nat recursor applications do not typecheck against the
checked standard interfaces; the authorized W08 amendment replaces them with
binary Bool addressing and static concrete-transformer composition, resolving
that type-feasibility finding. T01-W09 then froze the strict schema, identity,
boundary, transition, diagnostic, termination, and limit contracts and measured
both checkers at every capacity limit plus one; see ledger sections 11 and 12.
T01-W10 then published and reviewed the complete normative package; see ledger
section 13. T02-W01 then completed the closed private successor registry,
context, parameter, selection, contract-envelope, and predecessor-projection
implementation; see ledger section 14. T02-W02 then completed registered
foundation validation, root-driven closed specialization, concrete expansion,
and canonical monomorphic values; see ledger section 15. T02-W03 then added
the closed operation/check tags, monomorphic linear construction state,
binding commutation, explicit loop/pattern/exception control, and their
structural validators; see ledger section 16. T02-W04 then completed the
successor source-artifact/linkage layer; see ledger section 17. T02-W05 then
completed the strict successor VIR importer and validator; see ledger section
18. T02-W06 then completed the successor VC/skeleton and ordinary-context
assembly models; see ledger section 19. T02-W07 then completed the private
frontend protocol and complete artifact/inventory linkage; see ledger section
20. T02-W08 then completed private migration of all four predecessor producers;
see ledger section 21. T02-W09 then completed the private consumer and
foundation closure; see ledger section 22. T03-W01 then completed the private
source capture/declaration/closure gate; see ledger section 23. T03-W02 then
completed private concise-syntax, exact-type, and name-resolution
normalization; see ledger section 24. T03-W03 then completed private immutable
data declarations, enums, type graphs, and recursive defaults; see ledger
section 25. T03-W04 adds construction and invariant handoffs; see ledger
section 26. T03-W05 adds initializer transactions and finalization; see
section 27. T03-W06 adds shared structural equality and canonical ordering; see
section 28. T03-W07 adds explicit array ownership and initialization; see
section 29. T03-W08 adds the typed sequence construction substrate; see
section 30. T03-W09 adds ordered map/set bindings and typed operation handoffs;
see section 31. T03-W10 adds string plans and the shared typed codec relation;
see section 32. T03-W11 adds exact numeric relations and source plans; see
section 33. T03-W12 adds nullable and application outcome relations; see
section 34. T03-W13 adds calendar/time/GUID and bound instant/money relations;
see section 35. T03-W14 is ready. This
file remains planning material. W01 added its baseline and ledger; W02 added
only the private consumer inventory, owner tests, ledger evidence, and current-
status documentation; W03 added only private build-input evidence and its
harness and owner tests; W04 added only private disposable probe source,
canonical measurement evidence, its runner, documentation, and owner tests;
W05 added only the corresponding private control/exception/pattern probe,
canonical measurement, runner, documentation, and owner tests; W06 added only
the corresponding private dependency/generic/iterator/async-rejection probe,
canonical measurement, runner, documentation, and owner tests; W07 added only
the corresponding private primitive/string/numeric/codec runtime probe,
canonical measurement, runner, documentation, and owner tests. W08 adds the
candidate foundation/data specification, generated descriptor/definitions and
vectors, independent private runtime/model evidence and owner tests. W09 adds
only private freeze/capacity artifacts, generators, owner tests, and
documentation. W10 adds the normative profile/shared-artifact specifications,
manifested vectors, deterministic package generator, owner/upgrade/gate
closure, and specification tests. T02-W01 adds only the private, explicitly
injected successor registry/context validation module and its runtime tests.
T02-W02 adds only the private registered-foundation/closed-instance and
monomorphic-value module and its runtime tests. T02-W03 extends that same
private module and owner test with typed operation/check, linear-construction,
binding-commutation, control/pattern, closed-exception, handler, and unwind
models. It added no serialized successor artifacts or operation/check tables;
T02-W04 now implements those tables together with the context-bound successor
source-artifact, immutable-input, binding, boundary-evidence, source-map, and
manifest layer in a separate private module and owner test. T02-W05 adds only
the private, explicitly invoked strict successor VIR transport/import module,
the minimum W03/W04 private linkage accessors and transport derives it needs,
and its owner tests. T02-W06 adds only the Rustdoc-hidden successor VC,
theorem-skeleton, and ordinary-context assembly models; a narrow W05 validated
operation-table accessor and W04 lineage predicate; direct Certificate v0
structure validation; and its owner tests. It generates no proof and invokes
no checker. T02-W07 adds only the Rustdoc-hidden candidate frontend request,
success, and diagnostic protocol; strict inventory/hash/order validation; two
narrow W04 context/lineage predicates and lineage propagation; and its owner
tests. No application
fixture, installed candidate bundle, public route, active build input or active
registry/release descriptor has changed. T02-W08 adds only the Rustdoc-hidden
private predecessor adapter, its primary owner tests, and four pinned semantic-
difference reports. T02-W09 adds only the Rustdoc-hidden private successor
consumer orchestrator, its strict importers, candidate-only release consumer,
Certificate v0 handoff, receipt, and owner tests. T03-W01 adds only the private
pinned-Roslyn capture/declaration/closure gate, its exact input manifest,
executable C# harness, owner tests, and status evidence. T03-W02 adds only the
private concise-syntax, exact-type, and name-resolution normalizer, its exact
input manifest, executable C# harness, owner tests, and status evidence.
T03-W03 adds private immutable data declarations, enum/type/default validation,
its exact input manifest, executable C# harness, owner tests, and status
evidence. T03-W04 adds private constructor flow analysis, synthesized-member IL
checks, receiver-first functions, and pending type-invariant obligations.
T03-W05 adds ordered init/required transactions and finalization.
T03-W06 adds shared structural equality and canonical ordering.
T03-W09 adds canonical ordered map/set projections and typed operation handoffs.
T03-W10 adds UTF-16 string plans and the shared typed boundary codec relation.
T03-W11 adds exact numeric relations and source plans.
T03-W12 adds nullable and closed application outcome relations.
The next serial work is T03-W13's calendar, time, GUID, instant, and money values.

The following are unconditional stop conditions:

- an exact Roslyn or .NET behavior required by the design cannot be reproduced
  through a disposable public-API/runtime probe;
- any admitted capability cannot be encoded with ordinary Certificate v0
  terms, zero new axiom categories, an empty proof-node table, and an empty
  theory-certificate table;
- the registered foundation descriptor, bundle content hash, derived
  closed-instance closure, or concrete expansion cannot be frozen and
  independently recomputed, or any generic/template representation would
  remain at the VIR boundary;
- a deterministic bound sustainable by the frontend, VC generator, and both
  source-free checkers cannot be measured;
- an implementation would widen `mpk.csharp.scalar.v0`, accept ambient
  packages/project inputs, trust compiler/runtime output, or introduce a
  partial public practical-profile route; or
- predecessor semantics cannot be shown equivalent after the shared-artifact
  migration.

On a stop condition, mark the active work item `Blocked`, record the probe and
counterexample in the CSHARP-03 ledger, revise the source design, and repeat
T01. Do not silently narrow the frozen profile or publish a partial profile
under the same identity.

## 2. Execution protocol

The source design's IDs remain stage IDs. `Wnn` is the independently assignable
implementation unit. A later implementation request must name exactly one
`CSHARP-03-Tnn-Wnn`; an agent must not treat an entire stage as one task.
After the first full ID in a section, abbreviated `Tnn-Wnn` references always
mean the corresponding `CSHARP-03-Tnn-Wnn`; they never refer to CSHARP-02 or
another language phase. Throughout this document, `Tnn-Waa-Wbb` denotes the
inclusive range from `Tnn-Waa` through `Tnn-Wbb`.

All work items are serial. Within a stage, W02 depends on W01, W03 on W02, and
so on. W01 of a later stage depends on the final work item and closed gate of
the preceding stage. A work item is `Complete` only after its code, tests,
vectors/evidence, documentation, review fixes, and local verification are in
the same reviewed commit. No two CSHARP-03 work items may be active together.

Allowed statuses are `Blocked`, `Ready`, `In progress`, and `Complete`. The
traceability ledger created in T01 is the status authority and records for
every work item: commit, exact input hashes, produced artifact hashes, commands
run, result, reviewer findings, and any retained probe evidence.

Every work item follows this common contract:

1. Re-read the source-design sections and frozen rows owned by the item.
2. Confirm its predecessor is `Complete` and the worktree contains no
   unexplained changes.
3. Add or update the named primary implementation test before or with the
   implementation. Frozen spec-model tests do not count as production
   execution tests. T01 is the deliberate exception: its named executors test
   probes, schemas, and specifications without changing production, and its
   ledger must assign every vector to a later production executor.
4. Reject every unknown enum/tag/field/node/type/operation and every
   structural or transport over-limit input before emitting downstream
   artifacts. Represent an admitted run-time semantic bound with its frozen
   predicate and VC; do not claim verified acceptance until it is proved.
5. Run the targeted command listed below, then `./scripts/check-fast.sh`.
   Native Linux commands are additionally required only where stated.
6. Review the complete diff for contradictions, stale assumptions, missing
   negative cases, accidental public routes, ambient dependencies, and
   predecessor regressions; fix until the finding ledger is empty.
7. Update the CSHARP-03 ledger and commit only the bounded work item.

GitHub Actions and workflow files are forbidden. Validation is local. Git
commands use `/usr/bin/git`.

## 3. Frozen-name handoff and implementation surfaces

T01-W09 froze `mpk.csharp.practical.v1`, the successor shared-artifact family,
all sidecar schemas and hash domains, the context-dispatched C# bundle, and the
complete producer/consumer ownership map in
`develop/migrations/csharp-03/freeze/profile-freeze.json`. T01-W10 must publish
that private machine-readable handoff as the name-and-owner specification and
manifested vectors. Later tasks consume it verbatim and do not mint aliases.

Every implementation item preserves this end-to-end data flow:

```text
ordinary application .cs (no MPK dependency) + verification-overlay sidecars
  -> source/dependency/generic/binding validation
  -> exact closed-instance derivation from the registered foundation bundle
  -> deterministic concrete expansion
  -> monomorphic VIR (no source-generic or semantic-template node)
  -> VC, Certificate v0, and both source-free checkers
```

The application project ends at the first input boundary. Foundation
descriptors, semantic bindings, boundary documents, contracts, manifests, and
proof artifacts are MPK-owned verification inputs or outputs and are never
copied into the application build or deployed as an application runtime
dependency.

The registered foundation bundle is MPK's standard library only for
verification. It is distinct from the .NET Base Class Library and is not a
distributable .NET library for application projects.

Unless T01 records a measured reason to change a location, implementation
ownership is divided as follows:

| Surface | Primary location |
| --- | --- |
| C# capture, Roslyn adapter, subset, contracts, lowering, emission | `csharp-tools/csharp2vir/` |
| Semantic context, registry, foundation descriptor/instances, artifact validation, VIR, VC, hashes | `crates/mpk-vc/src/` |
| Installed frontend, sandbox, CLI, policy, evidence, AI | `crates/mpk-cli/src/` |
| Public structured API | `crates/mpk-api/src/successor_api.rs` and routed tests |
| Normative specifications and vectors | `develop/specs/` and `develop/specs/vectors/` |
| Traceability/status ledger | `develop/docs/csharp-03-implementation-traceability-ledger.md` |
| Probes, migration reports, receipts, and per-item review attachments | `develop/migrations/csharp-03/` |
| Build inputs and immutable bundles | `release/build-inputs/csharp/` and `release/bundles/` |
| Practical examples | `examples/csharp-practical/` |
| Existing policy fixture to repair | `fixtures/csharp/policy/` |

New files named in the task tables are expected test/owner names. T01 may
rename them once, before downstream implementation, and must update this file
and the traceability ledger atomically. Production logic must not live only in
test harnesses.

Until that one permitted T01 rename, “the CSHARP-03 ledger” below means
`develop/docs/csharp-03-implementation-traceability-ledger.md`. Machine-
readable baselines, inventories, receipts, probe results, and review-finding
attachments referenced by it live under `develop/migrations/csharp-03/`.

The following existing files are the mandatory starting points for the
corresponding changes. An implementation may split a large module, but it must
retain one obvious owner and update the T01-W02 consumer inventory in the same
work item.

| Concern | Existing starting point |
| --- | --- |
| C# transport/session/diagnostics | `Capture.cs`, `Selection.cs`, `RoslynSession.cs`, `RoslynAdapters.cs`, `FrontendProtocol.cs`, `FrontendDiagnostics.cs`, `FrontendLimits.cs` |
| C# subset/contracts/lowering/emission | `SubsetValidator.cs`, `SubsetModel.cs`, `ContractModel.cs`, `ContractParser.cs`, `ContractAttachment.cs`, `LoweringModel.cs`, `LoweringBuilder.cs`, `VirEmitter.cs`, `SourceMapEmitter.cs`, `SourceManifestEmitter.cs` under `csharp-tools/csharp2vir/` |
| Registry and source artifacts | `crates/mpk-vc/src/semantic_profile_registry.rs`, `successor_source_artifacts.rs`, `source_map.rs`, `source_manifest.rs` |
| VIR and VC | `crates/mpk-vc/src/vir.rs`, `vir_validate.rs`, `vir_canonical.rs`, `successor_vc.rs`, `vc_skeleton.rs`, `vc_canonical.rs` |
| CLI/frontend/release | `crates/mpk-cli/src/successor_frontend_protocol.rs`, `successor_frontend_runner.rs`, `frontend_sandbox.rs`, `successor_release_bundle.rs`, `successor_cli.rs` |
| Policy/evidence/AI/API | `crates/mpk-cli/src/successor_policy.rs`, `successor_ai_explain.rs`, `program_certificate.rs`, and `crates/mpk-api/src/successor_api.rs` |
| Installed registry/bundles | `release/bundles/semantic-profile-registry.json`, `bundle-registry.json`, and `release/bundles/candidates/` |

Nothing under the tracked `fixtures/csharp/policy/` path may count as CSHARP-03
frontend, policy, evidence, certificate, AI, or release acceptance evidence
until T08-W10 atomically replaces the complete linked fixture. Earlier tasks
use dedicated new fixtures and retain the old fixture only as a negative
mismatch case. T08-W01 stages the verified replacement under private migration
evidence and does not modify the tracked fixture path.

## 4. Common definition of done

The requirements below are cumulative only after their implementation owner
exists. Passing an earlier gate neither requires nor authorizes implementing a
later-stage owner early. Apply them at stage gates as follows:

| First applicable gate | Scope that becomes executable at that gate |
| --- | --- |
| T01 | frozen probes, specifications, vectors, hashes, limits, names, and later production-test ownership only; no production importer, source run, proof discharge, candidate, or public-route claim |
| T02 | private test-injected registry, foundation, artifact, importer, VC-schema, protocol, and predecessor-migration guarantees; no practical C# source-acceptance or installed-route claim |
| T03 | actual captured loop-free C# source-capability runs, direct collection representations, MPK-free application build/output inspection, source maps/manifests, generic rejection, specialization integration, monomorphic data emission, and production typed handoffs for later boundary/control owners; loop-dependent collection algorithms and boundary invocation remain deferred |
| T04 | source control, loop-dependent sequence/map/set algorithms, pattern, explicit throw/handler, and exceptional-region guarantees |
| T05 | boundary/transition source and sidecar guarantees, including structural rejection of every partial method on a total root closure; proof discharge remains T06-owned |
| T06 | ordinary VC/proof construction, totality discharge, Certificate v0, both-checker, policy/evidence, AI, and structured-API guarantees on private routes |
| T07 | registered private candidate, reproducible bundle, native sandbox, hostile-environment, and candidate installed-image guarantees |
| T08 | checked-in examples, complete aggregate/release evidence, active installed image, rollback, and final public-route guarantees |

At each gate, every requirement whose first applicable gate has been reached
must hold for the capability completed so far and remain true at every later
gate. When one requirement names artifacts introduced at different stages, it
applies artifact by artifact as each owner lands; for example, T03 proves the
generic-free VIR barrier and T06 extends that proof to VC and certificate
inputs:

- positive, negative, boundary, precedence, deterministic, mutation, and
  compiler/runtime-upgrade cases have exactly one implementation-test owner;
- accepted source runs twice from isolated inputs and emits byte-identical
  canonical artifacts; applicable cases also compare against the pinned .NET
  runtime over a finite domain;
- source maps cover original bytes, manifests account for every selected and
  reachable input, and hashes bind the exact semantic context and profile;
- the ordinary application build remains unchanged and contains no MPK
  package, namespace, assembly reference, attribute, interface, base type,
  generated source, or runtime component;
- every user-defined generic and unsupported constructed CLR type rejects,
  the exact value-type `T?` exception is specialized, and no generic or
  semantic-template node survives in VIR, VC, certificate, or checker input;
- the importer independently validates the registered foundation bytes and
  recomputes the descriptor hash, derived instance closure, identities,
  canonical order, deduplication, limits, and concrete expansion;
- failures are artifact-free, bounded, sanitized, and follow the frozen phase
  and within-method diagnostic precedence;
- no project file, NuGet resolution, ambient assembly, culture, clock,
  randomness, network, credential, mutable global state, task scheduler, or
  host path can affect an accepted result;
- iterator, `yield`, framework enumeration protocol, async, await, task, and
  async-iterator cases have rejection and compiler/runtime-upgrade coverage
  only;
- every boundary, transition, example, and public practical-profile root and
  its reachable call/loop closure has a discharged total-termination claim;
- predecessor Go, Rust, scalar C#, and Java source behavior, obligations,
  checker verdicts, and axiom inventory are unchanged; and
- the CSHARP-03 ledger has no open finding for the completed stage, and every
  linked per-item review attachment is closed.

The final stage additionally requires actual installed-source examples,
same-byte acceptance by both checkers, the complete native x86-64 Linux gate,
atomic activation, and a tested whole-image rollback procedure.

## 5. Milestone map

| Stage | Work items | Deliverable | Gate |
| --- | ---: | --- | --- |
| `CSHARP-03-T01` | 10 | measured feasibility record, frozen practical profile package, exact vectors and ledger | no unresolved or guessed fact |
| `CSHARP-03-T02` | 9 | private successor shared-artifact foundation and equivalent predecessor migration | no public route; all predecessor equivalence cases pass |
| `CSHARP-03-T03` | 14 | loop-free data frontend and collection/boundary representation substrates | every loop-free source capability and direct collection form lowers from actual C# source; internal typed handoffs and all deferred control/boundary cases have exact later owners |
| `CSHARP-03-T04` | 6 | loops, loop-dependent collection algorithms, switch/pattern, and explicit exceptional CFG | normal and abrupt control plus every deferred collection path are independently validated |
| `CSHARP-03-T05` | 6 | canonical boundary and pure transition frontend | overlay linkage, round trip, total-root structural eligibility, version, and optional idempotency validation pass; proof discharge remains T06-owned |
| `CSHARP-03-T06` | 12 | VC, certificate, semantic-binding, policy/evidence/AI/API integration | same certificate bytes pass both checkers with zero axioms |
| `CSHARP-03-T07` | 6 | reproducible registered private candidate and native sandbox evidence | two builds/runs and hostile-environment gates pass |
| `CSHARP-03-T08` | 10 | repaired evidence, three examples, complete rehearsal, atomic activation | sole installed successor release; zero findings |

### 5.1 Primary test routing

The paths below are the planned primary test executors. T01 rows execute only
probe, schema, specification, and routing checks and do not count as production
execution; T01-W10 additionally assigns every frozen vector row to exactly one
downstream T02-T08 production executor. Multiple work items may share a test
binary, but each owns a distinct test module/prefix matching its full W ID. A
primary owner is the exact pair of one path and one full-W-ID prefix;
additional regression commands do not create a second owner. T01-W10 freezes
all exact names.

| Work items | Primary test owner |
| --- | --- |
| T01-W01/W02 | `crates/mpk-vc/tests/csharp_practical_inventory.rs` |
| T01-W03 | `crates/mpk-cli/tests/csharp_practical_build_inputs.rs` |
| T01-W04-W07 | `crates/mpk-cli/tests/csharp_practical_probes.rs` |
| T01-W08-W10 | `crates/mpk-vc/tests/csharp_practical_spec.rs` |
| T02-W01 | `crates/mpk-vc/tests/csharp_practical_registry.rs` |
| T02-W02/W03 | `crates/mpk-vc/tests/csharp_practical_vir_model.rs` |
| T02-W04 | `crates/mpk-vc/tests/csharp_practical_source_artifacts.rs` |
| T02-W05 | `crates/mpk-vc/tests/csharp_practical_vir_validation.rs` |
| T02-W06 | `crates/mpk-vc/tests/csharp_practical_vc_model.rs` |
| T02-W07 | `crates/mpk-cli/tests/csharp_practical_frontend_protocol.rs` |
| T02-W08/W09 | `crates/mpk-cli/tests/csharp_practical_migration.rs` |
| T03-W01 | `crates/mpk-cli/tests/csharp_practical_capture.rs` |
| T03-W02 | `crates/mpk-cli/tests/csharp_practical_syntax.rs` |
| T03-W03-W06 | `crates/mpk-cli/tests/csharp_practical_types.rs` |
| T03-W07-W09 | `crates/mpk-cli/tests/csharp_practical_collections.rs` |
| T03-W10 | `crates/mpk-cli/tests/csharp_practical_codecs.rs` |
| T03-W11 | `crates/mpk-cli/tests/csharp_practical_numbers.rs` |
| T03-W12-W14 | `crates/mpk-cli/tests/csharp_practical_domain.rs` |
| T04-W01-W06 | `crates/mpk-cli/tests/csharp_practical_control.rs` |
| T05-W01-W03 | `crates/mpk-cli/tests/csharp_practical_boundary.rs` |
| T05-W04/W05 | `crates/mpk-cli/tests/csharp_practical_transition.rs` |
| T05-W06 | `crates/mpk-cli/tests/csharp_practical_boundary_transition.rs` |
| T06-W01-W09 | `crates/mpk-vc/tests/csharp_practical_vc.rs` |
| T06-W10 | `crates/mpk-cli/tests/csharp_practical_policy_verify.rs` |
| T06-W11 | `crates/mpk-api/tests/csharp_practical_api.rs` |
| T06-W12 | `crates/mpk-cli/tests/csharp_practical_end_to_end.rs` |
| T07-W01 | `crates/mpk-cli/tests/csharp_practical_build_inputs.rs` |
| T07-W02 | `crates/mpk-cli/tests/csharp_practical_release_bundle.rs` |
| T07-W03/W04 | `crates/mpk-cli/tests/csharp_practical_frontend_runner.rs` |
| T07-W05/W06 | `crates/mpk-cli/tests/csharp_practical_release_gate.rs` |
| T08-W01 | `crates/mpk-cli/tests/csharp_practical_fixture.rs` against the private candidate |
| T08-W02-W05 | `crates/mpk-cli/tests/csharp_practical_examples.rs` |
| T08-W06-W09 | `crates/mpk-cli/tests/csharp_practical_release_gate.rs` |
| T08-W10 | `crates/mpk-cli/tests/successor_atomic_cutover.rs` |

## 6. CSHARP-03-T01 — Feasibility and normative freeze

T01 changes specifications, vectors, private probe/test harnesses, private
build-input measurement descriptors under `develop/migrations/csharp-03/`,
and the ledger only. It must not change production acceptance, active
release/build descriptors, or install/register a candidate. All observed
values are captured from the final pinned toolchain; documentation recollection
is not evidence.

### CSHARP-03-T01-W01 — Close the entry audit and baseline receipt

Depends on: `JAVA-03-T10`.

Owns: confirm Java activation; capture the installed release registry and all
active tuple/hash/checker identities; record the exact Go/Rust/scalar-C#/Java
test corpus, release command, axiom inventory, and clean-worktree commit in
`develop/migrations/csharp-03/baseline.json`; create the ledger with every W
item in this document at
`develop/docs/csharp-03-implementation-traceability-ledger.md`.

Exit gate: the receipt is reproducible from the active image, `JAVA-03-T10` is
linked by commit and receipt hash, every later item has one owner/status row,
and no CSHARP-03 identity exists in an active registry.

Verification: ledger schema test plus the complete currently documented local
release status checks and `./scripts/check-fast.sh`.

### CSHARP-03-T01-W02 — Inventory every affected artifact and consumer

Depends on: T01-W01.

Owns: enumerate the current registry, semantic context, parameter, selection,
profile contract, source artifact, foundation, VIR, frontend protocol, maps,
manifests, VC/skeleton, release, policy/evidence, program assembly, AI, and API
identities; map every producer, validator, serializer, parser, hash domain,
bundle member, CLI/API route, fixture, and test that consumes them.

Exit gate:
`develop/migrations/csharp-03/artifact-consumer-inventory.json` has no unowned
read/write/hash edge, distinguishes active and private routes, and identifies
the single atomic migration set and whole-image rollback set.

Verification: repository search fixtures prove that deleting or adding a
known consumer makes the inventory test fail.

### CSHARP-03-T01-W03 — Pin and prove the frontend/toolchain closure

Depends on: T01-W02.

Owns: measure and freeze the .NET SDK/runtime, Roslyn assemblies/analyzers,
reference pack, native host, source/build flags, archive hashes/modes/notices,
offline extraction, and exact deterministic build inventory; prohibit project
files, restore, NuGet/package-cache discovery, generators, analyzers outside
the inventory, and ambient references. Store the private candidate descriptor
and inventory as
`develop/migrations/csharp-03/build-inputs/build-inputs.json` and
`develop/migrations/csharp-03/build-inputs/candidate-inventory.json`; the T01
harness consumes only those paths and must not rewrite
`release/build-inputs/csharp/`.

Exit gate: two clean offline builds from frozen bytes are identical; every
one-byte, file-count, mode, flag, reference, or declared environment-closure
mutation fails before publication, while an ambient setting outside that
closure is ignored and leaves the output bytes unchanged; the candidate
remains unregistered.

Verification: add `crates/mpk-cli/tests/csharp_practical_build_inputs.rs`,
extend the build-input script tests, run
`cargo test -p mpk-cli --test csharp_practical_build_inputs` against the two
private paths, run the isolated two-build recipe, and run
`./scripts/build-csharp-frontend.sh --check-build-inputs` to prove the active
scalar inputs remain unchanged. The currently installed native
`./scripts/check-java-frontend.sh` gate remains predecessor baseline evidence;
it is not a T01 practical toolchain probe. T01-W10 freezes the exact practical
candidate/release-gate command and its replacement, extension, or retirement
relation to the current gate; native execution of the practical gate is
reserved for T07/T08.

### CSHARP-03-T01-W04 — Measure Roslyn data and construction shapes

Depends on: T01-W03.

Owns: disposable probes for expression bodies, `var`, enums, readonly structs,
sealed immutable classes, fields/properties/constructors and same-type
constructor delegation, pure instance-method declarations/calls and overload
resolution, `init`, `required`, object initializers, synthesized members,
ordinary namespace `using`, the exact redundant file-wide `#nullable enable`,
nullable annotations, exact value-type `T?`, conditional access/coalescing,
conversions, arrays, collection calls, strings including restricted
interpolation, compiler-owned init/required markers, incidental generic
metadata on allowlisted intrinsics, and every proposed data intrinsic.

Exit gate: canonical probe output records syntax, symbols, `IOperation`, CFG
where present, implicit/synthesized flags, source spans, and rejected near
misses; every admitted shape has a distinct upgrade mutation.

Verification: a pinned compiler rerun is byte-identical and a fixture with one
changed observation fails the probe-schema test.

### CSHARP-03-T01-W05 — Measure Roslyn control, exception, and pattern shapes

Depends on: T01-W04.

Owns: disposable probes for all admitted loops, break/continue, switch
statements/expressions, guards, null/type/property/list patterns, throw,
try/catch/filter/finally, region nesting, lexical handler search, filter
failure, and abrupt completion.

Exit gate: the canonical decision/region/operation observations cover every
accepted and rejected source form, source ordering is explicit, and no
compiler-internal/private API is required.

Verification: pinned rerun equality plus decision-graph and exception-region
upgrade mutations.

### CSHARP-03-T01-W06 — Freeze dependency, generic, iterator, and async rejection observations

Depends on: T01-W05.

Owns: disposable Roslyn probes for every MPK package, assembly, namespace,
attribute, interface, base-type, generated-source, project, and ambient
dependency form; every source-written attribute versus the exact compiler-
synthesized init/required metadata observations; every user generic
declaration, method, type parameter, constraint, variance, explicit/inferred
generic call, closed use, and arbitrary constructed CLR/BCL type; the exact
compiler-owned value-type `T?` exception and rejected explicit
`System.Nullable<T>` spelling;
allowlisted intrinsics whose metadata incidentally mentions generics; and every
iterator, `yield`, enumeration-protocol, async, await, task/value-task,
awaiter, cancellation, and generated-state-machine shape.

Exit gate: each dependency and generic category has a frozen symbol/operation
observation and deterministic rejection family; exact `T?` remains the sole
constructed-generic source exception and is immediately specialized; an
allowlisted intrinsic remains admitted only through its exact selected symbol
and operation while any source-visible use of transitive generic metadata
rejects. Iterator and async forms have negative and upgrade vectors only; no
suspension semantics or positive iterator/async capability is frozen.

Verification: pinned-compiler rerun equality, one mutation per dependency and
generic family, exact-`T?` positive/near-miss cases, incidental-metadata
non-expansion cases, and complete iterator/async rejection vectors.

### CSHARP-03-T01-W07 — Measure primitive, string, numeric, and codec behavior

Depends on: T01-W06.

Owns: finite runtime probes for UTF-16/surrogates, ordinal string operations,
the exact string/char concatenation matrix and restricted-interpolation
normalization, null behavior, parse/format grammars, float/double result bits
including NaN and signed zero, decimal coefficient/scale/rounding/overflow,
and culture-independent error precedence.

Exit gate: every operation has an exact accepted domain, result encoding,
error, precedence, and differential vector; no result depends on ambient
culture or general framework parsing/formatting.

Verification: run under at least two hostile cultures and mutated runtime
inputs; compare canonical bits/values and rejection IDs.

### CSHARP-03-T01-W08 — Freeze foundation, specialization, binding, and data semantics

Depends on: T01-W07.

Owns: the final declaration, stored-member, constructor, `init`/`required`, and
object-initializer forms; construction/public invariants and receiver-first
statically resolved pure instance calls; arrays, count-then-allocate sequence
construction, application-owned ordered entry/map/set representations,
ownership/publication, duplicate add/replace precedence, recursive default
eligibility, structural equality, and the total-key matrix. Freeze the one
candidate foundation descriptor intended for later T07/T08 registration, its
member inventory, content-hash domain, operation sets, expansion definitions,
counters, and exact internal template registry: `bounded_sequence<T>`,
`sequence_construction<T>`,
`ordered_entry<K,V>`, `ordered_map<K,V>`, `ordered_set<T>`, `option<T>`,
`lookup<T>`, `result<T,E>`, `validation<T,E>`, `boundary_field<T>`,
`transition<S,E,R>`, and `money<C>`. Freeze `unit`, `parse_error`, the internal
instant, and the closed exception sum as separate non-template definitions in
that candidate descriptor; T01 does not install or activate it.
Freeze the application semantic-binding schemas and projection obligations,
as well as nullable/closed-outcome, date/time/duration/instant/GUID/money
representations, operations, codecs, ranges, and errors.

Exit gate: every template identity, arity, direct dependency, derivation
source, operation, and ordinary-core expansion is explicit; no caller-supplied
template or allowlist exists. The per-compilation algorithm freezes concrete
argument admissibility, transitive closure, canonical instance identity,
sorting, deduplication, source-binding provenance, count/depth/expansion
limits, and pre-VIR monomorphization. Every application binding freezes exact
role/member/tag/payload/carrier IDs, inferred closed argument IDs, actual
default arm, bounds, binding hash, total arm-distinct projection round trips,
operation commutation, and rejection of missing, stale, colliding, cyclic, or
unreachable entries. All source APIs remain ordinary non-generic application
code; no source-visible foundation or builder API is introduced.

Verification: descriptor/member/hash, template/dependency/expansion,
closed-instance closure/order/dedup/limit, binding/projection/identity, and
residual-generic vectors; spec-model completeness tests; and independent
finite .NET differential probes for every runtime-backed row.

Completed handoff: `develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md`, the
candidate descriptor/definitions in `develop/migrations/csharp-03/foundation/`,
`develop/specs/vectors/csharp-practical-foundation-v1.json`, and
`develop/migrations/csharp-03/probes/runtime-foundation-data.json`. The primary
owner is `crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W08`;
ledger section 10 records exact hashes, verification and the review/fix loop.

### CSHARP-03-T01-W09 — Freeze contracts, boundary, transition, identities, and limits

Depends on: T01-W08.

Current status: `Complete`. The private feasibility probe retains the original
cross-result rejections and demonstrates the replacement Boolean-cube
selection and concrete state-transformer fold in both checkers. F01 is resolved
without changing core or using `Std.Nat.rec` in the replacement. The private
freeze fixes all 17 successor identity families, 15 strict roots, 20 strict
nested records, three tagged unions, the fully typed closed expression union,
registry-entry/context/frontend equality linkage, canonical
boundary/transition/idempotency rules, 29
diagnostic families, 35 practical limits, and all retained scalar-v0 limits.
The 700 sorted private vectors assign every row to its downstream implementation
and primary production-test owner. The capacity record covers four generated-
certificate counters at limit minus one, limit, and limit plus one through both
checkers twice.

Owns: final type/method/semantic-binding/boundary/transition sidecar schemas
and expression union; verification-overlay handoff; missing/null/value
canonical JSON; raw-input/canonical-value and canonical-output/reparsed-value
linkage; transition version, ordered-event, response, and error precedence;
optional idempotency over complete retained application-owned `Command` and
`Context` snapshots; source equality equivalent to canonical field encodings;
rejection of float/double-containing or otherwise non-reflexive snapshots; all
successor identity and hash-domain names, including the successor
program-assembly profile; whether one context-dispatched C# executable safely
serves both scalar and practical profiles; diagnostic families/precedence;
exact measured limits classified as pre-invocation structural counters or
admitted run-time value predicates/VCs; total-termination requirements; and
producer/consumer ownership from T01-W02.

Exit gate: no “provisional”, “working”, or unmeasured value remains in a
normative row; unknown fields/tags reject; each counter has one increment site,
one comparison rule, and boundary vectors; all final names are globally unique
and immutable. The boundary document is explicitly an MPK verification-overlay
transport rather than an application protocol, and an idempotency claim is
unavailable unless both complete retained snapshots and their reflexive
field-complete equality relations satisfy the frozen rules.

Verification: strict schema/hash/limit/precedence tests, later-version and
duplicate-key mutations, and checker capacity probes at limit minus one,
limit, and limit plus one.

Completed handoff:
`develop/migrations/csharp-03/freeze/profile-freeze.json`,
`develop/migrations/csharp-03/freeze/profile-freeze-vectors.json`, and
`develop/migrations/csharp-03/probes/checker-capacity.json`. The primary owner
is `crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W09`; ledger
section 12 records exact hashes, verification, and the review/fix loop. These
are private candidate inputs to W10, not published or installed schemas.

### CSHARP-03-T01-W10 — Publish and review the complete freeze package

Depends on: T01-W09.

Current status: `Complete`.

Owns: the practical-profile specification package, successor shared-artifact
specifications, exact vectors and manifest entries, canonical probe records,
name-and-owner inventory, traceability ledger, upgrade matrix, and routing
updates in the design/todo/developer documentation; freezes the exact primary
test owner pairs and the exact local practical candidate/release-gate command
name and owner used by T07-W05/W06 and T08-W06/W09/W10, including whether that
command replaces, extends, or retires the currently installed Java-named gate
at atomic activation.

Exit gate: every freeze-only requirement maps to its exact T01 W item and one
probe/specification-test owner; every implementation or release requirement,
vector row, diagnostic, limit, implementation surface, and release criterion
maps to one primary downstream implementation W item, any separately named
verification/activation owners, and one primary production-test owner pair.
All hashes are recomputed; a complete specification review produces zero
findings. Production behavior is unchanged.

Verification: `python3 scripts/check-spec-vectors.py --check`, manifest tests,
`./scripts/check-fast.sh`, and a documented second-pass review after all fixes.

Completed handoff: `develop/specs/CSHARP_PRACTICAL_PROFILE_V1.md`,
`develop/specs/CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md`, and manifested vector
`develop/specs/vectors/csharp-practical-profile-v1.json`. The package preserves
the W09 freeze and all 700 rows exactly, binds 16 canonical probe/evidence
records, closes ten freeze owners and 63 downstream production owners, and
records the twelve-family upgrade matrix plus exact future release-gate
replacement decision. The primary owner is
`crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W10`; ledger section
13 records hashes, verification, non-activation, and both zero-finding review
passes.

## 7. CSHARP-03-T02 — Shared artifact foundation

T02 is private and test-injected. It may create successor models and candidate
adapters, but no public CLI/API route, installed tuple, compatibility flag,
dual-registry lookup, or ambient staging-root discovery.

### CSHARP-03-T02-W01 — Implement the closed successor registry and context

Depends on: T01-W10.

Current status: `Complete`.

Owns: strict transport/root/entry hashing, revision/order/limits, semantic
context, parameters, selection, compiled-profile envelopes, immutable profile
dispatch, registered foundation-descriptor identity/content-hash linkage,
installed-identity assertions, and append-only predecessor checks.

Exit gate: every frozen identity/hash/mutation/context vector runs through
production validation code; unknown and mixed language/profile pairs reject;
the released registry still exposes only the predecessor revision.

Verification: successor registry runtime tests plus all revision-1/2/3
predecessor vector tests and `successor_atomic_cutover` rejection cases.

### CSHARP-03-T02-W02 — Implement the registered foundation bundle and closed instances

Depends on: T02-W01.

Current status: `Complete`.

Owns: the one frozen descriptor schema and hash domain; the exact twelve
internal templates and four non-template definitions frozen by T01-W08;
template identities, arities, operations, dependency and expansion rules;
canonical descriptor/member encoding; the closed root/provenance input schema
and derived closed-instance entries; a reusable specialization engine whose
inputs are one validated registered descriptor and one explicit closed
root/provenance set; and monomorphic representations for enums, immutable
products,
arrays/sequences, ordered entries/maps/sets, strings/chars, float/decimal bits,
nullable/tagged sums, boundary presence, dates/times/duration/internal instant,
GUID/money, transitions, parse errors, and closed exceptions.

Exit gate: production validation accepts only the registered descriptor and
independently recomputed bundle content hash. Given an explicit validated
root/provenance set, it derives only instances reachable from those roots,
recursively closes dependencies, derives canonical identities, sorts and
deduplicates them, enforces all counters, and expands them into concrete
definitions. No caller bundle, caller allowlist, framework object, serializer
representation, source-facing MPK type, or generic/template value can enter
VIR. T02 executes this engine only with private test-injected root/provenance
sets. T03-W14 first derives the data-phase roots from actual C# source and data
sidecars; T04-W06 and T05-W06 extend that same cumulative root set for their
newly admitted control/exception and boundary/transition forms.

Verification: descriptor/hash/member mutations; every template arity,
dependency, operation, and expansion; closure/order/dedup/depth/count limits;
unreachable or caller-injected instances; residual-generic rejection; and
canonical concrete-value round trips in `csharp_practical_vir_model.rs`.

### CSHARP-03-T02-W03 — Implement operation and explicit-control vocabulary

Depends on: T02-W02.

Current status: `Complete`.

Owns: closed operation/check tags for the frozen data APIs and boundary codecs;
the monomorphic linear construction state expanded from each derived concrete
`sequence_construction` instance; application-binding
projection/reconstruction operations; loop/control nodes; pattern decisions;
explicit exception values, handler regions, abrupt edges; and unknown-tag
rejection. No iterator, async, scheduler, task, or suspension vocabulary is
introduced.

Exit gate: operand/result concrete types, ordered checks, normal/exceptional
successors, construction ownership state, and application-binding operation
commutation are structurally validated without invoking the C# runtime.

Verification: table-driven operation/control vectors and one mutation for
every tag, arity, type, edge, and ordering rule.

### CSHARP-03-T02-W04 — Implement successor source artifacts and linkage

Depends on: T02-W03.

Current status: `Complete`.

Owns: frozen selection/type/method/semantic-binding/boundary/transition
artifacts; registered foundation descriptor/hash linkage; the complete derived
closed-instance table with source-binding provenance; concrete operation/check
tables; original-source capture; raw-boundary-byte/canonical-value and
canonical-output/reparsed-value linkage; context linkage; source-map and both
manifest stages; canonical serialization; and every hash domain.

Exit gate: no artifact can be mixed across source/profile/context/schema or
boundary bytes/value/output; every selected/reachable declaration and input is
accounted for exactly once.

Verification: successor source-artifact tests covering field order,
duplicate/unknown fields, path/span coverage, hash substitution, missing
members, and cross-profile splicing.

### CSHARP-03-T02-W05 — Implement the successor VIR importer and validator

Depends on: T02-W04.

Current status: `Complete`.

Owns: strict parsing, resource counters, stable ID/reference validation,
dominance/control-region checks, finite acyclic call/type graphs, ownership,
exception structure, application-binding linkage, complete foundation/
operation linkage, and the generic-free VIR barrier.

Exit gate: only frozen vocabulary is admitted; malformed or over-limit inputs
fail before VC construction; import has no compiler/runtime callback. The
importer independently validates registered foundation bytes/hash, recomputes
closed-instance reachability, dependencies, identity, order, deduplication,
limits, and concrete expansion, and rejects any type parameter, generic
definition, constructed generic, generic call, or semantic-template node. Old
parsers reject successor bytes and successor parsers reject old-family bytes
where accepting both would be ambiguous.

Verification: `csharp_practical_vir_validation.rs`, bounded parser fuzz seeds,
and cross-artifact/profile/hash mutation suites.

### CSHARP-03-T02-W06 — Implement successor VC skeleton, VC, and hash models

Depends on: T02-W05.

Current status: `Complete`.

Owns: the successor skeleton/VC schemas, canonical ordering, context and VIR
linkage, ordinary-term type/value encoding, obligation groups, limits, and
hashes; the distinct successor program-assembly profile; and structural
enforcement of its foundation/context linkage while leaving Certificate v0 and
checker acceptance unchanged.

Exit gate: all new values and control forms have a closed encoding path. The
practical-profile assembly structurally enforces empty proof/theory tables and
zero axioms; predecessor assemblies retain their frozen table and axiom rules;
no intrinsic lacking a frozen ordinary-term encoding and later proof owner is
structurally admitted. T02 performs no proof discharge.

Verification: successor VC/hash vectors, canonical byte mutations, and direct
rejection of nonempty proof-node/theory-certificate tables.

### CSHARP-03-T02-W07 — Implement successor frontend protocol, maps, and manifests

Depends on: T02-W06.

Current status: `Complete`.

Owns: private request/result envelopes, phase/status precedence, sanitized
diagnostics, artifact-free failure, source-map original-byte coverage, frontend
and certificate manifests, inventory accounting, and deterministic output
ordering.

Exit gate: success returns the complete frozen artifact set; any failure
returns none; raw compiler prose, snippets, paths, stack text, culture, and
generated type names cannot escape.

Verification: protocol/map/manifest conformance and fuzz tests, including
truncation, duplicates, unknown versions, oversize values, and phase conflicts.

### CSHARP-03-T02-W08 — Migrate all predecessor producers privately

Depends on: T02-W07.

Current status: `Complete`.

Owns: private adapters for active Go, Rust, scalar C#, and Java producers to the
sole successor artifact family, including exact context propagation and
canonical regeneration from their real frontends.

Exit gate: each predecessor corpus produces successor artifacts with identical
source behavior, obligations, verdicts, and axiom inventory; no producer can
choose old versus new artifacts via a public flag. A predecessor gains no
practical foundation instance unless its frozen source/profile semantics
already require the same concrete definition.

Verification: per-language semantic-difference reports, two-run artifact
equality, and old/new interpreter/VC/checker comparison over the complete
accepted and rejected corpora.

### CSHARP-03-T02-W09 — Migrate consumers and close the private foundation gate

Depends on: T02-W08.

Current status: `Complete`.

Owns: private successor consumers in VC, certificate assembly, policy/evidence,
CLI, release, AI explanation, API, fixture tooling, and documentation. The
release consumer includes strict private successor release-root, tuple,
frontend/toolchain/foundation descriptor, member-inventory, linkage, and hash-
domain validation over test-injected descriptors; T07-W02 alone materializes
the final candidate bundles and tuple. This item removes private dual-format
fallbacks and records the complete migration receipt.

Exit gate: every T01-W02 inventory edge consumes exactly one successor format,
including the release and foundation-bundle edges needed by T06 and T07;
predecessor equivalence tests pass, no release candidate or public practical
route exists, and T02 review has zero findings.

Verification: targeted consumer tests, inventory search/mutation tests,
complete predecessor semantic/certificate regression suites, and
`./scripts/check-fast.sh`. Native installed-image gates remain owned by T07 and
T08.

## 8. CSHARP-03-T03 — Loop-free data frontend and collection substrates

Every T03 source-acceptance case starts as immutable captured `.cs` and exact
JSON sidecars, passes the pinned Roslyn public APIs, and is independently
validated after emission. A helper-constructed VIR is not source-acceptance
evidence. Production typed tests may exercise an internal handoff whose first
source/control/boundary consumer belongs to T04 or T05, but the handoff test
cannot claim that later capability. T03 may emit the frozen closed exceptional
successor of a data operation using the T02 vocabulary, but it does not admit
source-declared exceptions, explicit
`throw`, `catch`, filters, or `finally`; those source forms and their region
semantics belong to T04. Each T03 feature item extends source-side type/method
sidecar parsing and attachment for its owned expression forms; T03-W14 closes
that data-sidecar union before T04 begins. T04-W06 and T05-W06 extend the
closed root/provenance set for their newly accepted forms by invoking the same
T02-W02 specialization engine; neither later stage may fork its algorithms.
Because source loops are not admitted until T04, T03 accepts only loop-free
direct collection representations and straight-line construction paths.
T03-W08/W09 implement their data representations, bindings, and handoff
contracts; T04-W01/W02 own the first positive source acceptance and lowering
of count-then-allocate, membership, lookup, traversal, aggregation, sort, and
dedup loops, and T04-W06 closes their actual-source emission evidence.

### CSHARP-03-T03-W01 — Extend capture, declaration accounting, and closure

Depends on: T02-W09.

Current status: `Complete`.

Owns: selected source roots, immutable byte capture, encoding/path checks, all-
declaration accounting for selected files, reachable method/type closure,
finite call/type graphs, recursion rejection, and closed framework symbol/API
admission; the pinned compiler-diagnostic gate, under which every active error
or warning fails closed while informational and hidden diagnostics follow the
frozen allow/ignore table; and global rejection of source delegate declarations
and values, method-group conversion, lambdas, expression trees, query/LINQ,
reflection or run-time code generation, and external-effect or concurrency
APIs. It rejects MPK package/assembly/namespace/attribute/interface/base/
generated-source dependencies, every user-defined generic declaration or
method and every closed use, and every arbitrary constructed CLR/BCL type
other than the exact value-type `T?` form; every source-written attribute also
rejects, while compiler-synthesized init/required markers are validated as
metadata observations and never become callable source APIs.

Exit gate: every ordinary declaration in a selected file, reachable or dead,
is accounted for and either satisfies the frozen declaration rules or causes
artifact-free rejection; every reachable declaration is captured.
Unselected/ambient/generated/project/package input, a failing compiler
diagnostic, and every globally excluded source or API family reject; selected
application output remains MPK-independent, source-visible transitive generic
metadata rejects, and closure limits are enforced before lowering.

Verification: extend C# capture/frontend vectors with multi-file, dead-tree,
unsupported-dead-declaration, compiler severity, cycle, source-root, path,
encoding, every MPK dependency form, delegate/lambda/expression-tree/query,
reflection/run-time-code-generation/effect/concurrency, user-generic and
constructed-type categories, incidental generic metadata, exact `T?`, ambient-
reference, and limit cases.

### CSHARP-03-T03-W02 — Normalize concise syntax and name resolution

Depends on: T03-W01.

Current status: `Complete`.

Owns: expression-bodied methods/getters, `var` locals whose exact admitted type
is available, the reusable exact-type normalization handoff later consumed by
T04-W02 for a `foreach` variable, ordinary non-global namespace `using`
directives, and the exact redundant file-wide `#nullable enable`; normalizes
the T03 forms to the same internal form as block bodies, explicit types, fully
qualified names, and the profile's already enabled nullable context. A
positive `foreach` source form, including `var` in that position, remains
T04-W02-owned.

Exit gate: normalized artifacts and obligations are byte-identical to explicit
equivalents; ambiguous, anonymous, dynamic, target-typed, disallowed, or
nullable-inconsistent inference rejects with frozen precedence. Global/static/
alias/generated imports, `extern alias`, disposal `using`, imported MPK
namespaces, scoped or non-enable nullable directives, and every other source
directive reject and emit no partial artifacts.

Verification: equivalence pairs and negative Roslyn-shape mutations in
`csharp_practical_syntax.rs`, including conditional-compilation and every
other rejected directive family, plus a routing assertion that positive
`foreach`-variable cases are not claimed before T04-W02.

### CSHARP-03-T03-W03 — Admit enum, readonly struct, and sealed immutable class declarations

Depends on: T03-W02.

Current status: `Complete`.

Owns: declaration modifiers, members, source enums including exact underlying
values, the closed metadata-backed `System.DayOfWeek` enum, immutable type
graphs, exact implicit object/value-type/enum bases for ordinary data
declarations, inheritance/identity/mutation rejection, and recursive default
eligibility. It classifies a source-exception candidate separately from an
ordinary immutable class and preserves its source identity for the next
stage, but T04-W04 is the sole owner of its exact base-clause validation,
admission, diagnostics, and lowering.

Exit gate: every admitted instance is a structural value; unknown enum values,
casts, zero/default cases, cycles, mutable/static/virtual/reflection/identity
escapes, and ineligible defaults follow frozen rules. Records, record structs,
`with`, partial or nested type declarations, unsupported base lists,
interfaces, operators, conversions, events, indexers, finalizers, source-
declared non-enum type constants, static fields/properties, type initializers,
weak-reference state, and mutable/unsafe/ref-like layouts reject.
No source exception declaration is an accepted or emitted T03 data output, and
none can be misclassified as an ordinary sealed class.

Verification: declaration/type/default matrix tests, ordinary-data-path
exclusion of source-exception candidates, partial/nested/static/member-shape
rejections, classifier handoff tests that make no base-clause acceptance claim,
and source/runtime differential vectors.

### CSHARP-03-T03-W04 — Admit fields, properties, constructors, and invariants

Depends on: T03-W03.

Current status: `Complete`.

Owns: the frozen field/property forms, getter normalization, acyclic
same-type constructor delegation, constructor selection and assignment order,
definite initialization, constructor-only construction, per-path member state
and pre-finalization `this`-use restrictions, receiver-first lowering and
static resolution of admitted pure instance methods, construction invariants,
and public type invariants; the exact frozen inert compiler-synthesized
parameterless class constructor; exact positional, by-value, closed parameter
and argument rules; and explicit `new ExactType(...)` construction.

Exit gate: all stored members follow the frozen assignment multiplicity and
order and establish exactly one final value; partial construction,
constructor-delegation cycles, setter/alias escape, hidden mutable state,
virtual/dynamic/ambiguous instance dispatch, invalid overload, and a missing,
ill-typed, wrongly attached, or incompletely emitted invariant claim reject;
semantic discharge of the emitted invariant obligations belongs to T06-W02.
Optional/default or `params` parameters, named arguments, extension syntax,
`ref`/`in`/`out`, target-typed or anonymous construction,
reflection/activator construction, and any other synthesized constructor/member
shape reject. Before finalization, a read of a not-definitely-assigned member,
an instance method/getter call on `this`, passing, returning, storing,
comparing, or capturing `this`, whole-`this` assignment, and a member reference
escape reject.

Verification: constructor delegation/member-order cases, receiver-first pure
instance-call equivalence, synthesized-constructor mutations, parameter/
argument/construction and pre-finalization-receiver rejection matrices,
invariant attachment, and failure-precedence tests.

### CSHARP-03-T03-W05 — Admit `init`, `required`, and object initializers

Depends on: T03-W04.

Current status: `Complete`.

Owns: ordered object-initializer lowering, required-member coverage, init-only
assignment state, duplicate/missing/member-order errors, and attribute-bypass
rejection.

Exit gate: successful initialization emits the same construction sequence,
finalization point, and construction/public-invariant obligations as the frozen
canonical form, and no value is usable before finalization; T06-W02 owns
semantic discharge. An exceptional initializer exit discards the unique
construction transaction and publishes no object value; post-construction
writes reject.

Verification: constructor-vs-initializer equivalence, required/init boundary
cases, exceptional-discard/non-observation, synthesized/attribute mutations,
and source-order assertions.

### CSHARP-03-T03-W06 — Lower structural equality and canonical ordering

Depends on: T03-W05.

Current status: `Complete`.

Owns: internal `structural_equal` and `canonical_compare` over every admitted
recursive value, null, decimal, GUID, sequences, structural products, enums,
and business values; admitted primitive/string source equality; and source-defined
non-generic pure field-by-field helpers included in the captured closure.
Enforces the frozen total-key matrix and lexicographic rules. The internal
operations are contract/boundary/foundation vocabulary, not callable `Mpk.*`
C# methods. W06 implements one type-directed production generator over the
T02 concrete descriptor vocabulary and exercises it through the source types
available by W06. W07-W13 must route each newly admitted collection, numeric,
nullable/outcome, and business-value type through that same generator and add
its actual-source vectors; they may not fork equality or ordering logic.

Exit gate: no CLR identity, virtual equality, hash code, comparer, locale, or
insertion order is observable; float-containing/non-total types reject as map
or set keys.

Verification: algebraic finite-domain tests, corner vectors, pinned-runtime
comparisons where the spec deliberately mirrors .NET, and a production-
generator routing test over every source type available by W06. W07-W13 extend
that same test for their types; W14 closes its completeness assertion over all
admitted actual-source types.

### CSHARP-03-T03-W07 — Lower arrays with explicit ownership

Depends on: T03-W06.

Current status: `Complete` (2026-09-05).

Completion evidence: ledger section 29; private ordered array ownership and
initialization plans, exact source forms, separate C# exception/profile-bound
edges, and the T04 read-borrow conflict handoff. No public frontend activation.

Owns: fixed/bounded allocation, default-eligible versus fully initialized
elements, reads/writes/length/index checks, alias rules, active-foreach
read-borrow/write-conflict handoff, and freeze/escape behavior. T04-W02 owns
the first positive `foreach` source form and applies that conflict rule. The
admitted T03 source forms are the exact explicit/implicit one-dimensional array
creation and initializer forms frozen by the design, with exact `int` lengths
and indices.

Exit gate: every access has ordered symbolic or structural bounds checks. A
negative or otherwise invalid C# length takes its frozen exception edge; a
constant/initializer above the profile maximum rejects structurally; a
symbolic length carries the separate profile-bound predicate and VC. Unique
mutable ownership cannot be duplicated, and every permitted transfer, return,
storage, or alias freezes/transfers it exactly as specified; non-defaultable
elements are initialized before read. Multidimensional, jagged, covariant,
`System.Array`, `Span<T>`, `Memory<T>`, stack-allocation, range/from-end,
target-typed collection, `Array.Empty<T>()`, and best-common-type array forms
reject before emission.

Verification: boundary lengths/indices, alias/use-after-freeze, foreach
mutation routing, default eligibility, every admitted creation form, and every
rejected array/storage/index form listed above. Actual `foreach` mutation
rejection executes under T04-W02.

### CSHARP-03-T03-W08 — Lower bounded-sequence representations and the construction substrate

Depends on: T03-W07.

Current status: `Complete` (2026-09-06).

Completion evidence: ledger section 30; private typed sequence plans, wrapper
projection, monomorphic construction elimination and source-to-VIR replay.
T04 retains every positive two-pass source loop; T06 retains proof discharge.

Owns: projection of admitted arrays and bound non-generic immutable wrappers to
closed internal bounded sequences; length, indexed read, structural equality,
and applicable lexicographic ordering; integration of the specialized
monomorphic construction state for loop-free fixed-size and straight-line
initialization; and the exact typed allocation/fill/freeze handoff consumed by
T04. The filtered count-then-allocate source form, its loop-contract
attachment, its CFG lowering, and its count/fill loop obligations first become
accepted under T04-W01/W02. T06-W03/W04 later discharge the emitted data and
loop obligations.

Exit gate: for every loop-free direct form, element initialization, exact
result length, profile bound, unique ownership, and final publication are
explicit and independently validated. The specialized monomorphic construction
state is explicit in VIR, cannot cross an incompatible merge or public
boundary, and publication eliminates it to an immutable value before
certificate generation. A source loop still rejects at the frozen T03 control
gate and emits no artifact; no T03 test may claim positive two-pass source
coverage. `List<T>`, immutable framework collections, single-pass growth,
callbacks/lambdas, and every source-visible builder API reject.

Verification: zero/full/direct-result and straight-line initialization cases,
premature publication/read, ownership transfer, profile-capacity boundaries,
determinism, residual-construction-state rejection, wrapper-binding mutations,
and a routing assertion that every positive two-pass loop vector is owned by
T04 rather than accepted in T03.

### CSHARP-03-T03-W09 — Lower canonical ordered maps and sets

Depends on: T03-W08.

Current status: `Complete` (2026-09-06); evidence: ledger section 31.

Owns: semantic binding of an admitted element array as a set and an array of an
application-owned non-generic immutable entry type as a map; optional bound
non-generic wrappers; direct canonical construction from an already strictly
ordered, duplicate-free admitted array; internal ordered map/set projection;
missing-key versus stored-null outcomes; canonical key ordering; key
admissibility; and the exact typed operation handoff consumed by T04. T04-W01/
W02 own source-loop contracts and lowering for count, membership, lookup,
ordered traversal, aggregation, duplicate rejection/replacement, and closed
sort/dedup algorithms. T06-W03/W04 later discharge the collection and loop
obligations.

Exit gate: every loop-free direct publication point carries the frozen bound,
strict-order, and uniqueness obligations. No insertion-order-independence or
loop-helper claim is made at T03; T04 adds those obligations from the captured
source bodies, and they are not considered discharged before T06-W03/W04.
Source loops still reject artifact-free at the T03 control gate. Custom
comparers, runtime hashing, float/non-total or mutable keys, generic entry
types, framework collections or enumeration, insertion-order assumptions, and
source-visible map/set builders reject.

Verification: entry/array/wrapper binding mutations, direct preordered and
duplicate-input cases, canonical-order tests over the key matrix, missing-
versus-stored-null projection, bounds, rejected comparer/hash/framework cases,
and routing assertions for all loop-dependent operation vectors.

### CSHARP-03-T03-W10 — Lower strings, characters, and boundary codec relations

Depends on: T03-W09.

Current status: `Complete` (2026-09-06); evidence: ledger section 32.

Owns: ordinal UTF-16 operations, allowed string methods, exact string/string and
string/char concatenation, restricted interpolation normalized to the same
bounded concatenation, null/empty rules, surrogate handling, and the frozen
culture-free boundary codec relations for all admitted primitives/business
values. Source string operations and internal boundary codecs remain distinct;
the codec relation does not admit a general source parse/format call. T03
implements one type-dispatched production codec relation over the T02 concrete
descriptor vocabulary and tests it independently of a boundary document.
W11 and W13 route each applicable newly admitted numeric or business-value
type through that same relation and extend its source-type coverage and
internal typed-codec vectors without forking a grammar or parser. W12 owns no
composite presence/outcome document codec; T05-W01-W03 own that document
mapping, the first canonical-document invocation, and end-to-end input/output
linkage.

Exit gate: only intrinsic constant ordinal options are accepted. Boundary codec
syntax, noncanonical, range, scale/precision, and input-bound failures return
their frozen internal `parse_error`; admitted null receiver/argument and bad-
index/range cases take exact exception/result paths; output/profile-length
predicates become VCs. Interpolation with alignment, a format component, or a
non-string/non-char hole; char/char; object conversion; source parse/format;
culture-sensitive/general framework calls; and unknown codec/rounding IDs
reject during source/contract validation.

Verification: grammar and round-trip corpus, hostile cultures, surrogate/null
boundaries, concatenation/interpolation equivalence and rejection matrices,
and pinned-runtime differential cases.

### CSHARP-03-T03-W11 — Lower float, double, and decimal

Depends on: T03-W10.

Current status: `Complete` (2026-09-06); evidence: ledger section 33.

Owns: exact float/double bit values and operations, NaN/signed-zero behavior,
decimal coefficient/scale representation, checked operations, rounding and
overflow, conversions, ordering exclusions, and applicable boundary codecs
through T03-W10's shared typed relation.

Exit gate: emitted operations reproduce every frozen runtime bit/value/error
vector without theory primitives or checker floating/decimal primitives;
unsupported casts/transcendentals/non-total ordering reject.

Verification: exhaustive small domains, edge bit vectors, decimal scale and
rounding tables, differential harness, and canonical encode/decode mutations.

### CSHARP-03-T03-W12 — Lower nullable and closed outcome values

Depends on: T03-W11.

Current status: `Complete`; see ledger section 34.

Owns: nullable reference operations; the exact compiler-owned value-type `T?`
source spelling and immediate `option<T>` specialization; the restricted
conditional-access and null-coalescing matrices; and semantic binding of
application-owned non-generic option, lookup, result, and accumulating-
validation representations, plus an application-owned three-arm
representation's data-phase projection to internal `boundary_field<T>`
semantics independently of any boundary document or field attachment. Includes
active/inactive payloads, actual default arm, missing/null/value distinction,
exhaustive matching, deterministic array-
based error order/bounds, nesting restrictions, and the one frozen
`lookup<option<T>>` exception.

Exit gate: annotations never substitute for runtime null; missing key differs
from stored null; inactive-payload access and empty `Invalid` construction take
their exact frozen exceptional edges and must be proved unreachable, caught,
or declared before verified acceptance. Each application binding emits the
complete typed totality, arm-distinction, observation-preserving projection/
reconstruction, and operation-commutation obligations; T06-W06 owns their
semantic discharge. Parameterless default fallback rejects when its payload is
not `default_eligible`, while an explicit fallback carries its public-invariant
obligation; nested `option` forms outside the frozen exception reject during
source/contract validation. `T?` near-misses and all other constructed generic
source types, including explicit `System.Nullable<T>`, reject.
T03 establishes no required/optional boundary-field mapping, canonical byte
parse/format claim, or boundary invocation; those belong to T05.

Verification: full arm/payload matrix, null/missing/value boundaries,
error accumulation order/capacity, match exhaustiveness, and mutation tests.

### CSHARP-03-T03-W13 — Lower calendar, time, GUID, instant, and money values

Depends on: T03-W12.

Current status: `Complete`; see ledger section 35.

Owns: exact DateOnly/TimeOnly/TimeSpan/GUID intrinsics; application-owned
non-generic instant wrappers bound to the registered non-template internal
instant definition; the reusable internal instant carrier, operations, and
codecs later consumed by T05's exact raw-carrier classifications; and
application-owned non-generic money values bound to the internal `money<C>`
template. Includes construction, operations, comparison, exact boundary
codecs through T03-W10's shared typed relation, calendar/leap/day-of-week
rules, wrap/carry, precision/range, no-generation policy, and money currency/
scale/rate/division/rounding/error
precedence. An unclassified signed 64-bit value remains an ordinary integer;
only T05-W01 or T05-W04 may classify one as a raw instant carrier through its
owned boundary or transition field.

Exit gate: every operation uses only explicit inputs; date/time range failures
take their frozen exception or result edge; instant precision/range and money
currency/scale/division/overflow failures produce their frozen error arms;
ambient clock/time-zone, random GUID, implicit rounding, premature raw-instant
classification, and unsupported codec or source forms reject during source/
contract validation.

Verification: complete value-boundary/differential tables, leap/calendar cases,
instant wrapper and internal-carrier difference extremes, rejection of a raw
carrier without its later owning sidecar, GUID ordering/codecs, and money
operation matrix.

### CSHARP-03-T03-W14 — Close semantic binding, specialization, and data emission

Depends on: T03-W13.

Current status: `Ready`.

Owns: closure of data-relevant type/method contract expression parsing and
attachment; strict semantic-binding parsing/attachment; logical declaration
identity versus source provenance; reachable binding closure; inferred closed
argument IDs; tag/payload/member/default/bound validation; derivation of the
exact closed root/provenance set from actual source and sidecars; invocation
of the T02-W02 specialization engine; deterministic emission of that engine's
validated concrete definitions before VIR; complete data-phase emission,
diagnostics, negative-case ownership, and completeness of the shared T03-W06
equality/ordering generator and T03-W10 typed-codec routing across every
applicable admitted concrete type. It does not implement a second closure,
identity, ordering, deduplication, limit, expansion, equality, or codec
algorithm.

Exit gate: stale/colliding/missing/duplicate/unreachable/cyclic bindings and
tampered foundation descriptors, instance tables, manifests, or expansion
reject. No source-defined generic, unsupported constructed CLR type, generic
call, or template node survives the VIR barrier; no monomorphic sequence-
construction state crosses an incompatible merge/publication boundary or
is live at a function or public-value boundary. Each admitted concrete type
has exactly its applicable shared equality, ordering, and codec route, and no
later type-specific implementation bypass exists. T06-W09 separately owns proof
that no such state is encoded into a certificate. Frozen built-in data-operation
exception values and normal/exceptional successors are independently
validated, while every source exception declaration or explicit handler form
remains outside T03 acceptance and emits no T03 artifact; T04 owns its final
validation and diagnostics. No MPK application dependency is introduced;
iterator and async families are covered only by the frozen rejection vectors.
Every source vector whose first applicable gate is T03 runs through the real
frontend and independently validated importer when accepted, or reaches its
frozen owning source, sidecar, lowering, emission, or import barrier without a
downstream artifact when rejected. Each internal typed handoff vector executes
production validation code and names the later first source/control/boundary
owner that must replace model-only evidence with end-to-end evidence. Loop-
dependent collection positives are not T03 acceptance rows: their routing is
asserted at T03, and their first actual-source acceptance owner is T04. Boundary
invocation is likewise T05-owned. The T03 review has zero findings.

Verification: C# subset/contracts/lowering/emission suites, equality/ordering/
codec routing completeness and bypass mutations, and all T03 fuzz seeds in two
isolated runs, `./scripts/build-csharp-frontend.sh --check`, and
`./scripts/check-fast.sh`. The native installed-release gate remains owned by
T07/T08.

## 9. CSHARP-03-T04 — Control frontend

### CSHARP-03-T04-W01 — Parse and attach loop contracts

Depends on: T03-W14.

Owns: loop invariant, optional decreases, modifies/ownership facts, normal and
abrupt exit claims, strict loop-to-sidecar attachment, expression typing, and
complete loop nesting/accounting. Collection-loop records additionally own
exact output-count, initialized-prefix, canonical-order, uniqueness,
duplicate-policy, and applicable insertion-order-independence clauses for the
T03-W08/W09 handoff operations.

Exit gate: every admitted loop has the required contract; missing, duplicate,
wrong-target, ill-typed, impure, or out-of-scope facts reject before lowering.

Verification: contract attachment/typing/precedence tests for each loop form
and nested-loop mutation cases.

### CSHARP-03-T04-W02 — Lower structured loops and abrupt edges

Depends on: T04-W01.

Owns: canonical CFG for admitted `for`, `while`, `do`, and exact compiler-
recognized `foreach` over arrays or strings; stable IDs, invariant entry/back-
edge/exit points, decreases, break/continue, nested targets, and partial-versus-
total metadata. It consumes the T03-W08/W09 data-operation handoffs and owns
the first source lowering for count-then-allocate count/fill pairs and
source-defined count, membership, lookup, traversal, aggregation, sort, dedup,
and duplicate reject/replace loops; it does not duplicate the T03 collection
representations or the T02 specialization engine. For `foreach`, it consumes
T03-W02's exact-type normalization and T03-W07's array ownership handoff,
accepts explicit-type and `var` variables over the exact array/string forms,
and rejects a write conflicting with the active read borrow.

Exit gate: source-order evaluation and every normal/abrupt edge are explicit;
`goto`, labels, unsafe jumps, unsupported or irreducible loop forms, and
ambiguous targets reject. A partial loop may omit decreases only when its
partial-termination status remains explicit; a total claim requires the
frozen decreases obligations. Every admitted two-pass result emits exact count
agreement, initialized-prefix, bound, and publication obligations; every
admitted map/set algorithm emits its frozen ordering, uniqueness, duplicate-
policy, and insertion-order-independence obligations.

Verification: CFG golden vectors, loop interpreter differential cases, stable
ID determinism, invariant/decreases boundary tests, count/fill disagreements,
explicit-type/`var` `foreach` equivalence and active-borrow mutation, and
positive/negative collection-loop source cases routed from T03-W08/W09.

### CSHARP-03-T04-W03 — Lower switch and admitted patterns

Depends on: T04-W02.

Owns: source-order arms/guards, exhaustiveness, null/type/property/list patterns,
constant/discard/`var`/relational/parenthesized/logical patterns, bindings and
scopes, decision nodes, and unmatched behavior over the exact admitted closed
type families.

Exit gate: pattern selection matches frozen Roslyn/runtime probes; an
expression switch is exhaustive or carries the exact modeled non-exhaustive
exception path; dynamic, recursive/open, identity-sensitive, side-effecting,
positional/deconstruction, slice, extension-based, or otherwise unsupported
patterns reject deterministically. Statement fall-through, `goto case`, and
`goto default` reject. A property getter must pass the structural purity and
immutability rules and carry the frozen totality claim; T06-W04 owns semantic
discharge for its use in the decision graph.

Verification: decision-graph goldens, guard ordering, exhaustiveness/overlap,
binding scope, runtime differential, and compiler-upgrade cases.

### CSHARP-03-T04-W04 — Admit closed exception declarations and explicit throw sites

Depends on: T04-W03.

Owns: extension of declaration validation for the exact source-exception base
clauses reserved by T03-W03; the closed exception sum; parameterless admitted
built-in construction; source-exception declarations sealed directly over
`System.Exception` and frozen payload constructors with the exact
parameterless `System.Exception` base call;
standalone `throw new ExactException(...)`; method exceptional result sets;
exceptional postcondition attachment; and uncaught classification. It consumes
the frozen built-in operation exception values and successors already emitted
by the T03 data-operation owners and does not define a second conversion for
them.

Exit gate: runtime exception objects/messages/stacks never enter artifacts;
each throw has one typed closed value and edge. Unknown, dynamic, wrapped, or
reflection-origin exception forms, message/inner/runtime-state constructors,
throw expressions, `throw null`, stored/reused exception objects, and
observation of message/stack/runtime identity reject. Resource-exhaustion and
process/runtime exceptions, including `OutOfMemoryException` and
`StackOverflowException`, are outside the closed sum and cannot be constructed,
declared, caught, or used in a filter. A closed exception outside the method's
declared `throws` set instead creates the exact catch-or-unreachable obligation
and must be caught or proved unreachable before verified acceptance.

Verification: source-exception base-clause, constructor, tag, payload, throw,
and uncaught vectors plus exceptional contract type tests.

### CSHARP-03-T04-W05 — Lower catch, filters, finally, and propagation

Depends on: T04-W04.

Owns: lexical inner-to-outer handler search, pure Boolean filter typing and
filter-before-finally evaluation, filter throws, ordered catch selection,
finally on every exit, override/
propagation behavior, the exact bare `throw;` rethrow within its active catch,
catch-variable access limited to the exact admitted immutable payload,
return/break/continue interaction, and explicit CFG regions/edges.

Exit gate: every source-design search/unwind case has one canonical trace;
hidden runtime dispatch, unspecified order, exception suppression outside the
frozen rule, an ill-typed or impure filter, rethrow outside the active catch,
and source control that exits a `finally` via return or outward break/continue/
goto reject.

Verification: trace differential corpus, region/edge goldens, filter/finally
type/purity and mutation matrix, rethrow cases, and abrupt-control combinations.

### CSHARP-03-T04-W06 — Close control emission and validation

Depends on: T04-W05.

Owns: independent VIR validation for loop/pattern/exception regions, complete
maps/manifests, control diagnostics/counters/fuzz seeds, and T04 cross-feature
cases, including abrupt discard and catch/finally non-observation of a
partially constructed object, initialized array, or sequence-construction
state; actual-source emission evidence for every loop-dependent collection
vector deferred by T03-W08/W09; revalidation of construction ownership and
publication over every newly loop-lowered normal and abrupt path; derivation
of the additional closed root/provenance set introduced by
accepted control/exception source and contract forms; and invocation of the
T02-W02 specialization engine over the complete data-plus-control/exception
root set.
It validates propagation of partial/total metadata through the reachable call
graph, rejects a partial callee on a total path, and does not implement a
second specialization algorithm.

Exit gate: malformed dominance, region, target, handler, ordering, source
mapping, or closed-instance linkage rejects independent of Roslyn; the
importer recomputes the complete data-plus-control/exception closed-instance
table; no construction-state value becomes visible on an exceptional edge or
handler/finally path, crosses an incompatible merge, or remains live at a
function or public-value boundary; every normal publication eliminates it to
the exact immutable value. Every deferred collection positive now passes
through the real frontend and importer, while every negative is artifact-free;
all T04 source cases are deterministic; review has zero findings.

Verification: control importer tests, closed-root/instance/linkage mutations,
partial-object/array/sequence exceptional-path mutations, bounded pattern/
exception/CFG fuzzing, two-pass and map/set loop matrices, incompatible-merge/
premature-publication/residual-state mutations after loop lowering, full C#
control frontend suite, and
`./scripts/check-fast.sh`.

## 10. CSHARP-03-T05 — Boundary and transition frontend

T05 implements an MPK verification overlay around the unchanged selected
application method. It does not define an application runtime protocol,
serializer, framework integration, or deployable MPK library.

### CSHARP-03-T05-W01 — Validate boundary sidecars and presence bindings

Depends on: T04-W06.

Owns: strict parsing and attachment of the frozen boundary schema/version,
semantic context, selected-method identity, ordered input/output fields, exact
admitted value types, document/value limits, and parse/format profile; plus
attachment of a T03-W12-validated application-owned non-generic presence
binding to each exact boundary field, with checked missing/null/value arms,
payload, default, invariant, required/optional status, and optional-missing
mapping. It is also the sole owner of
classifying an exact signed 64-bit boundary field as the raw internal instant
carrier; absent that explicit field classification, the value remains an
ordinary integer.

Exit gate: duplicate/unknown fields, unknown enum/tag values, missing required
fields, null for non-null targets, implicit missing/null collapse, invalid
defaults, inactive payloads, schema/method/context/profile mismatch, stale
source identity, arbitrary nesting, and over-limit shapes reject before
invocation. A sidecar identifies but cannot invent or trust application
members, checks, constructors, or invariants.

Verification: strict boundary-schema vectors; required/optional/non-null and
missing/null/value matrices; presence binding projection, default, inactive-
payload, identity/hash, nesting, and limit mutations in
`csharp_practical_boundary.rs`.

### CSHARP-03-T05-W02 — Capture and decode canonical overlay input

Depends on: T05-W01.

Owns: immutable capture of one canonical boundary document per verification or
reproduction run; UTF-8 parsing; duplicate/unknown/member-order-independent
input checks; exact scalar/tag/map-entry-array codecs; typed conversion;
original-byte/provenance identity; canonical byte and typed-value hashes; and
manifest/evidence linkage before the internal wrapper invokes the original
application method.

Exit gate: direct object injection or bypass of the canonical byte parser
rejects. An MPK-side adapter may translate other bytes only by retaining their
separate provenance and producing the canonical document; neither that adapter
nor the external company's unchanged production adapter becomes proved. No
reflection serializer, runtime type name, culture, host path, credential, or
serializer exception text enters an artifact or public diagnostic.

Verification: canonical/noncanonical UTF-8 and numeric/text/date/time/GUID
corpora, map-as-entry-array cases, lone-surrogate escapes, depth/count/byte
boundaries, adapter-provenance and bypass mutations, and two-run byte/value/hash
determinism.

### CSHARP-03-T05-W03 — Encode, reparse, and link canonical overlay output

Depends on: T05-W02.

Owns: deterministic encoding of the returned application-owned value, fixed
member/tag/token/escape order, canonical output byte/value hashes, independent
reparse, structural equality with the original returned value, source-map and
manifest linkage, and artifact-free failure ordering.

Exit gate: evidence is retained only when the emitted bytes reparse to exactly
the returned value under the frozen codecs and limits. Serializer/runtime
output is never proof authority; a byte, field order, escaping, token kind,
tag, numeric spelling, value, context, or hash mutation fails closed. The
certificate proves the typed relation, not serializer correctness or external
meaning preservation.

Verification: output canonical goldens and reparse round trips for every
admitted value family, hostile serializer mutations, hash/context splicing,
limit-minus-one/exact/plus-one cases, and failure-with-no-artifact assertions.

### CSHARP-03-T05-W04 — Attach and lower pure application transitions

Depends on: T05-W03.

Owns: the exact pure source shape
`Apply(State, Command, Context) -> ApplicationOwnedApplyResult`; mandatory
application-owned non-generic bindings to
`result<transition<State,Event,Response>,DomainError>`; exact state, command,
context, event, response, and error types; ordered bounded event arrays; state
invariant, result arms, accepted command cases, expected version, explicit
effective time, response relation, event relation, and frozen error precedence.
It owns the corresponding exact signed 64-bit raw-instant classification for a
transition field when that form is selected; no other transition integer gains
instant semantics.

Exit gate: successful new-command paths emit the exact new-state invariant,
version-increment, event/response, and bound obligations; business-error paths
emit the exact unchanged-input-state obligation. T06-W08 owns semantic
discharge. The signature and application build name no MPK type or component.
Persistence, transactionality, locks, delivery, retries, clock, identity
generation, authentication truth, and transport remain explicit external
assumptions and cannot enter the selected closure or certificate.

Verification: binding/signature/contract attachment mutations, success and
every business-error arm, invariant/version/event/response counterexamples,
explicit-time cases, effect-firewall rejection, and source-to-runtime finite
differential traces in `csharp_practical_transition.rs`.

### CSHARP-03-T05-W05 — Lower optional full-snapshot idempotency and precedence

Depends on: T05-W04.

Owns: the optional processed-command record containing the key, complete
application-owned `Command` and `Context` snapshots, and stored response; the
source-defined field-complete snapshot equality helper; attachment and lowering
of the claim that it is equivalent to equality of exact canonical field
encodings; reflexive-equality eligibility; retained-key replay/mismatch
behavior; new-key version and history capacity behavior; event suppression on
replay; and exact check precedence.

Exit gate: after ordinary boundary preconditions, an existing key is checked
first. Equal complete snapshots return unchanged state, no events, and the
stored response; different snapshots return the explicit idempotency-conflict
error. A new key then checks expected version and finally bounded history
capacity. No digest substitutes for source equality, no smaller caller
projection is selectable, and float/double or recursively non-reflexive
snapshot fields reject the claim. A project without the complete record may
verify the transition only with idempotency disabled. The emitted equality-
equivalence obligation covers every snapshot field but is not considered
discharged until T06-W08 proves it.

Verification: field-by-field equality/encoding obligation shape, omitted-field
and collision-assumption mutations, missing-versus-null snapshots, reflexivity
rejection, replay/mismatch/version/capacity precedence matrix, event ordering,
and the no-idempotency variant. Proof-positive and counterexample execution
belongs to T06-W08.

### CSHARP-03-T05-W06 — Close boundary and transition emission

Depends on: T05-W05.

Owns: independent importer validation for boundary/transition bindings and
linkage; total-termination propagation for the selected root and its reachable
call/loop closure; diagnostics, counters, source maps, manifests, fuzz seeds,
cross-feature cases; derivation of the additional closed root/provenance set
introduced by the accepted boundary and transition sidecars; invocation of the
T02-W02 specialization engine over the complete data, control/exception,
boundary, and transition root set; and the T05 rows and linked review
attachments in the CSHARP-03 ledger. It does not implement a second closure,
identity, ordering, deduplication, limit, or expansion algorithm.

Exit gate: every accepted boundary and transition vector enters through actual
captured C# plus overlay sidecars, survives independent generic-free
monomorphic VIR validation, has deterministic input/output linkage, and has a
complete derived closed-instance table recomputed from the accepted source and
all attached sidecars. Every rejected vector reaches its frozen owning source,
sidecar, boundary, transition, specialization, emission, or import barrier and
emits no downstream artifact. Any partial loop or callee on the public root,
application MPK dependency, serializer/effect bypass, malformed binding,
residual generic/template value, iterator, or async shape rejects; review has
zero findings.

Verification: complete boundary and transition suites twice, bounded sidecar/
adapter/protocol fuzzing, root/instance/linkage, totality, and effect-firewall
mutations, `./scripts/build-csharp-frontend.sh --check`, and
`./scripts/check-fast.sh`.
The native installed-release gate remains owned by T07/T08.

## 11. CSHARP-03-T06 — Verification integration

### CSHARP-03-T06-W01 — Import and validate the complete contract expression union

Depends on: T05-W06.

Owns: independent verification-side parsing/import, typing, canonicalization,
resource bounds, and attachment-identity recomputation for every canonical
type/method/boundary/transition expression artifact already emitted by the
T03-W14, T04-W06, and T05-W06 frontend gates, including structural
projections, sums, collections, codecs, control state, exception values,
semantic-binding identities, specialized closed foundation operations,
boundary linkage, and transition relations. Source-side sidecar parsing and
attachment remain with those earlier feature owners; T06-W01 does not
reimplement them.

Exit gate: every expression has one typed ordinary-term encoding; unknown,
impure, partial, ill-scoped, ambiguous, duplicate, or over-limit expressions
reject before VC generation.

Verification: exhaustive tag/typing/normalization vectors, frontend-versus-
importer attachment comparisons, and importer parser fuzzing.

### CSHARP-03-T06-W02 — Generate construction and type-invariant obligations

Depends on: T06-W01.

Owns: constructor/object-initializer member assignment, definite initialization,
construction invariant, public type invariant, required/init state, enum/
default eligibility, and invariant preservation obligations.

Exit gate: no value becomes publicly usable before construction obligations
hold; every public operation assumes and re-establishes the exact frozen type
invariant.

Verification: positive/negative VC goldens and deliberately broken constructor,
required-member, default, and preservation cases.

### CSHARP-03-T06-W03 — Generate data, collection, string, numeric, and business VCs

Depends on: T06-W02.

Owns: array/sequence/map/set bounds and ownership; structural equality/order;
string/codecs; float-bit/decimal definitions; nullable/outcome payloads;
calendar/time/GUID/money checks; and ordered error obligations.

Exit gate: every emitted check is either structurally discharged or becomes an
ordinary bounded obligation; no runtime/library fact is trusted or axiomatized.

Verification: one success and one failing proof per semantic row, obligation
ordering tests, and importer/front-end cross-checks.

### CSHARP-03-T06-W04 — Generate loop, switch, and pattern obligations

Depends on: T06-W03.

Owns: invariant initialization/preservation/exit, decreases and partial/total
claims, break/continue facts, arm/guard order, exhaustiveness, bindings, and
pattern decision equivalence.

Exit gate: all loop exits and pattern paths are covered exactly once; total
correctness is claimed only with a valid decreases proof.

Verification: proof-positive and counterexample fixtures for each loop/pattern
path and stable obligation IDs.

### CSHARP-03-T06-W05 — Generate exceptional-control obligations

Depends on: T06-W04.

Owns: throw/callee exceptional sets, catch/filter/finally search and unwind,
normal/exceptional postconditions, filter failure, finally override, and
uncaught-result obligations.

Exit gate: each normal and exceptional edge has the correct pre/post state and
handler claim; exception runtime identity/messages are absent.

Verification: all canonical exception traces, broken exceptional postconditions,
and malformed handler-region inputs.

### CSHARP-03-T06-W06 — Generate binding and foundation-specialization obligations

Depends on: T06-W05.

Owns: application binding shape, source identity/provenance, tag/payload/member
agreement, source invariant, total projection, exactly-one-arm and arm-
distinctness, reconstruction, observation-preserving round trips, operation
commutation, actual default mapping, and bound obligations; plus registered
foundation descriptor/hash linkage, reachable closed-instance dependency
closure, canonical identity/order/deduplication, expansion counters, and
concrete-definition equivalence.

Exit gate: each application representation is proved equivalent to its exact
closed semantic role rather than trusted from a sidecar; the importer and VC
agree on the complete derived instance table and ordinary expansion. Missing,
stale, colliding, cyclic, unused, caller-injected, over-limit, or tampered
bindings/instances fail, and no generic/template value or theorem reaches the
VC or certificate.

Verification: positive and deliberately broken projection/reconstruction/
commutation proofs for every binding role; descriptor/hash/closure/identity/
order/limit mutations; and residual-generic ordinary-term rejection.

### CSHARP-03-T06-W07 — Generate canonical boundary round-trip obligations

Depends on: T06-W06.

Owns: ordinary-term obligations and independent cross-checks for T05's strict
canonical JSON parse, duplicate/unknown/required/non-null and missing/null/value
rules, numeric/text canonicality, depth/count/byte limits, typed conversion,
raw-input-to-value hash linkage, canonical output, and output-reparse equality
in the MPK verification overlay.

Exit gate: serializer/runtime output is never proof authority. The manifest and
evidence chain bind exact raw input and output bytes to the typed canonical and
reparsed values; the certificate proves only the ordinary relation over those
typed values and does not assert serializer correctness. All mutations break
linkage or fail parsing.

Verification: boundary parser corpus/fuzzing, three-state field matrix, limit
boundaries, hostile serializer mutations, and round-trip proof cases.

### CSHARP-03-T06-W08 — Generate pure-transition and idempotency obligations

Depends on: T06-W07.

Owns: ordinary-term obligations and counterexamples for T05's input/output
state invariants, expected/current version, success/error arms, ordered bounded
events, response relation, explicit time, optimistic conflict, optional
idempotency replay using complete retained application-owned `Command` and
`Context` snapshots, field-complete source equality equivalent to canonical
field encodings, history capacity, and frozen error precedence.

Exit gate: the verified function is pure; persistence, locking, transport,
clock, identity generation, and infrastructure idempotency remain explicitly
outside the certificate. A digest is evidence linkage only and never replaces
source equality; non-reflexive snapshot types reject the idempotency claim.

Verification: transition matrix covering accept/error/replay/snapshot mismatch/
version conflict/capacity and broken invariant/event/response/equality cases.

### CSHARP-03-T06-W09 — Encode ordinary foundations and close zero-axiom checking

Depends on: T06-W08.

Owns: translation of every already-expanded concrete foundation definition in
monomorphic VIR into ordinary core definitions and proof terms for all new
finite values and operations; the successor program-assembly profile;
certificate assembly, limits, same-byte dual-checker invocation; and explicit
enforcement of empty proof-node/theory-certificate tables and total axiom count
zero.

Exit gate: both checkers accept identical canonical certificate bytes and
reject every mutated proof/context/foundation/hash; neither checker sees a
template/generic construct or gains a C#-specific rule. Certificate v0 and both
acceptance rules are byte/behavior unchanged.

Verification: checker-agreement script, certificate mutation suite, axiom
inventory comparison, and full predecessor certificate corpus.

### CSHARP-03-T06-W10 — Integrate policy, evidence, and reproduction

Depends on: T06-W09.

Owns: practical-profile policy scan, evidence schemas, source/profile/context/
foundation/binding/closed-instance/boundary/transition/checker linkage,
provider redaction, registered-bundle-only reproduction semantics, and a
complete source-to-candidate recipe. Its sole bundle/registry input is a
private test fixture under
`develop/migrations/csharp-03/t06-policy-bundle/`; it is not a release bundle
or installed registry entry, and T07-W02 alone owns their final candidate
forms.

Exit gate: evidence can be reproduced without compiler/runtime trust, raw
paths, credentials, network, or unregistered binaries; any linkage mutation
fails closed. The foundation content hash is independently recomputed from the
registered bytes and agrees across context, manifest, evidence, and recipe.
Production logic accepts only a bundle present in its supplied validated
registry, while the T06 fixture and its identity remain absent from
`release/bundles/` and the active installed registry.

Verification: `csharp_practical_policy_verify.rs`, evidence schema/hash cases,
redaction tests, rejection of an unregistered fixture mutation, and offline
reproduction from the exact private fixture bundle and fixture registry.

### CSHARP-03-T06-W11 — Integrate AI explanation and structured API

Depends on: T06-W10.

Owns: practical-profile AI explanation and API request/response variants,
context propagation, boundary/transition summaries, bounded diagnostics,
provider isolation/redaction, and exact unsupported-version behavior.

Exit gate: AI output is non-authoritative and derived only from verified
artifacts; API callers cannot inject paths, toolchains, artifacts, compiler
output, a foundation bundle/instance allowlist, or a different profile/context.

Verification: AI/API schema, redaction, route, version, batch, and cross-profile
mutation tests; public production route remains inactive.

### CSHARP-03-T06-W12 — Close end-to-end private verification

Depends on: T06-W11.

Owns: actual-source frontend-to-certificate private tests across every
capability, boundary and transition linkage, both checkers, all consumer
inventories, application-build dependency inspection, total-root closure,
deterministic two-run evidence, and the T06 rows and linked review attachments
in the CSHARP-03 ledger.

Exit gate: no helper-constructed IR is used as sole evidence, all frozen rows
have one passing and one failing proof where meaningful, predecessor
certificates remain identical in verdict/axioms, application outputs contain
no MPK reference, every public root is total, every artifact is monomorphic,
and review has zero findings.

Verification: all mpk-vc/mpk-cli/mpk-api practical suites, checker agreement,
predecessor suites, `./scripts/check-fast.sh`.

## 12. CSHARP-03-T07 — Reproducible release candidate

### CSHARP-03-T07-W01 — Finalize deterministic offline build inputs

Depends on: T06-W12.

Owns: promotion of the frozen T01 private toolchain measurements into the final
candidate inputs under `release/build-inputs/csharp/`, the release build
descriptor, source closure, final frontend inventory, runtime files, notices,
modes, build recipe, deterministic archive, and hostile ambient build checks.
Toolchain facts must remain byte-for-byte traceable to T01;
implementation-owned source inventory is recomputed from the reviewed T06 tree
rather than copied from the T01 probe harness. No foundation descriptor,
template, expanded definition, or semantic operation is placed in the .NET
reference closure, application build output, or
`release/build-inputs/csharp/`; foundation bundle packaging belongs to
T07-W02.

Exit gate: two fresh offline builds produce byte-identical reviewed candidate
trees/archives; any archive/file/mode/flag/reference or declared build-
environment mutation fails, while every irrelevant ambient perturbation owned
by T07-W04 is ignored and leaves the candidate bytes unchanged.

Verification: C# build-input tests and the exact two-clean-build recipe.

### CSHARP-03-T07-W02 — Assemble immutable toolchain, frontend, and foundation bundles

Depends on: T07-W01.

Owns: candidate toolchain, frontend, and verification-foundation bundle
descriptors, content hashes, member inventories, launcher contract,
semantic-context/profile/foundation linkage, native dependencies, the exact
closed foundation descriptor and bytes selected by the registry, and the
private release-registry tuple. The foundation remains an MPK verification
input and is never a .NET application reference or runtime component. No
caller-provided foundation bundle or instance allowlist is accepted.

Exit gate: only exact registered members launch; replacement, omission, mode,
path traversal, symlink, extra file, wrong context/profile, and hash mutation
reject. The tuple is not public/installed.

Verification: release-bundle registry and mutation tests plus two materialized
candidate comparisons.

### CSHARP-03-T07-W03 — Implement the registered native sandbox runner

Depends on: T07-W02.

Owns: exact launcher, namespaces, read-only/materialized files, syscall policy,
cgroup v2 CPU/memory/process limits, filesystem/network denial, environment
closure, timeout, process cleanup, and bounded stdout/stderr/protocol handling.

Exit gate: only the private registered bundle runs; escape, network, fork,
resource, output, timeout, orphan, and cleanup probes fail with frozen public
diagnostics.

Verification: runner unit/integration tests and native root x86-64 Linux
sandbox gate.

### CSHARP-03-T07-W04 — Prove hostile-environment and resource determinism

Depends on: T07-W03.

Owns: tests for locale/time-zone/user/home/path/tmp/cache/credential/proxy/network
perturbation, ignored ambient SDK/runtime/package caches, filesystem layout,
resource boundary minus-one/exact/plus-one, and repeated cleanup.

Exit gate: accepted output bytes and diagnostics are invariant under all
irrelevant perturbations; relevant over-limit cases fail before artifact
publication and leave no process/file/cgroup residue.

Verification: hostile-environment runner matrix twice on native Linux.

### CSHARP-03-T07-W05 — Rehearse the complete private installed image

Depends on: T07-W04.

Owns: private image assembly, every active predecessor tuple plus a practical
tuple installed and executable only inside that private candidate image (and
absent from the active production image), all on the sole successor registry;
two builds/two runs; image mutation checks; predecessor/practical corpus;
policy/evidence/API/checker paths; ordinary practical-source builds and output
dependency inspection proving no MPK reference; and rollback image
materialization. Implements the exact local practical candidate/release-gate
command and owner frozen by T01-W10; later release tasks invoke this command
rather than inventing another aggregate gate.

Exit gate: the candidate image is byte-identical across builds, all runs are
deterministic, no old/staging compatibility route exists, and rollback restores
the exact T01-W01 baseline image.

Verification: local native Linux candidate release gate twice and complete
predecessor equivalence reports.

### CSHARP-03-T07-W06 — Publish the private candidate receipt and close T07

Depends on: T07-W05.

Owns: immutable receipt hashes for toolchain/frontend/foundation/registry/
artifacts/checkers/image, build and native-gate evidence, limits, corpus,
axiom inventory, rollback target, known exclusions, and zero-finding review.

Exit gate: every receipt field is independently recomputable; no identity or
hash points to an unregistered/ambient item; the receipt binds the independently
recomputed foundation descriptor/content hash to the context, closed-instance
manifest, and candidate image; T07 review has zero findings and the active
release remains unchanged.

Verification: receipt schema/recomputation tests, `./scripts/check-fast.sh`,
and a second native candidate run from the recorded inputs through the exact
T01-W10-frozen command.

## 13. CSHARP-03-T08 — Complete rehearsal and atomic activation

T08-W01 through T08-W09 remain private rehearsal work. They may check in
candidate evidence and examples only after each item satisfies its own
installed-frontend, boundary, and checker gates, but they must not modify the
tracked `fixtures/csharp/policy/` tree or expose an active/public practical
profile route. Throughout these items, “installed” means the exact T07 private
candidate image and its candidate-only practical tuple; the active production
image and registry remain unchanged. Examples checked in by T08-W02 through
T08-W04 remain uninstalled and unadvertised until T08-W10. T08-W10 is the sole
fixture replacement, installed-example routing, and activation owner.

### CSHARP-03-T08-W01 — Stage the C# policy-fixture replacement from actual source

Depends on: T07-W06.

Owns: a private candidate under
`develop/migrations/csharp-03/fixture-candidate/` containing the corrected
`Required.cs` and the selection, context, maps, manifests, VIR, VC/skeleton,
scan, evidence, certificate, hashes, AI fixtures, and manifest ownership
regenerated through the real private installed frontend. The tracked
`fixtures/csharp/policy/` tree remains byte-identical to the T01-W01 baseline.

Exit gate: candidate source syntax and artifacts agree; no manually attached
VIR or stale byte/hash remains; boundary bytes round-trip where applicable;
both checkers accept the exact recorded certificate bytes; a complete
candidate inventory proves that T08-W10 can replace the linked tracked files
atomically.

Verification: private-candidate reproduction from scratch, policy/evidence/AI
tests, checker agreement, a mutation proving the old mismatch rejects, and a
repository diff assertion that the tracked fixture path is unchanged.

### CSHARP-03-T08-W02 — Add the invoice pricing and tax example

Depends on: T08-W01.

Owns: runnable candidate-verified source example with immutable request/result,
application-owned non-generic money and outcome types, currency/scale,
business/effective dates, decimal rounding, ordered line aggregation,
count-then-allocate bounded arrays, construction styles, contracts, boundary
input/output, separate overlay artifacts, tests, and trust-boundary README.

Exit gate: positive and business-error cases run through `mpk policy scan` and
`mpk policy verify`, round-trip canonical JSON, and both checkers; README
states exactly what is and is not proved. The example compiles unchanged as an
ordinary application and its output contains no MPK dependency.

Verification: example-local runtime assertions plus installed frontend,
policy/evidence reproduction, and same-byte checker tests. Runtime assertions
use the pinned direct-compiler harness; no project or NuGet input is admitted
to the frontend.

### CSHARP-03-T08-W03 — Add the order transition example

Depends on: T08-W02.

Owns: runnable example with GUID command/idempotency keys, an application-owned
instant, expected version, switch/pattern state logic, application-owned closed
result/transition types, one allowlisted caught exception, replay-safe response,
ordered bounded event arrays, transition contract, separate overlay artifacts,
tests, and trust-boundary README.

Exit gate: accept/error/replay/full-snapshot-mismatch/version-conflict/capacity
cases prove the frozen relations without claiming persistence, locking, clock,
identity generation, or transport correctness. The source equality helper is
field-complete and proved equivalent to canonical field encodings; no encoding
digest is treated as equality.

Verification: transition matrix through installed `mpk policy scan` and
`mpk policy verify`, boundary round-trip, evidence reproduction, and both
checkers.

### CSHARP-03-T08-W04 — Add the batch validation example

Depends on: T08-W03.

Owns: runnable example with canonical boundary JSON, missing/null/value, exact
codecs, ordered map/set duplicate handling, accumulating closed validation,
application-owned entry/outcome types, synchronous array-based loops, artifacts,
tests, and trust-boundary README.

Exit gate: duplicate/unknown/noncanonical/limit and validation-order cases are
demonstrated; the application has no source/runtime MPK dependency; no example
requires iterator or async support; both checkers accept the exact
certificates.

Verification: installed end-to-end positive/negative `mpk policy scan` and
`mpk policy verify` runs, boundary reparse, duplicate/error accumulation order,
array/map/set bounds, evidence reproduction, and checker agreement. Separate
aggregate conformance tests retain iterator/async rejection coverage.

### CSHARP-03-T08-W05 — Close cross-example capability coverage

Depends on: T08-W04.

Owns: a machine-tested matrix proving the three examples collectively exercise
constructor-only and required/init/object-initializer construction, arrays,
two-pass sequence construction, application-owned entries/maps/sets,
strings/codecs, loop invariant/decreases, structural equality/order,
nullable/outcomes, exception, semantic bindings, foundation specialization,
boundary/transition, and every business primitive. Iterator and async remain
exclusion-matrix rows and are not example capabilities.

Exit gate: every admitted-capability row points to an actual source span,
emitted node, obligation, certificate theorem, runtime case, and README trust
statement; no helper-only artifact satisfies it. Each exclusion row instead
points to a selected-source rejection fixture, frozen diagnostic, and
assertion that no downstream artifact was emitted.

Verification: fail the coverage test by deleting each kind of link; reproduce
all three examples twice from clean materializations.

### CSHARP-03-T08-W06 — Run complete conformance, fuzz, mutation, and upgrade gates

Depends on: T08-W05.

Owns: the aggregate practical corpus and implementation executors for every
positive/rejection/boundary/precedence/differential/determinism row; bounded
fuzzing of source/contract/Roslyn/pattern/exception/collection/codec/calendar/
boundary/transition/source-binding/specialization/artifact protocols; iterator,
async, task, and enumeration rejection seeds; and compiler/runtime/schema/
context/foundation/hash mutations.

Exit gate: all seeds/counters/time budgets are recorded and reproducible; every
frozen row executes in production code; two complete runs are identical; no
open crash, timeout, nondeterminism, or unowned vector remains.

Verification: the T01-W10-frozen practical candidate/release-gate command
twice, plus all recorded fuzz and mutation commands and
`./scripts/check-fast.sh`.

### CSHARP-03-T08-W07 — Prove complete predecessor and checker preservation

Depends on: T08-W06.

Owns: full Go/Rust/scalar-C#/Java source, VIR, VC, policy/evidence/API/release
corpora under the final candidate; semantic-difference reports; Certificate v0
byte/acceptance audit; same-byte dual-checker agreement; proof/theory table and
axiom inventory audit.

Exit gate: all predecessor behavior/obligations/verdicts and prior axiom
inventories/categories are equivalent; every practical-profile certificate has
empty proof/theory tables and total axiom count zero; neither checker contains
a profile-specific acceptance bypass.

Verification: complete predecessor local gates, checker agreement, axiom audit,
and repository search/mutation tests for bypasses and obsolete formats.

### CSHARP-03-T08-W08 — Finalize release docs, exclusions, activation, and rollback plan

Depends on: T08-W07.

Owns: README/developer/spec routing, exact profile capability/exclusion text,
zero-application-dependency and verification-overlay guidance, registered
foundation/specialization rules, upgrade policy, operations/security/trust
boundary, installed release registry and bundle change set, atomic ordering,
failure points, no-dual-route searches, and executable whole-image rollback
procedure.

Exit gate: documentation never calls the profile full C# support; every final
identity/hash/limit is exact; activation is one atomic installed-image change;
rollback needs no artifact reinterpretation and returns to the T01-W01
pre-CSHARP-03 installed-image baseline retained in the T07 receipt.

Verification: docs/link/spec checks, release descriptor dry run, failure
injection at each activation step, rollback drill, and obsolete-route search.

### CSHARP-03-T08-W09 — Run the final local release gates and zero-finding review

Depends on: T08-W08.

Owns: `./scripts/check-fast.sh`, two runs of the exact T01-W10-frozen practical
candidate/release-gate command covering the complete reviewed native root
x86-64 Linux assembly/image-mutation/syscall/cgroup/resource/cleanup gate,
installed example reproduction, release receipt draft, and whole-diff
review/fix cycles.

Exit gate: every command passes from clean inputs on the exact candidate; both
native passes have identical hashes/verdicts; all reviewer findings are fixed
and the CSHARP-03 ledger plus every linked review attachment have no open
finding. No activation has occurred yet.

Verification: the commands and hashes recorded in the candidate receipt are
rerun by the final reviewer; any mismatch returns to the owning earlier item.

### CSHARP-03-T08-W10 — Atomically activate and record CSHARP-03 completion

Depends on: T08-W09 and transitively every earlier work item.

Owns: the one release commit that installs the frozen successor registry and
bundles for all active profiles, removes executable private/staging and old
format routes, atomically replaces the complete tracked C# policy fixture from
the exact T08-W01 candidate inventory, activates the practical C# tuple,
installs and advertises the three already verified examples, updates status/
routing documents, and finalizes the release receipt.

Exit gate: one installed successor release serves Go, Rust, scalar C#, Java,
and practical C#; no compatibility selector or alternate registry is
executable; actual-source examples pass; receipt records all identities,
hashes including the independently recomputed foundation hash and complete
closed-instance manifests, MPK-free application build inspection, native
evidence, checker agreement, empty proof/theory tables, total axiom count zero
for the practical-profile certificates, unchanged predecessor axiom
inventories/categories, rollback target, and zero findings; `DART-04` is
unblocked.

Verification: clean-image native release gate after activation, complete local
corpus and checker agreement through the exact T01-W10-frozen command,
installed-route and obsolete-format searches, an exact tracked-fixture-versus-
T08-W01-candidate inventory/hash comparison, rollback then re-activation
rehearsal, and `./scripts/check-fast.sh`.

## 14. Requirement-to-work-item traceability

This table is a completeness index, not a second owner. Detailed ownership is
the task contract above and the frozen T01 ledger.

| Source design area | Freeze owner | Implementation/verification owner |
| --- | --- | --- |
| authority, trust, atomic migration | T01-W01/W02/W09 | T02-W08/W09, T07-W05/W06, T08-W08-W10 |
| semantic registry/context/parameters/selection | T01-W02/W09 | T02-W01/W04/W08/W09, T08-W10 |
| source dependency, declaration/call/type closure | T01-W03/W04/W06 | T03-W01/W14, T06-W12, T07-W05, T08-W02-W04 |
| canonical source identity and provenance | T01-W02/W08/W09 | T02-W04/W05, T03-W14, T05-W01, T06-W06 |
| user-generic prohibition, exact `T?`, incidental metadata | T01-W04/W06/W08 | T02-W02/W05, T03-W01/W12/W14, T06-W06/W09 |
| registered foundation descriptor and closed specialization | T01-W02/W08/W09 | T02-W01-W06, T03-W14, T04-W06, T05-W06, T06-W06/W09/W10, T07-W02/W06 |
| application semantic bindings and projection obligations | T01-W08/W09 | T02-W03-W05, T03-W08/W09/W12-W14, T05-W01/W04, T06-W01/W06 |
| expression bodies, `var`, imports, nullable directive | T01-W04 | T03-W02, T04-W02 for `foreach var` |
| enum/struct/ordinary immutable class/default eligibility | T01-W04/W08 | T03-W03, T06-W02 |
| fields/properties/constructors/instance methods/init/required | T01-W04/W08 | T03-W04/W05, T04-W06, T06-W02 |
| structural equality and ordering | T01-W04/W08 | T03-W06/W14, T06-W03 |
| arrays and internal sequence construction | T01-W04/W08 | T02-W02/W03, T03-W07/W08, T04-W01/W02/W06, T06-W03/W04 |
| ordered maps and sets | T01-W04/W08 | T03-W09, T04-W01/W02/W06, T06-W03/W04 |
| UTF-16 strings and codecs | T01-W04/W07 | T03-W10/W14, T05-W02/W03, T06-W03/W07 |
| float/double and decimal | T01-W04/W07 | T03-W11, T06-W03/W09 |
| nullable and application-owned closed outcomes | T01-W04/W08 | T03-W12, T06-W03/W06 |
| date/time/duration/instant/GUID/money | T01-W04/W08 | T03-W13, T05-W01/W04/W06, T06-W03/W06 |
| loops and contracts | T01-W05/W09 | T04-W01/W02, T06-W04 |
| switch and patterns | T01-W05 | T04-W03, T06-W04 |
| exceptions, exact source-exception bases, and abrupt completion | T01-W05/W09 | T04-W04-W06, T06-W05 |
| iterator/async/task/enumeration exclusions | T01-W06/W09 | T03-W01/W14, T05-W06, T08-W06 |
| boundary JSON, overlay linkage, three-state presence | T01-W08/W09 | T05-W01-W03/W06, T06-W07 |
| pure transitions and optional full-snapshot idempotency | T01-W08/W09 | T05-W04-W06, T06-W08 |
| contract expression union | T01-W09 | T03-W04-W14, T04-W01/W04/W06, T05-W01/W04/W06, T06-W01 |
| closed framework surface and effect firewall | T01-W03/W06/W08/W09 | T03-W01/W10/W13/W14, T05-W02/W04/W06, T06-W10/W11, T07-W03/W04 |
| successor VIR/source artifacts/maps/manifests | T01-W02/W09 | T02-W02-W07, T03-W14, T04-W06, T05-W06 |
| VC/hash/certificate/zero axioms | T01-W09 | T02-W06, T06-W02-W09 |
| diagnostics, precedence, and limits | T01-W09 | each frontend task; gates T03-W14/T04-W06/T05-W06 |
| policy/evidence/AI/API | T01-W02/W09 | T02-W09, T06-W10-W12 |
| build, bundle, sandbox, release | T01-W03/W09 | T02-W09, T07-W01-W06, T08-W08-W10 |
| Required.cs repair and examples | T01-W10 vector ownership | T08-W01-W05, T08-W10 |
| aggregate conformance/fuzz/upgrade | T01-W04-W10 | T08-W06/W07/W09 |

## 15. Per-item handoff checklist

Before marking any work item complete, its commit and ledger row must answer
all of these without an implicit “as above”. An answer may be an exact value or
an explicit `not_applicable` that cites the first applicable owner/gate from
section 4; omission is never valid, and `not_applicable` is invalid once that
owner/gate has been reached:

- Which exact predecessor receipt and frozen spec/vector hashes were inputs?
- Which production files, tests, vectors, fixtures, and docs changed?
- Which source-design rows and diagnostic/limit counters does the item own?
- What accepted, rejected, boundary, precedence, mutation, determinism, and
  upgrade cases were added, and which production test executes each?
- What canonical artifacts and hashes changed, and why are predecessor bytes
  or semantics unchanged where required?
- How was the application's unchanged MPK-free build/output inspected, and how
  were user generics, unsupported constructed types, iterator, and async
  exclusions exercised?
- Which registered foundation descriptor/hash and derived closed-instance set
  were used, and where were closure, identity, ordering, limits, concrete
  expansion, and absence of residual generics independently recomputed?
- Which application semantic bindings changed, and where are total projection,
  arm distinction, round-trip, invariant, default, and operation-commutation
  obligations discharged?
- Which targeted commands and `./scripts/check-fast.sh` ran, on what host, and
  with what result? If native Linux evidence is required, where is its receipt?
- How was absence of an active/public/staging/ambient route checked?
- What review findings were found, how were they fixed, and where is the final
  zero-finding result?

If any answer is missing, the work item remains `In progress`.
