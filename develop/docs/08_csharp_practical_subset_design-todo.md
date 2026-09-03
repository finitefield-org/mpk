# CSHARP-03 Practical C# Implementation Milestones and Tasks

Status: current reviewed implementation decomposition, prepared on 2026-09-03.
CSHARP-03 has not started and every work item remains blocked pending the
native `JAVA-03-T10` x86-64 Linux release receipt.

Source design: `08_csharp_practical_subset_design.md`.

This document is subordinate to the source design and the future T01 frozen
specifications. It replaces the superseded source-visible-library,
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

`JAVA-03-T10` implemented the atomic Java activation on 2026-09-03, but the
ARM64 development host cannot produce the required native x86-64 Linux release
evidence. This prerequisite therefore remains open. This file remains planning
material. No CSHARP-03
specification, vector, identity, hash, build input, production code, fixture,
candidate bundle, or public route has been frozen or changed by the Java
release task.

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

Names in the source design such as `mpk.csharp.practical.v1`, “successor VIR”,
and the practical sidecar families are working names until T01 completes. T01
must publish one machine-readable name-and-owner inventory containing the
final profile, registry, schema, artifact, hash-domain, bundle, diagnostic,
and vector identities. Later tasks consume that inventory verbatim and do not
mint aliases.

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
| T03 | actual captured C# data-source runs, MPK-free application build/output inspection, source maps/manifests, generic rejection, specialization integration, and monomorphic data emission |
| T04 | source control, pattern, explicit throw/handler, and exceptional-region guarantees |
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
- the stage review ledger contains zero open findings.

The final stage additionally requires actual installed-source examples,
same-byte acceptance by both checkers, the complete native x86-64 Linux gate,
atomic activation, and a tested whole-image rollback procedure.

## 5. Milestone map

| Stage | Work items | Deliverable | Gate |
| --- | ---: | --- | --- |
| `CSHARP-03-T01` | 10 | measured feasibility record, frozen practical profile package, exact vectors and ledger | no unresolved or guessed fact |
| `CSHARP-03-T02` | 9 | private successor shared-artifact foundation and equivalent predecessor migration | no public route; all predecessor equivalence cases pass |
| `CSHARP-03-T03` | 14 | complete data-oriented practical frontend | every data capability lowers from actual C# source |
| `CSHARP-03-T04` | 6 | loops, switch/pattern, and explicit exceptional CFG | normal and abrupt control are independently validated |
| `CSHARP-03-T05` | 6 | canonical boundary and pure transition frontend | overlay linkage, round trip, totality, version, and optional idempotency pass |
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
one-byte, file-count, mode, flag, reference, and environment mutation fails
before publication; the candidate remains unregistered.

Verification: add `crates/mpk-cli/tests/csharp_practical_build_inputs.rs`,
extend the build-input script tests, run
`cargo test -p mpk-cli --test csharp_practical_build_inputs` against the two
private paths, run the isolated two-build recipe, and run
`./scripts/build-csharp-frontend.sh --check-build-inputs` to prove the active
scalar inputs remain unchanged. The native `./scripts/check-java-frontend.sh`
gate is reserved for T07/T08.

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
registered foundation descriptor, member inventory, content-hash domain,
operation sets, expansion definitions, counters, and exact internal template
registry: `bounded_sequence<T>`, `sequence_construction<T>`,
`ordered_entry<K,V>`, `ordered_map<K,V>`, `ordered_set<T>`, `option<T>`,
`lookup<T>`, `result<T,E>`, `validation<T,E>`, `boundary_field<T>`,
`transition<S,E,R>`, and `money<C>`. Register `unit`, `parse_error`, the
internal instant, and the closed exception sum as non-template definitions.
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

### CSHARP-03-T01-W09 — Freeze contracts, boundary, transition, identities, and limits

Depends on: T01-W08.

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

### CSHARP-03-T01-W10 — Publish and review the complete freeze package

Depends on: T01-W09.

Owns: the practical-profile specification package, successor shared-artifact
specifications, exact vectors and manifest entries, canonical probe records,
name-and-owner inventory, traceability ledger, upgrade matrix, and routing
updates in the design/todo/developer documentation; freezes the exact primary
test owner pairs and the exact local practical candidate/release-gate command
name and owner used by T07-W05/W06 and T08-W06/W09/W10.

Exit gate: every freeze-only requirement maps to its exact T01 W item and one
probe/specification-test owner; every implementation or release requirement,
vector row, diagnostic, limit, implementation surface, and release criterion
maps to one primary downstream implementation W item, any separately named
verification/activation owners, and one primary production-test owner pair.
All hashes are recomputed; a complete specification review produces zero
findings. Production behavior is unchanged.

Verification: `scripts/check-spec-vectors.py`, manifest tests,
`./scripts/check-fast.sh`, and a documented second-pass review after all fixes.

## 7. CSHARP-03-T02 — Shared artifact foundation

T02 is private and test-injected. It may create successor models and candidate
adapters, but no public CLI/API route, installed tuple, compatibility flag,
dual-registry lookup, or ambient staging-root discovery.

### CSHARP-03-T02-W01 — Implement the closed successor registry and context

Depends on: T01-W10.

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
sets; deriving those roots from actual C# source and sidecars belongs
exclusively to T03-W14.

Verification: descriptor/hash/member mutations; every template arity,
dependency, operation, and expansion; closure/order/dedup/depth/count limits;
unreachable or caller-injected instances; residual-generic rejection; and
canonical concrete-value round trips in `csharp_practical_vir_model.rs`.

### CSHARP-03-T02-W03 — Implement operation and explicit-control vocabulary

Depends on: T02-W02.

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

Owns: the successor skeleton/VC schemas, canonical ordering, context and VIR
linkage, ordinary-term type/value encoding, obligation groups, limits, and
hashes; the distinct successor program-assembly profile; and structural
enforcement of its foundation/context linkage while leaving Certificate v0 and
checker acceptance unchanged.

Exit gate: all new values and control forms have a closed encoding path. The
practical-profile assembly structurally enforces empty proof/theory tables and
zero axioms; predecessor assemblies retain their frozen table and axiom rules;
no unproved intrinsic is accepted.

Verification: successor VC/hash vectors, canonical byte mutations, and direct
rejection of nonempty proof-node/theory-certificate tables.

### CSHARP-03-T02-W07 — Implement successor frontend protocol, maps, and manifests

Depends on: T02-W06.

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

## 8. CSHARP-03-T03 — Data frontend

Every T03 accepted case starts as immutable captured `.cs` and exact JSON
sidecars, passes the pinned Roslyn public APIs, and is independently validated
after emission. A helper-constructed VIR is not acceptance evidence. T03 may
emit the frozen closed exceptional successor of a data operation using the
T02 vocabulary, but it does not admit source-declared exceptions, explicit
`throw`, `catch`, filters, or `finally`; those source forms and their region
semantics belong to T04. Each T03 feature item extends source-side type/method
sidecar parsing and attachment for its owned expression forms; T03-W14 closes
that data-sidecar union before T04 begins.

### CSHARP-03-T03-W01 — Extend capture, declaration accounting, and closure

Depends on: T02-W09.

Owns: selected source roots, immutable byte capture, encoding/path checks, all-
declaration accounting for selected files, reachable method/type closure,
finite call/type graphs, recursion rejection, and closed framework symbol/API
admission; rejects MPK package/assembly/namespace/attribute/interface/base/
generated-source dependencies, every user-defined generic declaration or
method and every closed use, and every arbitrary constructed CLR/BCL type
other than the exact value-type `T?` form; every source-written attribute also
rejects, while compiler-synthesized init/required markers are validated as
metadata observations and never become callable source APIs.

Exit gate: dead declarations in selected files are classified, every reachable
declaration is captured, unselected/ambient/generated/project/package input
rejects, selected application output remains MPK-independent, source-visible
transitive generic metadata rejects, and closure limits are enforced before
lowering.

Verification: extend C# capture/frontend vectors with multi-file, dead-tree,
cycle, source-root, path, encoding, every MPK dependency form, user-generic and
constructed-type categories, incidental generic metadata, exact `T?`, ambient-
reference, and limit cases.

### CSHARP-03-T03-W02 — Normalize concise syntax and name resolution

Depends on: T03-W01.

Owns: expression-bodied methods/getters, `var` locals/`foreach` variables whose
exact admitted type is available, ordinary non-global namespace `using`
directives, and the exact redundant file-wide `#nullable enable`; normalizes
them to the same internal form as block bodies, explicit types, fully qualified
names, and the profile's already enabled nullable context.

Exit gate: normalized artifacts and obligations are byte-identical to explicit
equivalents; ambiguous, anonymous, dynamic, target-typed, disallowed, or
nullable-inconsistent inference rejects with frozen precedence. Global/static/
alias/generated imports, `extern alias`, disposal `using`, imported MPK
namespaces, scoped or non-enable nullable directives, and every other source
directive reject and emit no partial artifacts.

Verification: equivalence pairs and negative Roslyn-shape mutations in
`csharp_practical_syntax.rs`.

### CSHARP-03-T03-W03 — Admit enum, readonly struct, and sealed immutable class declarations

Depends on: T03-W02.

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
`with`, unsupported base lists, interfaces, operators, conversions, and
mutable/unsafe/ref-like layouts reject. No source exception declaration is an
accepted or emitted T03 data output, and none can be misclassified as an
ordinary sealed class.

Verification: declaration/type/default matrix tests, ordinary-data-path
exclusion of source-exception candidates, classifier handoff tests that make no
base-clause acceptance claim, and source/runtime differential vectors.

### CSHARP-03-T03-W04 — Admit fields, properties, constructors, and invariants

Depends on: T03-W03.

Owns: the frozen field/property forms, getter normalization, acyclic
same-type constructor delegation, constructor selection and assignment order,
definite initialization, constructor-only construction, receiver-first
lowering and static resolution of admitted pure instance methods, construction
invariants, and public type invariants.

Exit gate: all stored members follow the frozen assignment multiplicity and
order and establish exactly one final value; partial construction,
constructor-delegation cycles, setter/alias escape, hidden mutable state,
virtual/dynamic/ambiguous instance dispatch, invalid overload, and unproved
invariant cases reject.

Verification: constructor delegation/member-order cases, receiver-first pure
instance-call equivalence, invariant attachment, and failure-precedence tests.

### CSHARP-03-T03-W05 — Admit `init`, `required`, and object initializers

Depends on: T03-W04.

Owns: ordered object-initializer lowering, required-member coverage, init-only
assignment state, duplicate/missing/member-order errors, and attribute-bypass
rejection.

Exit gate: initialized values are indistinguishable from the frozen canonical
construction sequence and satisfy construction/public invariants before use;
post-construction writes reject.

Verification: constructor-vs-initializer equivalence, required/init boundary
cases, synthesized/attribute mutations, and source-order assertions.

### CSHARP-03-T03-W06 — Lower structural equality and canonical ordering

Depends on: T03-W05.

Owns: internal `structural_equal` and `canonical_compare` over every admitted
recursive value, null, decimal, GUID, sequences, structural products, enums,
and business values; admitted primitive/string source equality; and source-defined
non-generic pure field-by-field helpers included in the captured closure.
Enforces the frozen total-key matrix and lexicographic rules. The internal
operations are contract/boundary/foundation vocabulary, not callable `Mpk.*`
C# methods.

Exit gate: no CLR identity, virtual equality, hash code, comparer, locale, or
insertion order is observable; float-containing/non-total types reject as map
or set keys.

Verification: algebraic finite-domain tests, corner vectors, and pinned-runtime
comparisons where the spec deliberately mirrors .NET.

### CSHARP-03-T03-W07 — Lower arrays with explicit ownership

Depends on: T03-W06.

Owns: fixed/bounded allocation, default-eligible versus fully initialized
elements, reads/writes/length/index checks, alias rules, active-foreach
mutation rejection, and freeze/escape behavior.

Exit gate: every access has ordered symbolic or structural bounds checks. A
negative or otherwise invalid C# length takes its frozen exception edge; a
constant/initializer above the profile maximum rejects structurally; a
symbolic length carries the separate profile-bound predicate and VC. Unique
mutable ownership cannot be duplicated, and every permitted transfer, return,
storage, or alias freezes/transfers it exactly as specified; non-defaultable
elements are initialized before read.

Verification: boundary lengths/indices, alias/use-after-freeze, foreach
mutation, and default-eligibility tests.

### CSHARP-03-T03-W08 — Lower bounded sequences and two-pass construction

Depends on: T03-W07.

Owns: projection of admitted arrays and bound non-generic immutable wrappers to
closed internal bounded sequences; length, indexed read, structural equality,
and applicable lexicographic ordering; plus filtered variable-result
construction using the exact count-then-allocate two-pass source form. The
first loop proves the output count, the fresh array allocation is exact, and
the second loop fills every index once in source order before publication.

Exit gate: count agreement, initialized-prefix, element invariant, exact
result length, profile bound, unique ownership, and final publication are
explicit and independently validated. The specialized monomorphic construction
state is explicit in VIR, cannot cross an incompatible merge or public
boundary, and publication eliminates it to an immutable value before
certificate generation; `List<T>`, immutable framework collections,
single-pass growth, callbacks/lambdas, and every
source-visible builder API reject.

Verification: zero/full/filtered-result cases, count/fill disagreement,
premature publication/read, ownership transfer, profile-capacity boundaries,
determinism, residual-construction-state rejection, and wrapper-binding
mutations.

### CSHARP-03-T03-W09 — Lower canonical ordered maps and sets

Depends on: T03-W08.

Owns: semantic binding of an admitted element array as a set and an array of an
application-owned non-generic immutable entry type as a map; optional bound
non-generic wrappers; source loops/pure helpers for count, membership, lookup,
ordered traversal, direct canonical construction, duplicate rejection or
replacement, and a proved closed sort/dedup algorithm; internal ordered map/set
projection; missing-key versus stored-null outcomes; canonical key ordering;
and key admissibility.

Exit gate: the source representation is bounded, strictly sorted, unique, and
independent of insertion order at each publication point; application helper
bodies remain in the captured closure. Custom comparers, runtime hashing,
float/non-total or mutable keys, generic entry types, framework collections or
enumeration, insertion-order assumptions, and source-visible map/set builders
reject.

Verification: entry/array/wrapper binding mutations, permutation and canonical
order tests over the key matrix, count/membership/lookup loops, duplicate
reject/replace paths, missing/stored-null lookup, bounds, and rejected
comparer/hash/framework cases.

### CSHARP-03-T03-W10 — Lower strings, characters, and boundary codec relations

Depends on: T03-W09.

Owns: ordinal UTF-16 operations, allowed string methods, exact string/string and
string/char concatenation, restricted interpolation normalized to the same
bounded concatenation, null/empty rules, surrogate handling, and the frozen
culture-free boundary codec relations for all admitted primitives/business
values. Source string operations and internal boundary codecs remain distinct;
the codec relation does not admit a general source parse/format call.

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

Owns: exact float/double bit values and operations, NaN/signed-zero behavior,
decimal coefficient/scale representation, checked operations, rounding and
overflow, conversions, and ordering exclusions.

Exit gate: emitted operations reproduce every frozen runtime bit/value/error
vector without theory primitives or checker floating/decimal primitives;
unsupported casts/transcendentals/non-total ordering reject.

Verification: exhaustive small domains, edge bit vectors, decimal scale and
rounding tables, differential harness, and canonical encode/decode mutations.

### CSHARP-03-T03-W12 — Lower nullable and closed outcome values

Depends on: T03-W11.

Owns: nullable reference operations; the exact compiler-owned value-type `T?`
source spelling and immediate `option<T>` specialization; the restricted
conditional-access and null-coalescing matrices; and semantic binding of
application-owned non-generic option, lookup, result, and accumulating-
validation representations, plus an application-owned three-arm
representation bound to internal `boundary_field<T>` semantics. Includes
active/inactive payloads, actual default arm, missing/null/value distinction,
exhaustive matching, deterministic array-
based error order/bounds, nesting restrictions, and the one frozen
`lookup<option<T>>` exception.

Exit gate: annotations never substitute for runtime null; missing key differs
from stored null; inactive-payload access and empty `Invalid` construction take
their exact frozen exceptional edges and must be proved unreachable, caught,
or declared. Each application binding passes total, arm-distinct,
observation-preserving projection/reconstruction and operation-commutation
checks. Parameterless default fallback rejects when its payload is not
`default_eligible`, while an explicit fallback carries its public-invariant
obligation; nested `option` forms outside the frozen exception reject during
source/contract validation. `T?` near-misses and all other constructed generic
source types, including explicit `System.Nullable<T>`, reject.

Verification: full arm/payload matrix, null/missing/value boundaries,
error accumulation order/capacity, match exhaustiveness, and mutation tests.

### CSHARP-03-T03-W13 — Lower calendar, time, GUID, instant, and money values

Depends on: T03-W12.

Owns: exact DateOnly/TimeOnly/TimeSpan/GUID intrinsics; application-owned
non-generic instant wrappers or explicitly classified raw instant boundary
carriers bound to the registered non-template internal instant definition; and
application-owned non-generic money values bound to the internal `money<C>`
template. Includes construction, operations, comparison, exact boundary
codecs, calendar/leap/day-of-week rules, wrap/carry, precision/range,
no-generation policy, and money currency/scale/rate/division/rounding/error
precedence.

Exit gate: every operation uses only explicit inputs; date/time range failures
take their frozen exception or result edge; instant precision/range and money
currency/scale/division/overflow failures produce their frozen error arms;
ambient clock/time-zone, random GUID, implicit rounding, and unsupported codec
or source forms reject during source/contract validation.

Verification: complete boundary/differential tables, leap/calendar cases,
instant difference extremes, GUID ordering/codecs, and money operation matrix.

### CSHARP-03-T03-W14 — Close semantic binding, specialization, and data emission

Depends on: T03-W13.

Owns: closure of data-relevant type/method contract expression parsing and
attachment; strict semantic-binding parsing/attachment; logical declaration
identity versus source provenance; reachable binding closure; inferred closed
argument IDs; tag/payload/member/default/bound validation; derivation of the
exact closed root/provenance set from actual source and sidecars; invocation
of the T02-W02 specialization engine; deterministic emission of that engine's
validated concrete definitions before VIR; complete data-phase emission,
diagnostics, and negative-case ownership. It does not implement a second
closure, identity, ordering, deduplication, limit, or expansion algorithm.

Exit gate: stale/colliding/missing/duplicate/unreachable/cyclic bindings and
tampered foundation descriptors, instance tables, manifests, or expansion
reject. No source-defined generic, unsupported constructed CLR type, generic
call, or template node survives the VIR barrier; no monomorphic sequence-
construction state crosses an incompatible merge/publication boundary or
is live at a function or public-value boundary. T06-W09 separately owns proof
that no such state is encoded into a certificate. Frozen built-in data-operation
exception values and normal/exceptional successors are independently
validated, while every source exception declaration or explicit handler form
remains outside T03 acceptance and emits no T03 artifact; T04 owns its final
validation and diagnostics. No MPK application dependency is introduced;
iterator and async families are covered only by the frozen rejection vectors.
Every T03 vector runs through the real frontend and importer, and the T03
review has zero findings.

Verification: C# subset/contracts/lowering/emission suites and all T03 fuzz
seeds in two isolated runs, `./scripts/build-csharp-frontend.sh --check`, and
`./scripts/check-fast.sh`. The native installed-release gate remains owned by
T07/T08.

## 9. CSHARP-03-T04 — Control frontend

### CSHARP-03-T04-W01 — Parse and attach loop contracts

Depends on: T03-W14.

Owns: loop invariant, optional decreases, modifies/ownership facts, normal and
abrupt exit claims, strict loop-to-sidecar attachment, expression typing, and
complete loop nesting/accounting.

Exit gate: every admitted loop has the required contract; missing, duplicate,
wrong-target, ill-typed, impure, or out-of-scope facts reject before lowering.

Verification: contract attachment/typing/precedence tests for each loop form
and nested-loop mutation cases.

### CSHARP-03-T04-W02 — Lower structured loops and abrupt edges

Depends on: T04-W01.

Owns: canonical CFG for admitted `for`, `while`, `do`, and exact compiler-
recognized `foreach` over arrays or strings; stable IDs, invariant entry/back-
edge/exit points, decreases, break/continue, nested targets, and partial-versus-
total metadata.

Exit gate: source-order evaluation and every normal/abrupt edge are explicit;
unsupported or irreducible loop forms and ambiguous targets reject. A partial
loop may omit decreases only when its partial-termination status remains
explicit; a total claim requires the frozen decreases obligations.

Verification: CFG golden vectors, loop interpreter differential cases, stable
ID determinism, and invariant/decreases boundary tests.

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
patterns reject deterministically. Property getters must be total, pure, and
immutable.

Verification: decision-graph goldens, guard ordering, exhaustiveness/overlap,
binding scope, runtime differential, and compiler-upgrade cases.

### CSHARP-03-T04-W04 — Admit closed exception declarations and explicit throw sites

Depends on: T04-W03.

Owns: extension of declaration validation for the exact source-exception base
clauses reserved by T03-W03; the closed exception sum; parameterless admitted
built-in construction; source-exception declarations and frozen payload
constructors with the exact parameterless `System.Exception` base call;
standalone `throw new ExactException(...)`; method exceptional result sets;
exceptional postcondition attachment; and uncaught classification. It consumes
the frozen built-in operation exception values and successors already emitted
by the T03 data-operation owners and does not define a second conversion for
them.

Exit gate: runtime exception objects/messages/stacks never enter artifacts;
each throw has one typed closed value and edge. Unknown, dynamic, wrapped, or
reflection-origin exception forms, message/inner/runtime-state constructors,
throw expressions, `throw null`, stored/reused exception objects, and
observation of message/stack/runtime identity reject. A closed exception
outside the method's declared `throws` set instead creates the exact catch-or-
unreachable obligation and must be caught or proved unreachable before
verified acceptance.

Verification: source-exception base-clause, constructor, tag, payload, throw,
and uncaught vectors plus exceptional contract type tests.

### CSHARP-03-T04-W05 — Lower catch, filters, finally, and propagation

Depends on: T04-W04.

Owns: lexical inner-to-outer handler search, filter-before-finally evaluation,
filter throws, ordered catch selection, finally on every exit, override/
propagation behavior, the exact bare `throw;` rethrow within its active catch,
return/break/continue interaction, and explicit CFG regions/edges.

Exit gate: every source-design search/unwind case has one canonical trace;
hidden runtime dispatch, unspecified order, exception suppression outside the
frozen rule, rethrow outside the active catch, and source control that exits a
`finally` via return or outward break/continue/goto reject.

Verification: trace differential corpus, region/edge goldens, filter/finally
mutation matrix, rethrow cases, and abrupt-control combinations.

### CSHARP-03-T04-W06 — Close control emission and validation

Depends on: T04-W05.

Owns: independent VIR validation for loop/pattern/exception regions, complete
maps/manifests, control diagnostics/counters/fuzz seeds, and T04 cross-feature
cases; validates propagation of partial/total metadata through the reachable
call graph and rejects a partial callee on a total path.

Exit gate: malformed dominance, region, target, handler, ordering, or source
mapping rejects independent of Roslyn; all T04 source cases are deterministic;
review has zero findings.

Verification: control importer tests, bounded pattern/exception/CFG fuzzing,
full C# control frontend suite, and `./scripts/check-fast.sh`.

## 10. CSHARP-03-T05 — Boundary and transition frontend

T05 implements an MPK verification overlay around the unchanged selected
application method. It does not define an application runtime protocol,
serializer, framework integration, or deployable MPK library.

### CSHARP-03-T05-W01 — Validate boundary sidecars and presence bindings

Depends on: T04-W06.

Owns: strict parsing and attachment of the frozen boundary schema/version,
semantic context, selected-method identity, ordered input/output fields, exact
admitted value types, document/value limits, and parse/format profile; plus
application-owned non-generic presence types bound to internal
`boundary_field<T>` semantics with exact missing/null/value arms, payload,
default, invariant, and optional-missing mapping.

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

Exit gate: successful new-command paths establish the new-state invariant,
version increment, event/response correspondence, and all bounds; business-
error paths leave the input state unchanged. The signature and application
build name no MPK type or component. Persistence, transactionality, locks,
delivery, retries, clock, identity generation, authentication truth, and
transport remain explicit external assumptions and cannot enter the selected
closure or certificate.

Verification: binding/signature/contract attachment mutations, success and
every business-error arm, invariant/version/event/response counterexamples,
explicit-time cases, effect-firewall rejection, and source-to-runtime finite
differential traces in `csharp_practical_transition.rs`.

### CSHARP-03-T05-W05 — Lower optional full-snapshot idempotency and precedence

Depends on: T05-W04.

Owns: the optional processed-command record containing the key, complete
application-owned `Command` and `Context` snapshots, and stored response; the
source-defined field-complete snapshot equality helper; proof that it is
equivalent to equality of exact canonical field encodings; reflexive-equality
eligibility; retained-key replay/mismatch behavior; new-key version and history
capacity behavior; event suppression on replay; and exact check precedence.

Exit gate: after ordinary boundary preconditions, an existing key is checked
first. Equal complete snapshots return unchanged state, no events, and the
stored response; different snapshots return the explicit idempotency-conflict
error. A new key then checks expected version and finally bounded history
capacity. No digest substitutes for source equality, no smaller caller
projection is selectable, and float/double or recursively non-reflexive
snapshot fields reject the claim. A project without the complete record may
verify the transition only with idempotency disabled.

Verification: field-by-field equality/encoding equivalence, omitted-field and
collision-assumption mutations, missing-versus-null snapshots, reflexivity
rejection, replay/mismatch/version/capacity precedence matrix, event ordering,
and the no-idempotency variant.

### CSHARP-03-T05-W06 — Close boundary and transition emission

Depends on: T05-W05.

Owns: independent importer validation for boundary/transition bindings and
linkage; total-termination propagation for the selected root and its reachable
call/loop closure; diagnostics, counters, source maps, manifests, fuzz seeds,
cross-feature cases, and the T05 review ledger.

Exit gate: every boundary and transition vector enters through actual captured
C# plus overlay sidecars, survives independent generic-free monomorphic VIR
validation, and has deterministic input/output linkage. Any partial loop or
callee on the public root, application MPK dependency, serializer/effect
bypass, malformed binding, residual generic/template value, iterator, or async
shape rejects; review has zero findings.

Verification: complete boundary and transition suites twice, bounded sidecar/
adapter/protocol fuzzing, totality and effect-firewall mutations,
`./scripts/build-csharp-frontend.sh --check`, and `./scripts/check-fast.sh`.
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
deterministic two-run evidence, and the T06 review ledger.

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
trees/archives; any archive/file/mode/flag/reference/environment mutation fails.

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
and the final review ledger is empty. No activation has occurred yet.

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
| registered foundation descriptor and closed specialization | T01-W02/W08/W09 | T02-W01-W06, T03-W14, T06-W06/W09/W10, T07-W02/W06 |
| application semantic bindings and projection obligations | T01-W08/W09 | T02-W03-W05, T03-W08/W09/W12-W14, T05-W01/W04, T06-W01/W06 |
| expression bodies, `var`, imports, nullable directive | T01-W04 | T03-W02 |
| enum/struct/ordinary immutable class/default eligibility | T01-W04/W08 | T03-W03, T06-W02 |
| fields/properties/constructors/instance methods/init/required | T01-W04/W08 | T03-W04/W05, T06-W02 |
| structural equality and ordering | T01-W04/W08 | T03-W06, T06-W03 |
| arrays and internal sequence construction | T01-W04/W08 | T02-W02/W03, T03-W07/W08, T06-W03 |
| ordered maps and sets | T01-W04/W08 | T03-W09, T06-W03 |
| UTF-16 strings and codecs | T01-W04/W07 | T03-W10, T06-W03/W07 |
| float/double and decimal | T01-W04/W07 | T03-W11, T06-W03/W09 |
| nullable and application-owned closed outcomes | T01-W04/W08 | T03-W12, T06-W03/W06 |
| date/time/duration/instant/GUID/money | T01-W04/W08 | T03-W13, T06-W03/W06 |
| loops and contracts | T01-W05/W09 | T04-W01/W02, T06-W04 |
| switch and patterns | T01-W05 | T04-W03, T06-W04 |
| exceptions, exact source-exception bases, and abrupt completion | T01-W05/W09 | T04-W04-W06, T06-W05 |
| iterator/async/task/enumeration exclusions | T01-W06/W09 | T03-W01/W14, T05-W06, T08-W06 |
| boundary JSON, overlay linkage, three-state presence | T01-W08/W09 | T05-W01-W03/W06, T06-W07 |
| pure transitions and optional full-snapshot idempotency | T01-W08/W09 | T05-W04-W06, T06-W08 |
| contract expression union | T01-W09 | T03-W04-W14, T04-W01/W04/W06, T05-W01/W04/W06, T06-W01 |
| closed framework surface and effect firewall | T01-W03/W06/W08/W09 | T03-W01/W10/W13/W14, T05-W02/W04/W06, T06-W10/W11, T07-W03/W04 |
| successor VIR/source artifacts/maps/manifests | T01-W02/W09 | T02-W02-W07 |
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
