# CSHARP-03 Practical C# Implementation Milestones and Tasks

Status: planning complete; execution blocked by `JAVA-03-T10`.

Source design: `08_csharp_practical_subset_design.md`.

This document decomposes the source design's eight serial CSHARP-03 stages
into implementation-sized work items. It is subordinate to the source design,
the specifications and vectors frozen by `CSHARP-03-T01`, and the active
release contracts. It does not itself freeze a profile identity, schema,
artifact hash, compiler observation, runtime observation, or deterministic
limit.

## 1. Entry gate, authority, and stop conditions

The only permitted entry edge is:

```text
JAVA-03-T10 -> CSHARP-03-T01-W01
```

`JAVA-03-T10` must have atomically activated Java and recorded its complete
local native Linux release evidence before W01 starts. Until then, this file is
planning material only. In particular, no CSHARP-03 specification, vector,
identity, hash, build input, production code, fixture, candidate bundle, or
public route may be frozen or changed from this plan.

The following are unconditional stop conditions:

- an exact Roslyn or .NET behavior required by the design cannot be reproduced
  through a disposable public-API/runtime probe;
- any requested capability, including iterator or async source shape, cannot
  be encoded with ordinary Certificate v0 terms, zero new axiom categories,
  an empty proof-node table, and an empty theory-certificate table;
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
another language phase.

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
   execution tests.
4. Reject every unknown enum/tag/field/node/type/operation and every
   out-of-limit input before emitting downstream artifacts.
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

Unless T01 records a measured reason to change a location, implementation
ownership is divided as follows:

| Surface | Primary location |
| --- | --- |
| C# capture, Roslyn adapter, subset, contracts, lowering, emission | `csharp-tools/csharp2vir/` |
| Semantic context, registry, artifact validation, VIR, VC, hashes | `crates/mpk-vc/src/` |
| Installed frontend, sandbox, CLI, policy, evidence, AI | `crates/mpk-cli/src/` |
| Public structured API | `crates/mpk-api/src/successor_api.rs` and routed tests |
| Normative specifications and vectors | `develop/specs/` and `develop/specs/vectors/` |
| Probes, migration reports, receipts, review ledgers | `develop/migrations/csharp-03/` |
| Build inputs and immutable bundles | `release/build-inputs/csharp/` and `release/bundles/` |
| Practical examples | `examples/csharp-practical/` |
| Existing policy fixture to repair | `fixtures/csharp/policy/` |

New files named in the task tables are expected test/owner names. T01 may
rename them once, before downstream implementation, and must update this file
and the traceability ledger atomically. Production logic must not live only in
test harnesses.

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

Until T08-W01 repairs and regenerates it, nothing under
`fixtures/csharp/policy/` may count as CSHARP-03 frontend, policy, evidence,
certificate, AI, or release acceptance evidence. Earlier tasks use dedicated
new fixtures and retain the old fixture only as a negative mismatch case.

## 4. Common definition of done

Each stage gate requires all of the following for the capability completed so
far:

- positive, negative, boundary, precedence, deterministic, mutation, and
  compiler/runtime-upgrade cases have exactly one implementation-test owner;
- accepted source runs twice from isolated inputs and emits byte-identical
  canonical artifacts; applicable cases also compare against the pinned .NET
  runtime over a finite domain;
- source maps cover original bytes, manifests account for every selected and
  reachable input, and hashes bind the exact semantic context and profile;
- failures are artifact-free, bounded, sanitized, and follow the frozen phase
  and within-method diagnostic precedence;
- no project file, NuGet resolution, ambient assembly, culture, clock,
  randomness, network, credential, mutable global state, task scheduler, or
  host path can affect an accepted result;
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
| `CSHARP-03-T05` | 6 | iterator and scheduler-unobservable async projection | laziness/disposal and erasure equivalence pass |
| `CSHARP-03-T06` | 12 | VC, certificate, boundary/transition, policy/evidence/AI/API integration | same certificate bytes pass both checkers with zero axioms |
| `CSHARP-03-T07` | 6 | reproducible registered private candidate and native sandbox evidence | two builds/runs and hostile-environment gates pass |
| `CSHARP-03-T08` | 10 | repaired evidence, three examples, complete rehearsal, atomic activation | sole installed successor release; zero findings |

### 5.1 Primary implementation-test routing

The paths below are the planned primary production executors. Multiple work
items may share a test binary, but each owns a distinct test module/prefix
matching its full W ID. T01-W10 freezes the exact names and puts each frozen
vector row under exactly one of these owners. In this table only,
`Tnn-Waa-Wbb` denotes the inclusive range Tnn-Waa through Tnn-Wbb.

| Work items | Primary implementation test |
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
| T03-W01/W02 | `crates/mpk-cli/tests/csharp_practical_capture.rs`, `crates/mpk-cli/tests/csharp_practical_syntax.rs` |
| T03-W03-W06 | `crates/mpk-cli/tests/csharp_practical_types.rs` |
| T03-W07-W09 | `crates/mpk-cli/tests/csharp_practical_collections.rs` |
| T03-W10/W11 | `crates/mpk-cli/tests/csharp_practical_codecs.rs`, `crates/mpk-cli/tests/csharp_practical_numbers.rs` |
| T03-W12-W14 | `crates/mpk-cli/tests/csharp_practical_domain.rs` |
| T04-W01-W06 | `crates/mpk-cli/tests/csharp_practical_control.rs` |
| T05-W01-W06 | `crates/mpk-cli/tests/csharp_practical_suspension.rs` |
| T06-W01-W09 | `crates/mpk-vc/tests/csharp_practical_vc.rs` |
| T06-W10-W12 | `crates/mpk-cli/tests/csharp_practical_policy_verify.rs`, `crates/mpk-api/tests/csharp_practical_api.rs` |
| T07-W01/W02 | `crates/mpk-cli/tests/csharp_practical_build_inputs.rs`, `crates/mpk-cli/tests/csharp_practical_release_bundle.rs` |
| T07-W03-W06 | `crates/mpk-cli/tests/csharp_practical_frontend_runner.rs`, `crates/mpk-cli/tests/csharp_practical_release_gate.rs` |
| T08-W01 | existing C# fixture policy/evidence/AI tests plus `crates/mpk-cli/tests/csharp_practical_fixture.rs` |
| T08-W02-W05 | `crates/mpk-cli/tests/csharp_practical_examples.rs` and example-local .NET tests |
| T08-W06-W10 | `crates/mpk-cli/tests/csharp_practical_release_gate.rs`, `crates/mpk-cli/tests/successor_atomic_cutover.rs` |

## 6. CSHARP-03-T01 — Feasibility and normative freeze

T01 changes specifications, vectors, probes, and the ledger only. It must not
change production acceptance or install a candidate. All observed values are
captured from the final pinned toolchain; documentation recollection is not
evidence.

### CSHARP-03-T01-W01 — Close the entry audit and baseline receipt

Depends on: `JAVA-03-T10`.

Owns: confirm Java activation; capture the installed release registry and all
active tuple/hash/checker identities; record the exact Go/Rust/scalar-C#/Java
test corpus, release command, axiom inventory, and clean-worktree commit in
`develop/migrations/csharp-03/baseline.json`; create the ledger with every W
item in this document.

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

Exit gate: `artifact-consumer-inventory.json` has no unowned read/write/hash
edge, distinguishes active and private routes, and identifies the single
atomic migration set and whole-image rollback set.

Verification: repository search fixtures prove that deleting or adding a
known consumer makes the inventory test fail.

### CSHARP-03-T01-W03 — Pin and prove the frontend/toolchain closure

Depends on: T01-W02.

Owns: measure and freeze the .NET SDK/runtime, Roslyn assemblies/analyzers,
reference pack, native host, source/build flags, archive hashes/modes/notices,
offline extraction, and exact deterministic build inventory; prohibit project
files, restore, NuGet/package-cache discovery, generators, analyzers outside
the inventory, and ambient references.

Exit gate: two clean offline builds from frozen bytes are identical; every
one-byte, file-count, mode, flag, reference, and environment mutation fails
before publication; the candidate remains unregistered.

Verification: add `crates/mpk-cli/tests/csharp_practical_build_inputs.rs` and
extend the build-input script tests;
run the isolated two-build recipe and
`./scripts/build-csharp-frontend.sh --check-build-inputs`. The native
`./scripts/check-csharp-frontend.sh` gate is reserved for T07/T08.

### CSHARP-03-T01-W04 — Measure Roslyn data and construction shapes

Depends on: T01-W03.

Owns: disposable probes for expression bodies, `var`, enums, readonly structs,
sealed immutable classes, fields/properties/constructors, `init`, `required`,
object initializers, synthesized members, nullable annotations, conversions,
arrays, collection calls, strings, and every proposed data intrinsic.

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

### CSHARP-03-T01-W06 — Measure iterator and async shapes

Depends on: T01-W05.

Owns: disposable compiler/runtime probes for iterator capture, yield/break,
early disposal and exception ordering, single-use consumption, immediate and
completed tasks, exception propagation, permitted await sites, task erasure,
async iterators, and every rejected scheduler/effect/task escape.

Exit gate: accepted behavior is expressible without generated state-machine
names or scheduler observations; iterator/async source shapes and runtime
traces are canonical; any delayed/custom awaitable or observable scheduling
case rejects.

Verification: repeat traces under perturbed culture, environment, thread-pool
settings, and process paths; canonical results remain identical.

### CSHARP-03-T01-W07 — Measure primitive, string, numeric, and codec behavior

Depends on: T01-W06.

Owns: finite runtime probes for UTF-16/surrogates, ordinal string operations,
the exact string/char concatenation matrix, null behavior, parse/format
grammars, float/double result bits including NaN and signed zero, decimal
coefficient/scale/rounding/overflow, and culture-independent error precedence.

Exit gate: every operation has an exact accepted domain, result encoding,
error, precedence, and differential vector; no result depends on ambient
culture or general framework parsing/formatting.

Verification: run under at least two hostile cultures and mutated runtime
inputs; compare canonical bits/values and rejection IDs.

### CSHARP-03-T01-W08 — Freeze collection, outcome, and business-value semantics

Depends on: T01-W07.

Owns: the final sequence/builder/map/set APIs and capacities; ownership/freeze
rules; duplicate add/replace precedence; recursive default eligibility;
structural equality and total-key ordering matrix; nullable/Option/Lookup/
Result/Validation forms and nesting exceptions; DateOnly/TimeOnly/TimeSpan/
Instant/Guid/Money representations, operations, codecs, ranges, and errors.

Exit gate: every source API maps to one closed semantic operation; active and
inactive payload rules, bounds, day-of-week mapping, instant granularity,
Money currency/scale/rounding rules, and all exclusions are explicit.

Verification: spec-model table completeness tests and independent finite .NET
differential probes for every runtime-backed row.

### CSHARP-03-T01-W09 — Freeze contracts, boundary, transition, identities, and limits

Depends on: T01-W08.

Owns: final type/method/boundary/transition sidecar schemas and expression
union; missing/null/value canonical JSON; raw-input/canonical-output linkage;
transition version/idempotency/event semantics; all successor identity and
hash-domain names; whether one context-dispatched C# executable safely serves
both scalar and practical profiles; diagnostic families/precedence; exact
measured limits classified as pre-invocation structural counters or admitted
run-time value predicates/VCs; and producer/consumer ownership from W02.

Exit gate: no “provisional”, “working”, or unmeasured value remains in a
normative row; unknown fields/tags reject; each counter has one increment site,
one comparison rule, and boundary vectors; all final names are globally unique
and immutable.

Verification: strict schema/hash/limit/precedence tests, later-version and
duplicate-key mutations, and checker capacity probes at limit minus one,
limit, and limit plus one.

### CSHARP-03-T01-W10 — Publish and review the complete freeze package

Depends on: T01-W09.

Owns: the practical-profile specification package, successor shared-artifact
specifications, exact vectors and manifest entries, canonical probe records,
name-and-owner inventory, traceability ledger, upgrade matrix, and routing
updates in the design/todo/developer documentation.

Exit gate: every source-design requirement, vector field, semantic row,
diagnostic, limit, implementation surface, and release criterion maps to one
later W item and one primary test; all hashes are recomputed; a complete
specification review produces zero findings. Production behavior is unchanged.

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
dispatch, installed-identity assertions, and append-only predecessor checks.

Exit gate: every frozen identity/hash/mutation/context vector runs through
production validation code; unknown and mixed language/profile pairs reject;
the released registry still exposes only the predecessor revision.

Verification: successor registry runtime tests plus all revision-1/2/3
predecessor vector tests and `successor_atomic_cutover` rejection cases.

### CSHARP-03-T02-W02 — Implement values, collections, sums, and business foundations

Depends on: T02-W01.

Owns: the frozen VIR/foundation representations for enums, immutable records,
arrays, builders, bounded sequences, ordered maps/sets, strings/chars,
float/decimal bits, nullable and tagged sums, boundary presence, dates/times/
duration/instant/Guid/Money, plus canonical encoders and closed validators.

Exit gate: recursive well-formedness, default eligibility, active-payload,
capacity, key-order, scale/range, and canonical encoding rules reject before
use; no framework object or serializer representation enters an artifact.

Verification: new `csharp_practical_foundation.rs` vectors including every tag,
recursive boundary, inactive payload mutation, and canonical round trip.

### CSHARP-03-T02-W03 — Implement operation and explicit-control vocabulary

Depends on: T02-W02.

Owns: closed operation/check tags for the frozen data APIs and codecs; builder
state transitions; loop/control nodes; pattern decisions; explicit exception
values, handler regions, abrupt edges; iterator/suspension projection; and
unknown-tag rejection.

Exit gate: operand/result types, ordered checks, normal/exceptional successors,
ownership state, and suspension points are structurally validated without
invoking the C# runtime.

Verification: table-driven operation/control vectors and one mutation for
every tag, arity, type, edge, and ordering rule.

### CSHARP-03-T02-W04 — Implement successor source artifacts and linkage

Depends on: T02-W03.

Owns: frozen selection/type/method/boundary/transition artifacts, foundation
and operation/check tables, original-source capture, raw-boundary-byte and
canonical-value linkage, context linkage, source-map and both manifest stages,
canonical serialization, and every hash domain.

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
exception and suspension structure, and complete foundation/operation linkage.

Exit gate: only frozen vocabulary is admitted; malformed or over-limit inputs
fail before VC construction; import has no compiler/runtime callback; old
parsers reject successor bytes and successor parsers reject old-family bytes
where accepting both would be ambiguous.

Verification: `csharp_practical_vir_validation.rs`, bounded parser fuzz seeds,
and cross-artifact/profile/hash mutation suites.

### CSHARP-03-T02-W06 — Implement successor VC skeleton, VC, and hash models

Depends on: T02-W05.

Owns: the successor skeleton/VC schemas, canonical ordering, context and VIR
linkage, ordinary-term type/value encoding, obligation groups, limits, and
hashes while leaving Certificate v0 and checker acceptance unchanged.

Exit gate: all new values and control forms have a closed encoding path; empty
proof/theory tables and zero axioms are structurally enforced; no unproved
intrinsic is accepted.

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
choose old versus new artifacts via a public flag.

Verification: per-language semantic-difference reports, two-run artifact
equality, and old/new interpreter/VC/checker comparison over the complete
accepted and rejected corpora.

### CSHARP-03-T02-W09 — Migrate consumers and close the private foundation gate

Depends on: T02-W08.

Owns: private successor consumers in VC, certificate assembly, policy/evidence,
CLI, release, AI explanation, API, fixture tooling, and documentation; removes
private dual-format fallbacks; records the complete migration receipt.

Exit gate: every W02 inventory edge consumes exactly one successor format,
predecessor equivalence tests pass, no public practical route exists, and T02
review has zero findings.

Verification: targeted consumer tests, inventory search/mutation tests,
complete predecessor local gates, and `./scripts/check-fast.sh`.

## 8. CSHARP-03-T03 — Data frontend

Every T03 accepted case starts as immutable captured `.cs` and exact JSON
sidecars, passes the pinned Roslyn public APIs, and is independently validated
after emission. A helper-constructed VIR is not acceptance evidence.

### CSHARP-03-T03-W01 — Extend capture, declaration accounting, and closure

Depends on: T02-W09.

Owns: selected source roots, immutable byte capture, encoding/path checks, all-
declaration accounting for selected files, reachable method/type closure,
finite call/type graphs, recursion rejection, and closed framework symbol/API
admission.

Exit gate: dead declarations in selected files are classified, every reachable
declaration is captured, unselected/ambient/generated/project/package input
rejects, and closure limits are enforced before lowering.

Verification: extend C# capture/frontend vectors with multi-file, dead-tree,
cycle, source-root, path, encoding, ambient-reference, and limit cases.

### CSHARP-03-T03-W02 — Normalize expression bodies and exact `var`

Depends on: T03-W01.

Owns: expression-bodied methods/getters and `var` locals whose exact admitted
type is available; normalizes them to the same internal form as block bodies
and explicit types.

Exit gate: normalized artifacts and obligations are byte-identical to explicit
equivalents; ambiguous, anonymous, dynamic, target-typed, disallowed, or
nullable-inconsistent inference rejects with frozen precedence.

Verification: equivalence pairs and negative Roslyn-shape mutations in
`csharp_practical_syntax.rs`.

### CSHARP-03-T03-W03 — Admit enum, readonly struct, and sealed immutable class declarations

Depends on: T03-W02.

Owns: declaration modifiers, members, source enums including exact underlying
values and `System.DayOfWeek`, immutable type graphs, inheritance/identity/
mutation rejection, and recursive default eligibility.

Exit gate: every admitted instance is a structural value; unknown enum values,
casts, zero/default cases, cycles, mutable/static/virtual/reflection/identity
escapes, and ineligible defaults follow frozen rules.

Verification: declaration/type/default matrix tests and source/runtime
differential vectors.

### CSHARP-03-T03-W04 — Admit fields, properties, constructors, and invariants

Depends on: T03-W03.

Owns: the frozen field/property forms, getter normalization, constructor
selection and assignment order, definite initialization, constructor-only
construction, construction invariants, and public type invariants.

Exit gate: all stored members initialize exactly once under the frozen order;
partial construction, setter/alias escape, hidden mutable state, invalid
overload, and unproved invariant cases reject.

Verification: constructor/member order cases plus invariant attachment and
failure-precedence tests.

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

Owns: `Mpk.Value.Equal`/`Compare` over every admitted recursive value, null,
decimal, Guid, sequences, records, enums, and business values; enforces the
frozen total-key matrix and lexicographic rules.

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

Exit gate: every access has ordered symbolic or structural bounds checks;
ownership cannot duplicate or escape; non-defaultable elements are initialized
before read.

Verification: boundary lengths/indices, alias/use-after-freeze, foreach
mutation, and default-eligibility tests.

### CSHARP-03-T03-W08 — Lower bounded sequence builders and immutable sequences

Depends on: T03-W07.

Owns: builder create/add/add-range/filter/freeze operations, linear state,
capacity and result-length bounds, immutable sequence indexing/enumeration, and
variable-result construction.

Exit gate: capacity precedence and freeze state are exact; use-after-freeze,
double freeze, alias, escape, over-capacity, and unbounded growth reject.

Verification: state-machine transition matrix, filtered construction, boundary
capacity, determinism, and mutation tests.

### CSHARP-03-T03-W09 — Lower canonical ordered maps and sets

Depends on: T03-W08.

Owns: builders, add/replace/duplicate behavior, lookup, freeze, capacity,
canonical key ordering/enumeration, map stored-null versus missing semantics,
and key admissibility.

Exit gate: output is independent of insertion order; duplicate/error
precedence is exact; custom comparers, hashing, float keys, mutable keys, and
unbounded collections reject.

Verification: permutation tests over the key matrix, duplicate add/replace,
missing/stored-null lookup, capacity, and rejected comparer/hash cases.

### CSHARP-03-T03-W10 — Lower strings, characters, parsing, and formatting

Depends on: T03-W09.

Owns: ordinal UTF-16 operations, allowed string methods, exact string/string and
string/char concatenation, null/empty rules, surrogate handling, and the frozen
culture-free codecs for all admitted primitives/business values.

Exit gate: only intrinsic constant ordinal options are accepted; char/char,
object conversion, culture-sensitive/general framework calls, noncanonical
forms, bad ranges, and null receiver/argument cases follow frozen results.

Verification: grammar and round-trip corpus, hostile cultures, surrogate/null
boundaries, concatenation matrix, and pinned-runtime differential cases.

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

Owns: nullable reference/value operations and the frozen Option, Lookup,
Result, and accumulating Validation APIs; active/inactive payloads, exhaustive
matching, deterministic error order/bounds, nesting restrictions, and the one
frozen Lookup<Option> exception.

Exit gate: annotations never substitute for runtime null; missing key differs
from stored null; default-ineligible fallback and empty-invalid validation
reject; nested Option forms outside the frozen exception reject.

Verification: full transition/payload matrix, null/missing/value boundaries,
error accumulation order/capacity, match exhaustiveness, and mutation tests.

### CSHARP-03-T03-W13 — Lower calendar, time, Guid, and Money values

Depends on: T03-W12.

Owns: DateOnly/TimeOnly/TimeSpan/Instant/Guid/Money construction, operations,
comparison, exact codecs, calendar/leap/day-of-week rules, wrap/carry,
precision/range, no-generation policy, and Money currency/scale/rate/division/
rounding/error precedence.

Exit gate: every operation uses only explicit inputs; invalid date/time/range,
ambient clock/time-zone, random Guid, currency mismatch, scale mismatch,
division/rounding, and unsupported codec cases reject exactly.

Verification: complete boundary/differential tables, leap/calendar cases,
instant difference extremes, Guid ordering/codecs, and Money operation matrix.

### CSHARP-03-T03-W14 — Admit boundary/transition declarations and close the data gate

Depends on: T03-W13.

Owns: source attachment and typing of frozen boundary and transition sidecars,
three-state presence types, pure transition signatures, version/idempotency/
event response shapes, complete data-phase emission, diagnostics, counters,
and negative-case ownership.

Exit gate: sidecars bind exact source/type/method IDs and context; effects,
serializer/framework calls, persistence/transport/time/random access, unknown
fields, and invalid transition shapes reject; every T03 vector runs through
the real frontend; T03 review has zero findings.

Verification: C# subset/contracts/lowering/emission suites, two isolated runs,
all T03 fuzz seeds, `./scripts/check-csharp-frontend.sh`, and
`./scripts/check-fast.sh`.

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

Owns: canonical CFG for admitted `for`, `while`, `do`, and closed `foreach`,
stable IDs, invariant entry/back-edge/exit points, decreases, break/continue,
nested targets, and partial-versus-total metadata.

Exit gate: source-order evaluation and every normal/abrupt edge are explicit;
unbounded/unsupported loop forms and ambiguous targets reject.

Verification: CFG golden vectors, loop interpreter differential cases, stable
ID determinism, and invariant/decreases boundary tests.

### CSHARP-03-T04-W03 — Lower switch and admitted patterns

Depends on: T04-W02.

Owns: source-order arms/guards, exhaustiveness, null/type/property/list patterns,
bindings and scopes, decision nodes, and unmatched behavior.

Exit gate: pattern selection matches frozen Roslyn/runtime probes; dynamic,
recursive/open, identity, side-effecting, nonexhaustive, or unsupported
patterns reject deterministically.

Verification: decision-graph goldens, guard ordering, exhaustiveness/overlap,
binding scope, runtime differential, and compiler-upgrade cases.

### CSHARP-03-T04-W04 — Lower closed exception values and throw sites

Depends on: T04-W03.

Owns: the exception sum, admitted built-in error conversion, explicit throw,
method exceptional result sets, exceptional postcondition attachment, and
uncaught classification.

Exit gate: runtime exception objects/messages/stacks never enter artifacts;
each throw has one typed closed value and edge; undeclared/dynamic/wrapped/
reflection-origin exceptions reject.

Verification: exception constructor/tag/payload/throw/uncaught vectors and
exceptional contract type tests.

### CSHARP-03-T04-W05 — Lower catch, filters, finally, and propagation

Depends on: T04-W04.

Owns: lexical inner-to-outer handler search, filter-before-finally evaluation,
filter throws, ordered catch selection, finally on every exit, override/
propagation behavior, return/break/continue interaction, and explicit CFG
regions/edges.

Exit gate: every source-design search/unwind case has one canonical trace;
hidden runtime dispatch, unspecified order, exception suppression outside the
frozen rule, and unsupported nesting reject.

Verification: trace differential corpus, region/edge goldens, filter/finally
mutation matrix, and abrupt-control combinations.

### CSHARP-03-T04-W06 — Close control emission and validation

Depends on: T04-W05.

Owns: independent VIR validation for loop/pattern/exception regions, complete
maps/manifests, control diagnostics/counters/fuzz seeds, and T04 cross-feature
cases.

Exit gate: malformed dominance, region, target, handler, ordering, or source
mapping rejects independent of Roslyn; all T04 source cases are deterministic;
review has zero findings.

Verification: control importer tests, bounded pattern/exception/CFG fuzzing,
full C# control frontend suite, and `./scripts/check-fast.sh`.

## 10. CSHARP-03-T05 — Suspension frontend

### CSHARP-03-T05-W01 — Admit closed single-use iterators

Depends on: T04-W06.

Owns: iterator signature/source restrictions, capture inventory, closed
consumers, no escape/re-enumeration/concurrent use/aliasing, and ownership of
enumerator lifetime.

Exit gate: every iterator has exactly one statically known consumer and bounded
yield count; LINQ, general interfaces, boxing, heap escape, multiple consumers,
and ambient enumeration reject.

Verification: source closure and ownership matrix with every rejection route.

### CSHARP-03-T05-W02 — Lower iterator state, contracts, and disposal

Depends on: T05-W01.

Owns: capture state, lazy entry, yield/break, per-yield/prefix/completion and
exceptional obligations, early consumer break, disposal/finally ordering, and
canonical suspension projection without generated names.

Exit gate: runtime traces and VIR projection agree for full, prefix, early-
break, exceptional, and disposal paths; IDs/artifacts are deterministic.

Verification: iterator trace differential corpus, contract attachment,
projection goldens, and disposal/order mutations.

### CSHARP-03-T05-W03 — Admit immediate/completed-task async source

Depends on: T05-W02.

Owns: permitted async signatures/await expressions, exact completed-task
producers, no task/effect escape, no delayed/custom awaitable, no scheduler/
context/cancellation/concurrency observation, and finite call closure.

Exit gate: every await is proven immediate under the frozen closed producer
set; tasks cannot be stored, returned as business data, compared, combined,
timed, cancelled, or observed outside erasure.

Verification: accepted/rejected producer and escape matrix plus hostile
thread-pool/context probes.

### CSHARP-03-T05-W04 — Lower async by verified erasure

Depends on: T05-W03.

Owns: result and exception propagation, canonical removal of task wrappers and
await points, source-map retention, erasure-equivalence claims, and rejection
of generated state-machine details.

Exit gate: erased and pinned-runtime executions agree on result/exception and
side-effect-free trace for every finite case; artifacts expose no scheduler or
generated type identity.

Verification: paired sync/async source corpus, equivalence evaluator, exception
cases, deterministic artifact comparison, and upgrade mutations.

### CSHARP-03-T05-W05 — Admit and lower closed async iterators

Depends on: T05-W04.

Owns: the intersection of W01-W04 rules: single closed consumer, immediate
awaits, bounded yields, prefix/completion/disposal obligations, exception
propagation, and double erasure to the frozen suspension projection.

Exit gate: every async iterator reduces to the same observable value/exception
sequence as its closed synchronous projection; any delayed or escaped state
rejects.

Verification: full/prefix/break/exception/disposal corpus and scheduler-hostile
differential runs.

### CSHARP-03-T05-W06 — Close the suspension gate

Depends on: T05-W05.

Owns: independent suspension validation, iterator/async diagnostics and limits,
fuzz seeds, map/manifest coverage, all cross-feature suspension cases, and the
T05 review ledger.

Exit gate: every requested iterator/async capability has positive, rejection,
boundary, differential, determinism, and upgrade evidence; review has zero
findings. If not, stop and return to T01 rather than narrowing the profile.

Verification: complete iterator/async suites twice, bounded adapter/protocol
fuzzing, `./scripts/check-csharp-frontend.sh`, and `./scripts/check-fast.sh`.

## 11. CSHARP-03-T06 — Verification integration

### CSHARP-03-T06-W01 — Implement the complete contract expression union

Depends on: T05-W06.

Owns: strict parsing, typing, canonicalization, resource bounds, and source
attachment for every frozen type/method/boundary/transition expression form,
including structural projections, sums, collections, codecs, control state,
exception values, iterator prefixes, and transition relations.

Exit gate: every expression has one typed ordinary-term encoding; unknown,
impure, partial, ill-scoped, ambiguous, duplicate, or over-limit expressions
reject before VC generation.

Verification: exhaustive tag/typing/normalization vectors and parser fuzzing.

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
calendar/time/Guid/Money checks; and ordered error obligations.

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

### CSHARP-03-T06-W06 — Generate iterator and async obligations

Depends on: T06-W05.

Owns: laziness, capture initialization, per-yield/prefix/completion, early
disposal, exceptional iteration, single-consumer ownership, immediate-await
proofs, task/async-iterator erasure, and result/exception equivalence.

Exit gate: suspension verification uses only ordinary finite state and terms;
no scheduler, task, enumerator, or generated-machine theory enters checking.

Verification: proof/counterexample cases for every consumer path and erasure
mutation.

### CSHARP-03-T06-W07 — Verify canonical boundary round trips

Depends on: T06-W06.

Owns: strict canonical JSON parse, duplicate/unknown/required/non-null and
missing/null/value rules, numeric/text canonicality, depth/count/byte limits,
typed conversion, raw-input-to-value hash linkage, canonical output, and output
reparse equality.

Exit gate: serializer/runtime output is never proof authority; a certificate
binds exact raw input, typed canonical value, output bytes, and reparsed value;
all mutations break linkage or fail parsing.

Verification: boundary parser corpus/fuzzing, three-state field matrix, limit
boundaries, hostile serializer mutations, and round-trip proof cases.

### CSHARP-03-T06-W08 — Verify pure state transitions and idempotency

Depends on: T06-W07.

Owns: input/output state invariants, expected/current version, success/error
arms, ordered bounded events, response relation, explicit time, optimistic
conflict, idempotency replay and command-encoding match, history capacity, and
frozen error precedence.

Exit gate: the verified function is pure; persistence, locking, transport,
clock, identity generation, and infrastructure idempotency remain explicitly
outside the certificate.

Verification: transition matrix covering accept/error/replay/conflict/capacity/
encoding mismatch and broken invariant/event/response cases.

### CSHARP-03-T06-W09 — Encode ordinary foundations and close zero-axiom checking

Depends on: T06-W08.

Owns: ordinary core definitions and proof terms for all new finite values and
operations, certificate assembly, limits, same-byte dual-checker invocation,
and explicit enforcement of empty proof-node/theory-certificate tables and
total axiom count zero.

Exit gate: both checkers accept identical canonical certificate bytes and
reject every mutated proof/context/hash; Certificate v0 and both acceptance
rules are byte/behavior unchanged.

Verification: checker-agreement script, certificate mutation suite, axiom
inventory comparison, and full predecessor certificate corpus.

### CSHARP-03-T06-W10 — Integrate policy, evidence, and reproduction

Depends on: T06-W09.

Owns: practical-profile policy scan, evidence schemas, source/profile/context/
boundary/transition/checker linkage, provider redaction, registered-bundle-only
reproduction, and complete installed-source recipe.

Exit gate: evidence can be reproduced without compiler/runtime trust, raw
paths, credentials, network, or unregistered binaries; any linkage mutation
fails closed.

Verification: `csharp_practical_policy_verify.rs`, evidence schema/hash cases,
redaction tests, and offline reproduction from the private registered bundle.

### CSHARP-03-T06-W11 — Integrate AI explanation and structured API

Depends on: T06-W10.

Owns: practical-profile AI explanation and API request/response variants,
context propagation, boundary/transition summaries, bounded diagnostics,
provider isolation/redaction, and exact unsupported-version behavior.

Exit gate: AI output is non-authoritative and derived only from verified
artifacts; API callers cannot inject paths, toolchains, artifacts, compiler
output, or a different profile/context.

Verification: AI/API schema, redaction, route, version, batch, and cross-profile
mutation tests; public production route remains inactive.

### CSHARP-03-T06-W12 — Close end-to-end private verification

Depends on: T06-W11.

Owns: actual-source frontend-to-certificate private tests across every
capability, boundary and transition linkage, both checkers, all consumer
inventories, deterministic two-run evidence, and the T06 review ledger.

Exit gate: no helper-constructed IR is used as sole evidence, all frozen rows
have one passing and one failing proof where meaningful, predecessor
certificates remain identical in verdict/axioms, and review has zero findings.

Verification: all mpk-vc/mpk-cli/mpk-api practical suites, checker agreement,
predecessor suites, `./scripts/check-fast.sh`.

## 12. CSHARP-03-T07 — Reproducible release candidate

### CSHARP-03-T07-W01 — Finalize deterministic offline build inputs

Depends on: T06-W12.

Owns: the release build descriptor, source closure, final frontend/foundation
assembly inventory, runtime files, notices, modes, build recipe, deterministic
archive, and hostile ambient build checks.

Exit gate: two fresh offline builds produce byte-identical reviewed candidate
trees/archives; any archive/file/mode/flag/reference/environment mutation fails.

Verification: C# build-input tests and the exact two-clean-build recipe.

### CSHARP-03-T07-W02 — Assemble immutable toolchain and frontend bundles

Depends on: T07-W01.

Owns: candidate bundle descriptors, content hashes, member inventories,
launcher contract, semantic-context/profile linkage, native dependencies, and
private release-registry tuple.

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

Owns: private image assembly, all active tuples on the sole successor registry,
two builds/two runs, image mutation checks, predecessor/practical corpus,
policy/evidence/API/checker paths, and rollback image materialization.

Exit gate: the candidate image is byte-identical across builds, all runs are
deterministic, no old/staging compatibility route exists, and rollback restores
the exact W01 baseline image.

Verification: local native Linux candidate release gate twice and complete
predecessor equivalence reports.

### CSHARP-03-T07-W06 — Publish the private candidate receipt and close T07

Depends on: T07-W05.

Owns: immutable receipt hashes for toolchain/frontend/foundation/registry/
artifacts/checkers/image, build and native-gate evidence, limits, corpus,
axiom inventory, rollback target, known exclusions, and zero-finding review.

Exit gate: every receipt field is independently recomputable; no identity or
hash points to an unregistered/ambient item; T07 review has zero findings and
the active release remains unchanged.

Verification: receipt schema/recomputation tests, `./scripts/check-fast.sh`,
and a second native candidate run from the recorded inputs.

## 13. CSHARP-03-T08 — Complete rehearsal and atomic activation

### CSHARP-03-T08-W01 — Repair the existing C# policy fixture from actual source

Depends on: T07-W06.

Owns: correction of `fixtures/csharp/policy/source/src/Required.cs` and atomic
regeneration, through the real private installed frontend, of its selection,
context, maps, manifests, VIR, VC/skeleton, scan, evidence, certificate, hashes,
AI fixtures, and manifest ownership.

Exit gate: source syntax and artifacts agree; no manually attached VIR or stale
byte/hash remains; boundary bytes round-trip where applicable; both checkers
accept the exact recorded certificate bytes.

Verification: fixture reproduction from scratch, policy/evidence/AI tests,
checker agreement, and a mutation proving the old mismatch rejects.

### CSHARP-03-T08-W02 — Add the invoice pricing and tax example

Depends on: T08-W01.

Owns: runnable installed-source example with immutable request/result, Money,
currency/scale, business/effective dates, decimal rounding, ordered line
aggregation, bounded sequence builder, construction styles, contracts,
boundary input/output, artifacts, tests, and trust-boundary README.

Exit gate: positive and business-error cases run through scan/verify,
round-trip canonical JSON, and both checkers; README states exactly what is and
is not proved.

Verification: example-local runtime assertions plus installed frontend,
policy/evidence reproduction, and same-byte checker tests. Runtime assertions
use the pinned direct-compiler harness; no project or NuGet input is admitted
to the frontend.

### CSHARP-03-T08-W03 — Add the order transition example

Depends on: T08-W02.

Owns: runnable example with Guid command/idempotency keys, explicit Instant and
expected version, switch/pattern state logic, Result, one allowlisted caught
exception, replay-safe response, ordered bounded events, transition contract,
artifacts, tests, and trust-boundary README.

Exit gate: accept/error/replay/conflict/encoding-mismatch/capacity cases prove
the frozen relations without claiming persistence, locking, clock, identity
generation, or transport correctness.

Verification: transition matrix through installed scan/verify, boundary
round-trip, evidence reproduction, and both checkers.

### CSHARP-03-T08-W04 — Add the batch validation example

Depends on: T08-W03.

Owns: runnable example with canonical boundary JSON, missing/null/value, exact
codecs, ordered map/set duplicate handling, accumulating Validation, closed
iterator, immediate-await wrapper, artifacts, tests, and trust-boundary README.

Exit gate: duplicate/unknown/noncanonical/limit and validation-order cases are
demonstrated; iterator prefixes/disposal and async erasure are verified; both
checkers accept the exact certificates.

Verification: installed end-to-end positive/negative runs, boundary reparse,
iterator/async differential checks, evidence reproduction, and checker
agreement.

### CSHARP-03-T08-W05 — Close cross-example capability coverage

Depends on: T08-W04.

Owns: a machine-tested matrix proving the three examples collectively exercise
constructor-only and required/init/object-initializer construction, arrays,
builders/sequences/maps/sets, strings/codecs, loop invariant/decreases,
structural equality/order, nullable/outcomes, exception, iterator/async,
boundary/transition, and every business primitive.

Exit gate: every required row points to actual source span, emitted node,
obligation, certificate theorem, runtime case, and README trust statement; no
helper-only artifact satisfies a row.

Verification: fail the coverage test by deleting each kind of link; reproduce
all three examples twice from clean materializations.

### CSHARP-03-T08-W06 — Run complete conformance, fuzz, mutation, and upgrade gates

Depends on: T08-W05.

Owns: the aggregate practical corpus and implementation executors for every
positive/rejection/boundary/precedence/differential/determinism row; bounded
fuzzing of source/contract/Roslyn/pattern/exception/collection/codec/calendar/
boundary/transition/iterator/async/artifact protocols; and compiler/runtime/
schema/context/hash mutations.

Exit gate: all seeds/counters/time budgets are recorded and reproducible; every
frozen row executes in production code; two complete runs are identical; no
open crash, timeout, nondeterminism, or unowned vector remains.

Verification: practical release-gate script twice plus all recorded fuzz and
mutation commands and `./scripts/check-fast.sh`.

### CSHARP-03-T08-W07 — Prove complete predecessor and checker preservation

Depends on: T08-W06.

Owns: full Go/Rust/scalar-C#/Java source, VIR, VC, policy/evidence/API/release
corpora under the final candidate; semantic-difference reports; Certificate v0
byte/acceptance audit; same-byte dual-checker agreement; proof/theory table and
axiom inventory audit.

Exit gate: all predecessor behavior/obligations/verdicts are equivalent, all
new and old certificates retain allowed zero-axiom state, and neither checker
contains a profile-specific acceptance bypass.

Verification: complete predecessor local gates, checker agreement, axiom audit,
and repository search/mutation tests for bypasses and obsolete formats.

### CSHARP-03-T08-W08 — Finalize release docs, exclusions, activation, and rollback plan

Depends on: T08-W07.

Owns: README/developer/spec routing, exact profile capability/exclusion text,
upgrade policy, operations/security/trust boundary, installed release registry
and bundle change set, atomic ordering, failure points, no-dual-route searches,
and executable whole-image rollback procedure.

Exit gate: documentation never calls the profile full C# support; every final
identity/hash/limit is exact; activation is one atomic installed-image change;
rollback needs no artifact reinterpretation and returns to the recorded T07
baseline.

Verification: docs/link/spec checks, release descriptor dry run, failure
injection at each activation step, rollback drill, and obsolete-route search.

### CSHARP-03-T08-W09 — Run the final local release gates and zero-finding review

Depends on: T08-W08.

Owns: `./scripts/check-fast.sh`, the complete reviewed native root x86-64 Linux
assembly/image-mutation/syscall/cgroup/resource/cleanup gate twice, installed
example reproduction, release receipt draft, and whole-diff review/fix cycles.

Exit gate: every command passes from clean inputs on the exact candidate; both
native passes have identical hashes/verdicts; all reviewer findings are fixed
and the final review ledger is empty. No activation has occurred yet.

Verification: the commands and hashes recorded in the candidate receipt are
rerun by the final reviewer; any mismatch returns to the owning earlier item.

### CSHARP-03-T08-W10 — Atomically activate and record CSHARP-03 completion

Depends on: T08-W09 and transitively every earlier work item.

Owns: the one release commit that installs only the frozen successor registry
and bundles for all active profiles, removes executable private/staging and old
format routes, activates the practical C# tuple, publishes the three examples,
updates status/routing documents, and finalizes the release receipt.

Exit gate: one installed successor release serves Go, Rust, scalar C#, Java,
and practical C#; no compatibility selector or alternate registry is
executable; actual-source examples pass; receipt records all identities,
hashes, native evidence, checker agreement, empty proof/theory tables, total
axiom count zero, rollback target, and zero findings; `DART-04` is unblocked.

Verification: clean-image native release gate after activation, complete local
corpus and checker agreement, installed-route and obsolete-format searches,
rollback then re-activation rehearsal, and `./scripts/check-fast.sh`.

## 14. Requirement-to-work-item traceability

This table is a completeness index, not a second owner. Detailed ownership is
the task contract above and the frozen T01 ledger.

| Source design area | Freeze owner | Implementation/verification owner |
| --- | --- | --- |
| authority, trust, atomic migration | T01-W01/W02/W09 | T02-W08/W09, T07-W05/W06, T08-W08-W10 |
| source/declaration/call/type closure | T01-W03/W04 | T03-W01 |
| expression bodies and `var` | T01-W04 | T03-W02 |
| enum/struct/class/default eligibility | T01-W04/W08 | T03-W03, T06-W02 |
| fields/properties/constructors/init/required | T01-W04/W08 | T03-W04/W05, T06-W02 |
| structural equality and ordering | T01-W08 | T03-W06, T06-W03 |
| arrays, builders, sequences | T01-W04/W08 | T03-W07/W08, T06-W03 |
| ordered maps and sets | T01-W08 | T03-W09, T06-W03 |
| UTF-16 strings and codecs | T01-W07 | T03-W10, T06-W03/W07 |
| float/double and decimal | T01-W07 | T03-W11, T06-W03/W09 |
| nullable and closed outcomes | T01-W08 | T03-W12, T06-W03 |
| date/time/duration/instant/Guid/Money | T01-W08 | T03-W13, T06-W03 |
| loops and contracts | T01-W05/W09 | T04-W01/W02, T06-W04 |
| switch and patterns | T01-W05 | T04-W03, T06-W04 |
| exceptions and abrupt completion | T01-W05/W09 | T04-W04-W06, T06-W05 |
| iterators | T01-W06/W09 | T05-W01/W02/W06, T06-W06 |
| async/await and async iterators | T01-W06/W09 | T05-W03-W06, T06-W06 |
| boundary JSON and three-state presence | T01-W09 | T03-W14, T06-W07 |
| pure transitions and idempotency | T01-W09 | T03-W14, T06-W08 |
| contract expression union | T01-W09 | T06-W01 |
| closed library surface and effect firewall | T01-W03/W08/W09 | T03-W01/W14, T05-W03, T06-W10/W11, T07-W03/W04 |
| successor VIR/source artifacts/maps/manifests | T01-W02/W09 | T02-W02-W07 |
| VC/hash/certificate/zero axioms | T01-W09 | T02-W06, T06-W02-W09 |
| diagnostics, precedence, and limits | T01-W09 | each frontend task; gates T03-W14/T04-W06/T05-W06 |
| policy/evidence/AI/API | T01-W02/W09 | T02-W09, T06-W10-W12 |
| build, bundle, sandbox, release | T01-W03/W09 | T07-W01-W06, T08-W08-W10 |
| Required.cs repair and examples | T01-W10 vector ownership | T08-W01-W05 |
| aggregate conformance/fuzz/upgrade | T01-W04-W10 | T08-W06/W07/W09 |

## 15. Per-item handoff checklist

Before marking any work item complete, its commit and ledger row must answer
all of these without an implicit “as above”:

- Which exact predecessor receipt and frozen spec/vector hashes were inputs?
- Which production files, tests, vectors, fixtures, and docs changed?
- Which source-design rows and diagnostic/limit counters does the item own?
- What accepted, rejected, boundary, precedence, mutation, determinism, and
  upgrade cases were added, and which production test executes each?
- What canonical artifacts and hashes changed, and why are predecessor bytes
  or semantics unchanged where required?
- Which targeted commands and `./scripts/check-fast.sh` ran, on what host, and
  with what result? If native Linux evidence is required, where is its receipt?
- How was absence of an active/public/staging/ambient route checked?
- What review findings were found, how were they fixed, and where is the final
  zero-finding result?

If any answer is missing, the work item remains `In progress`.
