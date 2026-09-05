# Post-Rust Multi-Language Frontend Expansion Todo

Source design: `develop/docs/06_multilanguage_frontend_design.md`

Status: Gate C, `MLANG-01`, and Gate D are complete after the completed Rust
and `MLANG-00` gates. `CSHARP-02-T01` through `CSHARP-02-T20` and
`JAVA-03-T01` through T10 are complete, including the native x86-64 two-pass
release receipt. The checked-in Go/Rust/C#/Java
successor release is active. T01 completed
the inactive normative profile/vector/toolchain freeze and disposable
compiler/JVM compatibility probes. T02 completed the inactive offline build
and T03 completed the inactive contract/artifact validators. T04 completed
internal capture, the public compiler adapter and bounded diagnostics. T05
completed source admission, inert initialization, call closure and typed
sidecars. T06 completed private CFG/lowering and complete artifact emission;
T07 completed the private candidate/JVM runner and native x86-64 Linux
acceptance. T08 completed private verification integration, and T09 completed
conformance and local release rehearsal, and T10 implemented atomic activation.
Registry revision 3 and all five tuples are installed.
`CSHARP-03-T01-W01` through W10 completed the entry audit, probes,
foundation/freeze work, and normative package publication without adding an
installed practical-profile identity or production path.
T02-W01/W02/W03/W04/W05/W06/W07/W08/W09 and T03-W01/W02/W03/W04/W05/W06/W07/W08 are complete,
T03-W09 is ready, and all later items remain serially blocked.

## Scope and sequencing

This plan queues C#, Java, the published but inactive practical C# expansion,
Dart, TypeScript, and Python after the complete Go/VIR/Rust critical path. It
defines one flow; none of its production milestones overlap the Rust program
or another language milestone.

```text
VIR-00 -> VIR-01 -> GO-VIR-02 -> RUST-03 -> RUST-04 -> RUST-05
  -> RUST-06 -> RUST-07-T05
  -> MLANG-00 design and feasibility
  -> MLANG-01 successor contract freeze
  -> CSHARP-02
  -> JAVA-03
  -> CSHARP-03 practical C# expansion
  -> DART-04
  -> TYPESCRIPT-05
  -> PYTHON-06
```

Every production phase starts only after the preceding phase's exit gate. A
later language or source-profile expansion's feasibility, specification, and
implementation work are contained in its production phase and start only
after the preceding production phase's final release gate. A proposal-only
governance record may be prepared earlier but grants no implementation
authority.

## Common execution rules

1. Do not add placeholder future-language values to a frozen VIR, frontend,
   manifest, release, policy, evidence, or AI schema.
2. Do not start `MLANG-00` before `RUST-07-T05`, and do not start any later
   phase before its immediate predecessor completes.
3. Keep every compiler, analyzer, SDK, frontend, source, contract, IR, map,
   manifest claim, VC, policy output, and differential result untrusted.
4. Keep Certificate v0, core/checker semantics, source-free checker inputs, and
   the four axiom categories unchanged.
5. A language may own multiple globally unique, immutable semantic profiles.
   One request and one VIR module select exactly one language/profile pair; an
   implementation never infers behavior from `source_language` alone or reuses
   a profile owned by another language.
6. Unknown syntax, compiler nodes, types, conversions, operations, dispatch,
   targets, options, or semantics reject deterministically.
7. One VIR module has one language/profile. Cross-language composition uses
   checked hash-pinned certificate imports, not mixed VIR or FFI.
8. Evidence routes resolve only registered bundles and accept no raw compiler,
   analyzer, frontend, SDK, package-cache, or runtime path.
9. A compiler/API upgrade is a semantic migration with pinned before/after
   corpora, not an automatic dependency update.
10. New shared code requires demonstrated identical behavior in at least two
    completed frontends; speculative plugin infrastructure is out of scope.
11. Execute numbered tasks and language/profile phases in the displayed order.
    Do not overlap tasks within `MLANG-00` or `MLANG-01`, and do not begin a
    feasibility refresh or specification work for a later production phase
    while its predecessor is still active. A proposal-only governance record
    creates no profile identity or implementation authority.
12. Execute `CSHARP-02-T01` through `CSHARP-02-T20` serially under
    `csharp-02-implementation-traceability-ledger.md`. T02 through T19 remain
    inactive staging; T20 alone may activate the successor release.

## Common definition of done for a language/profile phase

Each production language or source-profile expansion phase must provide:

- frozen subset, semantic-profile, toolchain, frontend, selection, contract,
  diagnostic, limit, and upgrade contracts;
- complete positive/negative/boundary/mutation/hash vectors with manifest test
  ownership;
- pinned build and execution inputs plus deterministic registered bundles;
- source-to-VIR lowering, source maps, both manifest stages, VC generation,
  policy/evidence/reporting, and structured reproduction recipes;
- exact rejection for every omitted construct family;
- differential execution on the pinned runtime for bounded cases;
- two-build/two-run determinism and path/credential/network isolation;
- parser/protocol/compiler-output/resource fuzz gates;
- canonical certificates accepted from identical bytes by both checkers; and
- a zero-new-category axiom review and empty implementation review ledger.

## MLANG-00: Design and feasibility phase

Entry gate: `RUST-07-T05` complete. This phase starts after the Rust program,
never during it. Its planned committed outputs are documentation and non-
normative feasibility evidence; a defect in the completed Go/Rust path pauses
the phase for an explicit serial remediation milestone.

### MLANG-00-T01 Build the semantic comparison matrix

Status: Complete (2026-08-25).

Depends on: `RUST-07-T05`.

Deliverable:

- `mlang-00-semantic-comparison-matrix.md`, the non-normative closed inventory
  of 34 candidate behaviors and 170 single-disposition language cells, with
  required-check and proposed-foundation catalogs plus a blocked unresolved-
  question ledger.

Tasks:

1. Record types, integer/float behavior, evaluation order, conversions, null,
   heap, exceptions, calls, dispatch, initialization, concurrency, async, and
   target/runtime variability for all five languages.
2. Map each behavior to an existing VIR operation, a required-check rule, a
   proposed new checked foundation, or an explicit initial rejection.
3. Record unresolved questions without inventing a default.

Exit gate: every candidate initial operation has exactly one disposition and
no unresolved item is marked accepted.

Completion evidence: every language cell has exactly one of existing VIR,
required check, proposed checked foundation, or initial rejection; all 16
unresolved questions are marked `blocked`; no future-language schema/profile,
production path, release entry, or axiom category was added.

### MLANG-00-T02 Validate official compiler integration boundaries

Status: Complete (2026-08-25).

Depends on: `MLANG-00-T01`.

Deliverable:

- `mlang-00-compiler-integration-feasibility.md`, the non-normative reviewed
  feasibility records for all five official compiler/analyzer boundaries,
  including deterministic input closures and 170 explicit integration
  go/no-go verdicts.

Tasks:

1. Confirm the minimum pinned APIs needed to obtain resolved syntax, symbols,
   types, control flow, source positions, diagnostics, and target/options.
2. Inventory compiler/analyzer/SDK/reference/standard-library/package inputs
   required for an isolated deterministic invocation.
3. Identify APIs that are unstable, internal, target-specific, ambient-state
   dependent, or unable to express required semantics.
4. Keep experiments non-normative and disconnected from `mpk-cli`, release
   bundles, and production VIR emission.

Exit gate: each language has a reviewed feasibility record and an explicit
go/no-go list for its candidate initial subset.

Completion evidence: every language record names supported public APIs for all
seven required fact categories, inventories compiler/analyzer, host, SDK,
reference/standard-library, package, source, option, and isolation inputs, and
identifies forbidden internal or ambient surfaces. The 55 `GO` and 115
`NO-GO` results cover every `M01`-`M34` row exactly once per language; no T01
foundation/rejection was upgraded, and no experiment was connected to a CLI,
release bundle, registry, or production VIR path.

### MLANG-00-T03 Audit the completed Go/Rust path for speculative hooks

Status: Complete (2026-08-25).

Depends on: `MLANG-00-T02`.

Deliverable:

- `mlang-00-go-rust-shared-boundary-audit.md`, the completed code, schema,
  release, executable, callback, and profile-default audit; its closed blocker
  ledger includes the bounded `MLANG-00-T03-R01` remediation and repeated
  zero-finding audit.

Tasks:

1. Inspect the completed VIR-01, GO-VIR-02, and RUST-03..RUST-07 path for
   language-name branches, raw tool paths, and implicit profile defaults.
2. Record any correctness defect or dormant plugin, callback, registry-
   executable, or future-language enum as a phase blocker. Insert and complete
   a bounded serial remediation task, then repeat this audit before
   `MLANG-01`; do not defer the finding into a parallel track.
3. Confirm that abstractions already shared by Go and Rust are sufficient as
   implemented, without editing them speculatively for a future language.

Exit gate: Go and Rust demonstrate the shared boundary without production code
or accepted schema values for C#, Java, Dart, TypeScript, or Python.

#### MLANG-00-T03-R01 Close the ambient reference-checker launch

Status: Complete (2026-08-25).

Depends on: the blocking first-pass finding in `MLANG-00-T03`.

Tasks:

1. Replace both production `go run` reference-checker launches with one fixed,
   deterministic executable payload embedded in `bin/mpk` and executed from a
   sealed anonymous descriptor under a closed environment and bounded I/O/time.
2. Build that payload with the digest-pinned Go image and require byte equality
   in release check and fixture modes without adding it to a registry or
   installed bundle inventory.
3. Retain each package certificate's Rust-checked bytes and submit that exact
   slice to the Go checker rather than reopening its pathname.
4. Prove package and installed Rust policy verification work with
   `PATH=/nonexistent`, then repeat the T03 audit.

Exit gate: required dual-checker verification has no ambient Go, source-tree,
checker-path, registry-executable, callback, or plugin dependency, and the
repeated T03 blocker ledger is empty.

T03 completion evidence: language/profile parsers, release tuples, frontend
paths, compiler callbacks, production process launches, and future-language
literals were re-audited. The only future-language values in active
non-documentation artifacts are three unknown-language rejection mutations for
`typescript`.
The reference-checker asset rebuilds byte-identically, both checkers receive
the same candidate bytes, and the installed `policy verify` fixture succeeds
with no usable host `PATH`. Existing VIR, VC, map/manifest, release, policy,
evidence, certificate, and AI boundaries required no speculative extension.

## MLANG-01: Rust feedback and successor contract freeze

Entry gate: `MLANG-00` complete.

### MLANG-01-T01 Audit the implemented Go/Rust abstractions

Status: Complete (2026-08-25).

Depends on: `MLANG-00-T03`.

Deliverable:

- `mlang-01-go-rust-csharp-gap-audit.md`, the implementation-backed complete
  C# semantic and cross-contract gap ledger with one class and owner per gap.

Tasks:

1. Compare the implemented VIR, VC, source-map, manifest, runner, selection,
   release, policy, evidence, and AI contracts with the C# candidate subset.
2. Classify every gap as profile-only, registry/selection shape, VIR operation,
   checked foundation/theory, or unsupported.
3. Use the Rust implementation evidence rather than the pre-implementation
   design to decide what is genuinely reusable.

Exit gate: the gap ledger is complete and has no ambiguous owner.

T01 completion evidence: all 34 C# semantic rows occur exactly once: the 18
compiler-API-feasible candidate rows are profile-only against the implemented
VIR/VC core, while the blocked rows are one VIR-operation gap, two checked-
foundation/theory gaps, and 13 unsupported rows. Four disjoint successor
registry/selection-shape gaps cover semantic identity, source selection,
release/execution descriptors, and policy/evidence/AI registration. All ten
required contract surfaces have an explicit reuse decision and next normative
owner. No production code, schema, vector, bundle, or accepted C# value was
added.

### MLANG-01-T02 Freeze the successor extension mechanism

Status: Complete (2026-08-25).

Depends on: `MLANG-01-T01`.

Deliverable:

- `../specs/SEMANTIC_PROFILE_REGISTRY_V1.md` and
  `../specs/vectors/semantic-profile-registry-v1.json`, the normative but
  inactive closed hash-pinned successor mechanism and its owned conformance
  baseline.

Tasks:

1. Choose a closed tagged-union revision or a closed hash-pinned semantic-
   profile registry revision.
2. Freeze strict validation, unknown-profile rejection, registry-root
   identity, selection/parameter schema ownership, versioning, hashes, limits,
   status precedence, and vectors.
3. Define the atomic migration of active Go/Rust producers and consumers if a
   shared schema changes.
4. Prove that the mechanism cannot load executable validator/checker plugins.

Exit gate: one normative successor design exists; no production parser/emitter
precedes it and no released dual IR input is planned.

T02 completion evidence: the chosen registry freezes exact closed root, entry,
identity, parameter, selection, and compiled-profile envelope shapes;
revision-1 Go/Rust hashes; limits; validation/status precedence;
compiled-contract closure; all affected successor identities and hash domains;
and whole-release atomic migration with bidirectional old/new rejection. The
owner test executes all transport, registry, context, profile-envelope, hash,
limit, no-plugin, and migration vectors. Registry revision 1 contains no C#
entry, no production parser/emitter accepts a successor schema, Certificate v0
and both checkers are unchanged, and `MLANG-01-T03` retains sole ownership of
exact C# content and revision 2.

### MLANG-01-T03 Freeze the C# specification package

Status: Complete (2026-08-25).

Depends on: `MLANG-01-T02`.

Deliverables:

- `../specs/CSHARP_PROFILE_V0.md`, the normative, then-inactive exact C# scalar
  profile and implementation handoff;
- `../specs/vectors/csharp-profile-v0.json`, the complete semantic,
  toolchain, reference, contract, diagnostic, limit, isolation, and upgrade
  vector package; and
- `../specs/vectors/semantic-profile-registry-v2.json`, the immutable C#
  entry and append-only revision-2 root.

Tasks:

1. Freeze the initial C# subset and exact Roslyn/compiler/target/reference
   inventory.
2. Freeze overflow context, conversions, evaluation/abrupt-completion rules,
   contracts, diagnostics, limits, VIR mapping, and required checks.
3. Add every conformance and hash vector plus implementation-test ownership.

Exit gate: every accepted C# form has exact semantics and every other form
rejects before VIR publication.

T03 completion evidence: the profile fixes C# 14, Roslyn 5.6.0, .NET SDK
10.0.400, runtime/reference pack 10.0.11, Linux x64, every archive and selected
assembly hash, and the complete 167-assembly reference inventory. It closes
the five admitted scalar types, explicit checked/unchecked overflow contexts,
all identity/implicit/explicit conversion forms, evaluation and abrupt-
completion behavior, contracts, public Roslyn operation/CFG boundary, source
mapping, diagnostics, limits, and required checks. The owner test recomputes
all profile, selection, sidecar, contract, toolchain, entry, reference, and
registry hashes; covers all 34 semantic rows and every vector category; and
proves revision 2 preserves the revision-1 Go/Rust entries. No production
parser, bundle, release tuple, policy route, checker rule, or active C# value
was added.

## CSHARP-02: Implement and release C#

Entry gate: `MLANG-01` complete.

Status: Complete (2026-08-30). `CSHARP-02-T01` through `CSHARP-02-T20`
completed serially, including the atomic successor cutover.

Authoritative execution plan:
`csharp-02-implementation-traceability-ledger.md`.

Deliverables:

- isolated pinned `csharp2vir` frontend over the frozen Roslyn boundary;
- deterministic registered frontend/toolchain bundles;
- candidate subset lowering and exact required safety checks;
- source maps, manifests, VC/policy/evidence integration;
- positive, negative, differential, fuzz, determinism, and upgrade corpora;
- a representative canonical certificate accepted by both checkers.

Exit gate: all common definition-of-done items pass, C# is active through the
sole shared path, and the review ledger is empty.

The phase is strictly serial:

```text
T01 -> T02 -> T03 -> T04 -> T05 -> T06 -> T07 -> T08 -> T09
  -> T10 -> T11 -> T12 -> T13 -> T14 -> T15 -> T16 -> T17
  -> T18 -> T19 -> T20
```

T02 through T19 may add candidate code and private staging only. The current
Go/Rust schemas remain the sole production inputs until T20 switches the whole
release atomically.

### CSHARP-02-T01 Create the C# implementation decomposition and traceability ledger

Status: Complete (2026-08-25).

Depends on: `MLANG-01-T03`.

Deliverable:

- `csharp-02-implementation-traceability-ledger.md`, assigning every frozen
  C# and successor requirement to one of 20 ordered tasks, an implementation
  test owner, and an inactive or activation boundary.

Exit gate: all specification sections, vector fields, semantic rows, compiled
profile contracts, successor identities, and common definition-of-done items
have unambiguous ownership; no production or normative artifact changes.

Completion evidence: all 31 C# profile-vector fields, 12 registry-v1 fields,
10 registry-v2 fields, 34 semantic rows, nine compiled-profile contracts, and
22 successor identities are covered exactly once by the primary-owner
ledgers. T02 is the sole ready successor and the T01 review ledger is empty.

### CSHARP-02-T02 Create the isolated pinned C# frontend project

Status: Complete (2026-08-26).

Depends on: `CSHARP-02-T01`.

Exit gate: an unregistered `csharp2vir` candidate builds twice to identical
bytes from the exact offline input closure; no parser, bundle registration, or
active route exists.

Completion evidence: `scripts/build-csharp-frontend.sh --check` validates all
six frozen archives, exact SDK/runtime extraction inventories and modes, the
four package graphs, both Roslyn projections, and all 167 references; it then
performs two no-restore builds in separate network namespaces and requires
byte-identical frontend and notice inventories. The pinned candidate DLL is
5,120 bytes with SHA-256
`76aadd20282a655783089cf8148ef3fc627b73f26da7fcc48a653c844ca63b26`.
Only `--version` succeeds, the active registry remains Go/Rust-only, and the
raw archive cache is ignored and untracked.

### CSHARP-02-T03 Implement the inactive semantic-profile registry core

Status: Complete (2026-08-26).

Depends on: `CSHARP-02-T02`.

Exit gate: registry/context/profile-envelope vectors execute against private
production validation code while released inputs remain unchanged.

Completion evidence: `mpk-vc::semantic_profile_registry` implements strict
registry transport, hashes, limits, identity precedence, closed models,
finite Go/Rust/C# dispatch, status mapping, revision lookup, and append-only
validation behind an explicitly injected inactive boundary. The runtime owner
executes all 87 revision-1 transport/hash/registry/context/profile/limit
cases, both revision-2 hash cases, all eight append-only cases, the exact C#
parameters, and all nine C# compiled-profile payloads. Both frozen vector byte
hashes are unchanged, their manifest owners are appended, and active VIR v0
and semantic-profile parsers still reject successor/C# identities.

### CSHARP-02-T04 Stage successor source-artifact models and hash domains

Status: Complete (2026-08-27).

Depends on: `CSHARP-02-T03`.

Exit gate: successor VIR/frontend/map/manifest artifacts validate and hash in
staging with bidirectional old/new rejection and no active producer.

Completion evidence: `mpk-vc::successor_source_artifacts` provides sealed,
registry-injected models for successor VIR/contracts, source maps, and source
manifests under the four new source-artifact hash domains. The hidden
`mpk-cli::successor_frontend_protocol` validator requires exact typed request
identity, canonical LF-framed transport, status/exit agreement, and complete
artifact linkage. The two primary implementation owners prove canonical
round trips, the frozen normalized C# contract hash, all common context and
artifact mismatches, and bidirectional v0/v1 rejection. The registry vector
bytes and active producers/consumers remain unchanged.

### CSHARP-02-T05 Implement C# selection, path preflight, and immutable capture

Status: Complete (2026-08-27).

Depends on: `CSHARP-02-T04`.

Exit gate: exact selection and source-transport mutations fail at their owned
phase, and successful capture never rereads an original input.

Completion evidence: the pinned candidate implements the exact ordered private
CLI assertions, canonical method/path/selection validation, the frozen
215-byte `MPK-CSHARP-SELECTION-0.1` hash, Linux no-follow type/link/inventory
preflight, checked inclusive file/snapshot counters, one-read immutable bytes,
and strict UTF-8/LF source transport. The executable offline harness covers
selection/assertion drift, path and inventory mutations, symbolic and hard
links, immutable-after-source-mutation behavior, encoding mutations, and
file/total/entry boundaries. The candidate remains unregistered and stops
before Roslyn without a partial artifact.

### CSHARP-02-T06 Build and validate the exact Roslyn compilation session

Status: Complete (2026-08-27).

Depends on: `CSHARP-02-T05`.

Exit gate: the frozen public Roslyn session and reference closure match at
getter/API level, with every drift case rejected before lowering.

Completion evidence: the candidate loads only the two frozen Roslyn managed
assemblies, constructs exact SHA-256 `SourceText` and C# 14 regular trees in
selection order, validates the canonical 167-file reference projection before
creating assembly references, and creates the exact x64 Release compilation.
Syntax and compilation diagnostics are queried in source/metadata order, and
the public semantic-model, symbol, type, conversion, operation, and
`IMethodBodyOperation` CFG adapters use only the frozen cancellation and
accessibility arguments. The pinned executable harness covers all getter
families, reference/option/API mutations, diagnostic precedence, and M33. The
candidate remains unregistered and stops before subset admission without a
partial artifact.

### CSHARP-02-T07 Enforce the C# subset, closure, purity, and initialization

Status: Complete (2026-08-27).

Depends on: `CSHARP-02-T06`.

Exit gate: every omitted construct rejects before VIR and every accepted
method belongs to one deterministic pure acyclic closure.

Completion evidence: the private candidate validates exact declaration,
identifier, source-type, literal, statement, operator, conversion, call, and
abrupt-completion forms against public Roslyn symbols and operations. Closure
discovery follows every source static call, including source-dead calls,
rejects cycles and unrelated methods, proves immutable parameters, local
definite assignment, purity, and inert type initialization, and stores methods
in deterministic callee-first order. Reference-identity operation unions, CFG
blocks, syntax nodes, and closure totals enforce all six T07 limits before an
excess item is retained. The pinned offline harness executes all 16
`reject_before_vir` rows and exact 128/129 closure boundaries. The candidate
remains unregistered and stops before contract parsing without a partial
artifact.

### CSHARP-02-T08 Implement typed C# contracts and attachment

Status: Complete (2026-08-27).

Depends on: `CSHARP-02-T07`.

Exit gate: every closure method has one exact normalized/hash-bound contract,
and all malformed, missing, duplicate, or unused sidecars reject.

Completion evidence: the private candidate parses only the closed strict JSON
sidecar union, rejects duplicate members, validates canonical typed integers,
checks the exact successor expression rules, and enforces clause, node, closure,
depth, and operator-arity bounds before excess retention. It verifies and
carries the T05 selection hash, attaches exactly one sidecar to every closure
method in deterministic callee-first order, normalizes the complete successor
semantic context, and reproduces the frozen 440-byte sidecar and 1,151-byte
contract hash payloads exactly. The pinned offline harness covers every stable
contract diagnostic, M34, all operator/type families, and exact limit
boundaries. Raw bytes remain a distinct untrusted input identity. The candidate
remains unregistered and stops before lowering without a partial artifact.

### CSHARP-02-T09 Lower scalar C# operations and required checks

Status: Complete (2026-08-27).

Depends on: `CSHARP-02-T08`.

Exit gate: scalar/control/conversion lowering is deterministic and every
required check is complete, unique, and canonically ordered.

Completion evidence: the private candidate lowers Bool and signed/unsigned
BV32/BV64 constants, locals, copies, returns, eager expressions, short
circuits, branches, joins, and early returns in stable order. It owns all 34
non-call mappings, all 12 Roslyn checked-state cases, all 20 conversion rules,
and all 15 T09 semantic rows. Checked arithmetic, division/remainder guards,
and masked shifts produce the exact per-instruction checks and a canonical
required-check ledger; missing, extra, and reordered checks reject before emission.
The pinned executable harness runs in the reproducible offline build, while
`CallStatic`, public serialization, and every production route remain closed.
The reviewed candidate DLL is 158,720 bytes with SHA-256
`bf94b267c3a67af9057ce103cbffbe3bebfeb307f4ebe7c3335a2756d94bc81e`.

### CSHARP-02-T10 Lower static calls and emit stable VIR, maps, and manifests

Status: Complete (2026-08-27).

Depends on: `CSHARP-02-T09`.

Exit gate: every accepted source emits complete staged successor artifacts
with faithful origins, stable IDs, call dependencies, and exact hashes.

Completion evidence: the candidate lowers exact-signature direct static calls
left to right, links each `CallStatic` to the callee contract hash, and orders
the acyclic closure callee first with lexical tie-breaking. Structural
function, binding, block, instruction, and terminator IDs do not depend on
Roslyn ordinals or captures. The source mapper rejects split surrogate, empty,
out-of-range, and external spans and maps every emitted node from exact UTF-16
boundaries to UTF-8 source bytes without synthetic origins. The emitters
produce domain-hashed canonical `mpk.vir.v1`, `mpk.source_map.v1`, frontend-
stage `mpk.source_manifest.v1`, and LF-framed `mpk.frontend.cli.v1` success
bytes with the exact C# `vir`, `source_map`, and `manifest` profile contracts.
The pinned harness proves deterministic bytes and the primary Rust owner feeds
the real envelope to the shared successor validators without a C# adapter.
The reviewed candidate DLL is 190,464 bytes with SHA-256
`c2a999ded31b8825670fcc824719162efb5c5d40ae7c93140411242820bb43ee`.
The active release registry remains Go/Rust-only.

### CSHARP-02-T11 Close C# frontend diagnostics, limits, and source-case vectors

Status: Complete (2026-08-27).

Depends on: `CSHARP-02-T10`.

Exit gate: all frontend-owned accepted/rejected, diagnostic, precedence,
limit, and hash vectors execute against candidate code with exact artifact-
free non-success behavior and no active C# route.

Completion evidence: the candidate owns all 44 stable diagnostics, two-stage
compiler-issue normalization and redaction, exact status/phase/exit
precedence, all 32 inclusive profile/operational limits, bounded artifact and
protocol writers, and canonical artifact-free failure envelopes. Its pinned
aggregate runs all prior C# mutation owners, all 30 accepted source pipelines,
all 88 rejection IDs, all 34 semantic rows, and every diagnostic, precedence,
limit, and hash vector. The Rust primary owner checks exact report closure and
feeds every non-success envelope through the shared successor validator. The
reviewed candidate DLL is 208,384 bytes with SHA-256
`0783dc269c152ad1b13e77f42f9eff6f6891002c65890bc1445f2fe1a1a0410d`.
The active release remains Go/Rust-only.

### CSHARP-02-T12 Assemble C# candidate bundles and the staged sandbox runner

Status: Complete (2026-08-27).

Depends on: `CSHARP-02-T11`.

Exit gate: a staged installed tree launches only exact registered-candidate C#
bytes under the frozen isolation contract; the active registry is unchanged.

Completion evidence: the source-only staging directory contains canonical
successor candidate, registry, and semantic-registry transports. The pinned
assembler reproduces an 18-file frontend inventory and a 373-file toolchain
inventory twice, including the exact .NET 10.0.11 runtime, 167-file reference
projection, selected native closure, and reviewed compatibility shim. The
private runner resolves only beside the executing installed fixture, snapshots
every described byte through no-follow descriptors, launches the reviewed
208,384-byte frontend with the exact argv and 18-entry closed environment,
and validates its successor envelope and emitted release identity. Two
isolated runs emit byte-identical reports. The active registry remains
Go/Rust-only.

### CSHARP-02-T13 Migrate the Go producer to successor artifacts in staging

Status: Complete (2026-08-28).

Depends on: `CSHARP-02-T12`.

Exit gate: a staged `go2vir` emits only successor source artifacts with
unchanged Go semantics, while the active Go binary and fixtures remain
unchanged.

Completion evidence: a seven-source exact-hash overlay builds the staging-only
frontend twice with the frozen offline Go 1.25.0 image and produces the exact
static candidate binary and one-tuple successor Go bundle projection. The
complete 13-positive/eight-negative corpus regenerates deterministically with
only successor envelopes, semantic contexts, selection envelopes, VIR,
contracts, source maps, manifests, and hash domains. The checked semantic-
difference report has zero source-behavior, required-check, VC-input-intent,
and diagnostic changes. Shared artifact, protocol, and release validators
accept the staged family and reject predecessor or crossed input; active
validators reject the staged family. No active Go source, binary, fixture,
release tuple, registry, or production route changed.

### CSHARP-02-T14 Migrate the Rust producer and private driver in staging

Status: Complete (2026-08-28).

Depends on: `CSHARP-02-T13`.

Exit gate: staged `rust2vir` and its private driver emit only successor
artifacts with unchanged Rust semantics, while active Rust bytes remain
unchanged and no binary accepts both public VIR schemas.

Completion evidence: an eleven-source exact-hash overlay builds the staged
frontend and subordinate driver twice from the frozen offline nightly and
produces an exact two-target successor Rust bundle projection. The complete
13-positive/73-negative corpus regenerates deterministically with successor
public artifacts plus v1 private request, result, raw-lowering, and raw-map
identities. Its checked report has zero source-behavior, required-check,
target, source-map, manifest-input-intent, and diagnostic changes. Shared
artifact, protocol, and release validators accept the staged family and reject
predecessor or crossed input; the staging-only subordinate parser rejects all
v0 private identities. No active Rust source, binary, fixture, release tuple,
registry, or production route changed.

### CSHARP-02-T15 Stage successor VC and skeleton integration

Status: Complete (2026-08-28).

Depends on: `CSHARP-02-T14`.

Exit gate: all three staged profiles generate exact successor VC/skeleton
bytes and required checks with unchanged Go/Rust obligation semantics.

Completion evidence: an explicitly injected `mpk-vc` staging path binds the
revision-2 semantic context, successor VIR and manifest, and profile-owned VC
contract into canonical `mpk.vc.v2` and `mpk.vc.cert_skeleton.v2` bytes. It
uses only `MPK-VC-2.0`, regenerates all members, groups, dependencies, and
declarations, preserves active Go/Rust obligations, and routes C# required
checks through the unchanged checked Bool/BV foundation. Active and successor
VC families reject one another, and no released route discovers this API.

### CSHARP-02-T16 Stage policy, evidence, and certificate integration

Status: Complete (2026-08-28).

Depends on: `CSHARP-02-T15`.

Exit gate: all three staged profiles reach policy/evidence and unchanged
Certificate v0, including a representative C# certificate accepted by both
checkers from identical bytes.

Completion evidence: hidden, explicitly injected `mpk-cli` staging paths
regenerate canonical `mpk.policy.scan.v2` from the validated frontend graph
and `mpk.policy.evidence.v2` from the complete successor VC/skeleton graph.
They bind exact profile-owned strategy/checker/axiom/recipe contracts,
finalize only the VC hash and self-hash in `mpk.source_manifest.v1`, and apply
`mpk.program_certificate.alpha.v1` through the unchanged Certificate v0
assembler. Representative Go/Rust verdicts remain unchanged; the canonical
C# candidate and its registered zero-axiom foundations pass both source-free
checkers from identical bytes. Exact regeneration rejects old/crossed or
mutated policy artifacts, and no helper/compiler output is promoted to
trusted evidence or routed through the released policy CLI.

### CSHARP-02-T17 Stage successor AI explanation integration

Status: Complete (2026-08-28).

Depends on: `CSHARP-02-T16`.

Exit gate: closed profile-aware AI explanation staging accepts only validated
successor evidence/context, applies the C# redaction contract, and cannot
affect proof acceptance.

Completion evidence: the hidden successor explainer validates the exact
profile-owned `ai` envelope against complete successor policy evidence,
projects deterministic minimal-v1 requests for Go, Rust, and C#, and emits
canonical v2 request/report artifacts without adding a provider, credential,
network, CLI, or released-schema route. The C# fixture proves that source and
contract contents and paths, compiler prose, selection values, hashes, and
unsanitized evidence identifiers do not enter the provider request. Provider
responses remain prose-only untrusted helper analysis; local aliases restore
the original evidence IDs and immutable statuses, while evidence and
Certificate v0 outcomes remain byte-identical. Old, crossed, widened,
noncanonical, metadata-injected, unknown-alias, and bidi inputs reject.

### CSHARP-02-T18 Stage successor AI API integration

Status: Complete (2026-08-28).

Depends on: `CSHARP-02-T17`.

Exit gate: successor API staging binds sessions to exact semantic contexts and
accepts no old, crossed, or proof-bypassing helper input.

Completion evidence: the hidden `mpk.ai.api.v2` service has its own exact
profile-gated route registry while leaving the released v1 router unchanged.
Session creation and every VIR/VC request repeat one registry-validated
`SemanticContext` and `SelectionEnvelope`; the immutable artifact store accepts
only validated successor VIR, source-map, frontend-manifest, and compiled VC
contract capabilities. Go, Rust, and C# produce deterministic canonical
responses. Old, successor-at-v1, crossed-profile, crossed-selection, stale,
unknown, and noncanonical operations reject without state mutation. Candidate
results remain `helper_only`, and a valid helper result leaves both accepted
and rejected canonical Certificate v0 checker outcomes byte-for-byte
unchanged.

### CSHARP-02-T19 Complete cross-profile hardening and release rehearsal

Status: Complete (2026-08-28).

Depends on: `CSHARP-02-T18`.

Exit gate: all corpus, differential, fuzz, isolation, determinism, upgrade,
checker, axiom, and staged installed-release gates pass with an empty findings
ledger.

Completion evidence: one aggregate executes every frozen C# profile field,
all 12 upgrade cases, predecessor/successor and three-profile cross-rejection,
exact-runtime differential cases, and five bounded hash-bound fuzz targets.
All canonical C# artifacts and staged Go/Rust/C# bundles rebuild and replay
byte-identically. The offline two-pass installed-release gate confirms no
plugin, credential, host-path, network, checker, Certificate v0, or axiom-
category expansion, and the final review ledger has zero findings.

### CSHARP-02-T20 Perform the atomic successor cutover and C# release

Status: Complete (2026-08-30).

Depends on: `CSHARP-02-T19` and transitively every earlier CSHARP-02 task.

Exit gate: one release activates revision 2 and successor schemas for
Go/Rust/C#, removes every old public helper path, passes both checkers and the
installed release gates, and makes `JAVA-03` eligible.

Completion evidence: the active semantic registry is the frozen revision-2
root, the active successor bundle registry contains three frontend bundles,
three toolchain bundles, and four exact tuples, and C# exposes all nine frozen
compiled-profile contracts. `crates/mpk-cli/tests/successor_atomic_cutover.rs`
materializes one installed image beside both registries, executes Go, Rust,
and C# only through registered descriptor-relative bundles, rejects old and
crossed identities, and detects bundle tampering. The executable staging tree
is removed; the T19 review and Go/Rust semantic-difference reports are retained
only under `develop/migrations/archive/`. The two-pass offline installed gate,
both source-free checker agreement, unchanged Certificate v0 bytes, unchanged
four-category axiom inventory, and final zero-finding review close Gate D and
make `JAVA-03` eligible.

## JAVA-03: Implement and release Java

Entry gate: `CSHARP-02` complete.

Design: `07_java_frontend_design.md` (2026-08-31). Java SE 25, a pinned Temurin
JDK, public compiler/tree/language-model APIs, and a scalar subset of
field-free interfaces with pure static methods are selected. Java owns its
wrapping/division/shift check rules; Go/Rust/C# profile entries remain
byte-identical in the now-installed registry revision 3.

Execution follows the ten serial tasks in that design's section 11 and
`java-03-implementation-traceability-ledger.md`. T01-T10 are complete: T08
integrated VC/policy/evidence/AI/API, and T09 hardened and rehearsed the
release. T10 installed the four-language release metadata and route. No public
Java tuple was activated before T10. The native T10 receipt is accepted;
CSHARP-03-T01-W01 through W10 completed the practical-profile entry, evidence,
freeze, and normative publication work. T02-W01/W02/W03/W04/W05/W06/W07/W08/W09
and T03-W01/W02/W03/W04/W05/W06/W07/W08 are complete, T03-W09 is ready, and later phases remain
serially blocked.

T01 completion evidence: `JAVA_PROFILE_V0.md`, Java and revision-3 vectors,
manifest entries, the traceability ledger, exact Temurin 25.0.4.1+1/JDK/native
inventory, fixed JVM options/budgets, and recorded public-API/host probes. All
34 semantic rows are classified. The probes establish actual Linux amd64
compiler/JVM compatibility under CPU emulation; they do not establish every
native Linux cgroup/syscall/resource-failure release condition. T07 later
accepted its installed native gate and T09 later completed release rehearsal;
T10 retained activation ownership. At T01 completion, the production registry
remained revision 2 and no Java source-processing or active route was added.

T02 completion evidence: `java-tools/java2vir`, the offline
`scripts/build-java-frontend.sh` owner, exact source/recipe/class/JAR records
in `release/build-inputs/java/`, and `java_build_inputs.rs` tests. Two fresh
offline containers produce identical candidate bytes. At T02 completion, the
packaged entrypoint only supported its version check. Source parsing and VIR generation were
pending at T02 and are now implemented internally by T04 and T06, respectively.
See `java-tools/README.md` for provisioning and the explicit local build test.

T03 completion evidence: compiled revision-3/Java identities, exact parameters,
selection and nine payload validators, independent Java VIR/VC/map/manifest
admission, and `java_profile_vectors.rs` mutation tests. Revision-2 entry bytes
and the installed four tuples are unchanged. CLI and API regressions confirm
that Java cannot open a public source route or mutate session state. T04 owns
capture, public javac adaptation and bounded diagnostics; T08 still owns the
complete policy/evidence/certificate/AI/API integration.

T04 completion evidence: descriptor-based Java capture support, strict UTF-8
and original byte positions, fresh parse/analyze-only javac sessions, closed
application dependency/output paths, raw pre/post tree inventories and bounded
fixed-message failure transport. `java_frontend_vectors.rs` independently
validates diagnostics and runs the private pinned-JDK conformance harness.
T05 now runs raw subset gates before accepted-subtree transformation checks;
T06 owns successful artifacts and T07 owns installed runtime isolation. The
packaged main entry was version-only at T04 completion. T07 connects that private
entry to the candidate runner; the active release remains unchanged.

T05 completion evidence: raw declaration/name/statement/expression gates,
inert interfaces, explicit 255-unit signatures, typed bindings and conversions,
conservative acyclic source-call closure with deterministic callee-first order,
and strict JSON sidecar coverage, typing, normalization and hashes. The private
`JavaAdmission` pipeline rejects all unaccounted selected content and only
returns an internal model. `java_subset.rs` and `java_contracts.rs` execute the
frozen source/contract families, all 34 semantic rows and 35 conversion rules,
exact-boundary/plus-one consumers and cross-file JSON/attachment precedence.
T06 consumes these immutable source and contract models.

T06 completion evidence: acyclic statement/short-circuit CFGs, explicit live
value block arguments, callee-first functions and canonical BFS/parameter/
instruction IDs. Java wrapping/division/shift/conversion rules have exact
checks and complete original UTF-8 origins. Bounded canonical serializers
assemble VIR, source map, frontend manifest and one LF-framed success response
only after all checks pass. Private pinned-JDK tests execute the 49 accepted
cases, 27 operation mappings, six symbolic CFGs, seven map vectors and all
nine lowering/emission counter boundaries. The Rust owning tests independently
import real artifacts and reject rehashed semantic and binding mutations.
The main entrypoint, active registry and four installed tuples remain unchanged.

T07 completion evidence: private registered frontend/toolchain candidates,
exact revision-3 context, no-follow bundle snapshots and the fixed JVM launcher;
native x86-64 Linux candidate assembly and image/native gates passed on
2026-09-01. The native receipt binds registry and runner bytes, three complete
lowering outputs, hostile-environment equality, six real resource faults, ten
release-before-source mutations, and JVM-attributed seccomp/socket/clone
evidence. Full lowering requests retain the frozen 120-second limit; a
source-free registered identity probe records syscall evidence without charging
ptrace transport overhead to those requests. At T07 completion, T08 was next
and T09/T10 release rehearsal and activation were still pending; revision 2
remains active.

T08 completion evidence: the inactive Java route generated Java-owned VC,
policy/evidence, same-byte dual-checker Certificate v0 results, sanitized AI
requests and API responses while preserving full revision-3 context and adding
no Java axiom or public strategy route. T09 then executed the complete
conformance, 102-case JDK differential, 160 fixed-seed fuzz, upgrade and
private release-rehearsal owners, froze candidate v2 without mutating v1, and
recorded its native release receipt.

T10 implementation evidence: the sole checked-in semantic registry is
revision 3 and the sole release registry contains exactly four frontends, four
toolchains and five Go/Rust/C#/Java tuples. CLI/API installed execution, all
nine Java compiled contracts, old/crossed/partial rejection, regenerated
context-bound corpora, active docs and the Java payment-policy example are
owned by `java_activation.rs` and `successor_atomic_cutover.rs`. Executable
Java staging routes were removed. `scripts/check-java-frontend.sh` is the sole
offline two-pass release gate and `check-all.sh` delegates to it. Certificate
v0, both source-free checker inputs and the four axiom categories are
unchanged.

The T10 native receipt was accepted on 2026-09-03. On native x86-64 Linux with
kernel `6.8.0-134-generic`, `./scripts/check-java-frontend.sh` returned exit 0
after both installed-release passes, including every namespace, cgroup,
resource-failure, mutation, syscall-trace, checker and report check. The
canonical receipt is
`develop/migrations/archive/java-03-t10-native-receipt.json`; it unblocked
CSHARP-03 T01, which has now completed.

Exit gate: Java passes the common definition of done without reinterpreting C#,
Go, or Rust semantics, including the local offline two-pass installed gate,
same-byte dual-checker certificate corpus and zero open review findings.

## CSHARP-03: Design, implement, and release the practical C# profile

Source design: `develop/docs/08_csharp_practical_subset_design.md`.

Entry gate: `JAVA-03` is complete, including T10 atomic activation and its
local native Linux release evidence. CSHARP-03 T01 used that authority to
publish the normative but inactive freeze package; publication does not permit
a production route or partial successor activation.

Only after entry, freeze and implement one new immutable practical profile for
expression bodies, `var`, ordinary namespace `using`, immutable domain value
types and construction, bounded arrays and internal monomorphic sequence/map/
set semantics, strings and exact boundary codecs, floating-point and decimal
values, nullable values, application-owned closed outcomes, explicit missing/
null/value boundary presence, dates, times, durations, instants, GUIDs,
application-owned money, structural equality/order, structured loops, switch/
patterns, and closed exceptions. Reject every user-defined generic and
arbitrary constructed framework type except the exact nullable-value `T?`
source form; specialize only enumerated MPK-owned internal semantic templates
from one registered, content-hash-pinned foundation bundle before VIR emission.
Application source and runtime must not import, reference, implement, generate,
link, or deploy an MPK API or assembly. Iterators and async source are negative
conformance scope, not requested implementation capabilities.

Freeze the canonical serialization boundary as an MPK verification-overlay
transport rather than an application runtime protocol, and freeze pure
business state transitions with optional full-snapshot idempotency without
claiming infrastructure correctness. Keep `mpk.csharp.scalar.v0` immutable.
The required successor registry and affected-artifact migration follow the
source design, are atomic across all active profiles, keep Certificate v0 and
both checker acceptance rules unchanged, and retain empty proof-node/theory
tables with a recomputed total axiom count of zero.

Execution follows the serial stages and acceptance criteria in the source
design. The reviewed detailed
`08_csharp_practical_subset_design-todo.md` is the current subordinate
execution decomposition. The accepted native `JAVA-03-T10` x86-64 Linux
release receipt satisfied its phase entry gate.
`CSHARP-03-T01-W01` through W10 completed the entry audit, probes, foundation,
private freeze, and complete publication/review. The normative inactive
outputs are `CSHARP_PRACTICAL_PROFILE_V1.md`,
`CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md`, and the manifested 700-vector
package. `CSHARP-03-T02-W01/W02/W03/W04/W05/W06/W07/W08/W09` and
`CSHARP-03-T03-W01/W02/W03/W04/W05/W06/W07/W08` are complete, `CSHARP-03-T03-W09` is ready, and each
later `Wnn` item remains blocked until
its serial predecessor and task-local gate are complete.
Each requested capability must have positive, rejection,
boundary, differential, determinism, and compiler-upgrade evidence. Actual C#
source, not manually assembled VIR, must pass the installed frontend and the
same-byte dual-checker path. Local native Linux and fast gates plus a
zero-finding review are required before activation.

Exit gate: the practical C# profile is atomically active, every predecessor
profile remains semantically equivalent, all three general-facing business
samples are runnable, and the release receipt records all identities, hashes,
native-gate evidence, checker agreement, axiom inventory, and zero open
findings.

## DART-04: Implement and release Dart

Entry gate: `CSHARP-03` complete.

Only after the entry gate, freeze one Dart execution target and exact integer,
null, dispatch, failure, SDK/analyzer, package-config, platform-library, and
analysis-option semantics. If no exact scalar subset is defensible, this phase
stops rather than emitting VIR.

Exit gate: Dart passes the common definition of done under one explicit target.

## TYPESCRIPT-05: Implement and release TypeScript

Entry gate: `DART-04` complete.

Only after the entry gate, freeze the JavaScript runtime target and numeric
model, TypeScript compiler, standard declaration library, module resolution,
package graph, and rejected dynamic behaviors. Static TypeScript types must
not be treated as runtime semantics.

Exit gate: TypeScript passes the common definition of done against the pinned
JavaScript runtime behavior.

## PYTHON-06: Implement and release Python

Entry gate: `TYPESCRIPT-05` complete.

Only after the entry gate, freeze the CPython version, AST boundary, MPK-owned
closed name/type/subset analysis, integer/operation semantics, standard-library
input set, import/builtin policy, and all rejected dynamic behavior.

Exit gate: Python passes the common definition of done without trusting type
annotations or runtime introspection.

## Final handoff checklist

- Rust v0 completed `RUST-07-T05` before `MLANG-00` began.
- Every feasibility, specification, and production phase began only after its
  immediate predecessor's exit gate; proposal-only design records granted no
  implementation authority and no production phases overlapped.
- Language/profile production phases were admitted in the recorded order, or
  an explicit governance amendment changed the future sequence before the
  affected production phase began.
- Every active language has one or more globally unique immutable profiles;
  every profile has complete vectors and every request selects exactly one.
- No mixed-language VIR, runtime plugin, raw executable path, or new language-
  axiom category exists.
- Certificate v0 and both source-free checker acceptance rules remain
  unchanged.
- All active frontend bundles, compilers, targets, manifests, and evidence are
  reproducible but remain labeled untrusted.
