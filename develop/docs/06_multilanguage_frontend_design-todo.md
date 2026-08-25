# Post-Rust Multi-Language Frontend Expansion Todo

Source design: `develop/docs/06_multilanguage_frontend_design.md`

Status: Gate C and `MLANG-01` are complete after the completed Rust and
`MLANG-00` gates. Gate D is active at its non-production planning boundary:
`CSHARP-02-T01` is complete and `CSHARP-02-T02` is next. No C# production
route or later-language phase is active.

## Scope and sequencing

This plan queues C#, Java, Dart, TypeScript, and Python after the complete
Go/VIR/Rust critical path. It defines one flow; none of its milestones overlap
the Rust program or another language milestone.

```text
VIR-00 -> VIR-01 -> GO-VIR-02 -> RUST-03 -> RUST-04 -> RUST-05
  -> RUST-06 -> RUST-07-T05
  -> MLANG-00 design and feasibility
  -> MLANG-01 successor contract freeze
  -> CSHARP-02
  -> JAVA-03
  -> DART-04
  -> TYPESCRIPT-05
  -> PYTHON-06
```

Every phase starts only after the preceding phase's exit gate. A later
language's specification and production work are both contained in its phase
and start only after the preceding language's final release gate.

## Common execution rules

1. Do not add placeholder future-language values to a frozen VIR, frontend,
   manifest, release, policy, evidence, or AI schema.
2. Do not start `MLANG-00` before `RUST-07-T05`, and do not start any later
   phase before its immediate predecessor completes.
3. Keep every compiler, analyzer, SDK, frontend, source, contract, IR, map,
   manifest claim, VC, policy output, and differential result untrusted.
4. Keep Certificate v0, core/checker semantics, source-free checker inputs, and
   the four axiom categories unchanged.
5. Each language owns a distinct semantic profile. An implementation never
   infers source behavior from `source_language` or reuses another language's
   profile.
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
11. Execute numbered tasks and language phases in the displayed order. Do not
    overlap tasks within `MLANG-00` or `MLANG-01`, and do not begin feasibility
    refresh or specification work for a later language while its predecessor
    is still active.
12. Execute `CSHARP-02-T01` through `CSHARP-02-T20` serially under
    `csharp-02-implementation-traceability-ledger.md`. T02 through T19 remain
    inactive staging; T20 alone may activate the successor release.

## Common definition of done for a language

Each production language phase must provide:

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

- `../specs/CSHARP_PROFILE_V0.md`, the normative but inactive exact C# scalar
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

Status: Active at the inactive implementation boundary. `CSHARP-02-T01` is
complete; `CSHARP-02-T02` is next.

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

Status: Next.

Depends on: `CSHARP-02-T01`.

Exit gate: an unregistered `csharp2vir` candidate builds twice to identical
bytes from the exact offline input closure; no parser, bundle registration, or
active route exists.

### CSHARP-02-T03 Implement the inactive semantic-profile registry core

Status: Pending.

Depends on: `CSHARP-02-T02`.

Exit gate: registry/context/profile-envelope vectors execute against private
production validation code while released inputs remain unchanged.

### CSHARP-02-T04 Stage successor source-artifact models and hash domains

Status: Pending.

Depends on: `CSHARP-02-T03`.

Exit gate: successor VIR/frontend/map/manifest artifacts validate and hash in
staging with bidirectional old/new rejection and no active producer.

### CSHARP-02-T05 Implement C# selection, path preflight, and immutable capture

Status: Pending.

Depends on: `CSHARP-02-T04`.

Exit gate: exact selection and source-transport mutations fail at their owned
phase, and successful capture never rereads an original input.

### CSHARP-02-T06 Build and validate the exact Roslyn compilation session

Status: Pending.

Depends on: `CSHARP-02-T05`.

Exit gate: the frozen public Roslyn session and reference closure match at
getter/API level, with every drift case rejected before lowering.

### CSHARP-02-T07 Enforce the C# subset, closure, purity, and initialization

Status: Pending.

Depends on: `CSHARP-02-T06`.

Exit gate: every omitted construct rejects before VIR and every accepted
method belongs to one deterministic pure acyclic closure.

### CSHARP-02-T08 Implement typed C# contracts and attachment

Status: Pending.

Depends on: `CSHARP-02-T07`.

Exit gate: every closure method has one exact normalized/hash-bound contract,
and all malformed, missing, duplicate, or unused sidecars reject.

### CSHARP-02-T09 Lower scalar C# operations and required checks

Status: Pending.

Depends on: `CSHARP-02-T08`.

Exit gate: scalar/control/conversion lowering is deterministic and every
required check is complete, unique, and canonically ordered.

### CSHARP-02-T10 Lower static calls and emit stable VIR, maps, and manifests

Status: Pending.

Depends on: `CSHARP-02-T09`.

Exit gate: every accepted source emits complete staged successor artifacts
with faithful origins, stable IDs, call dependencies, and exact hashes.

### CSHARP-02-T11 Close C# frontend diagnostics, limits, and source-case vectors

Status: Pending.

Depends on: `CSHARP-02-T10`.

Exit gate: all frontend-owned accepted/rejected, diagnostic, precedence,
limit, and hash vectors execute against candidate code with exact artifact-
free non-success behavior and no active C# route.

### CSHARP-02-T12 Assemble C# candidate bundles and the staged sandbox runner

Status: Pending.

Depends on: `CSHARP-02-T11`.

Exit gate: a staged installed tree launches only exact registered-candidate C#
bytes under the frozen isolation contract; the active registry is unchanged.

### CSHARP-02-T13 Migrate the Go producer to successor artifacts in staging

Status: Pending.

Depends on: `CSHARP-02-T12`.

Exit gate: a staged `go2vir` emits only successor source artifacts with
unchanged Go semantics, while the active Go binary and fixtures remain
unchanged.

### CSHARP-02-T14 Migrate the Rust producer and private driver in staging

Status: Pending.

Depends on: `CSHARP-02-T13`.

Exit gate: staged `rust2vir` and its private driver emit only successor
artifacts with unchanged Rust semantics, while active Rust bytes remain
unchanged and no binary accepts both public VIR schemas.

### CSHARP-02-T15 Stage successor VC and skeleton integration

Status: Pending.

Depends on: `CSHARP-02-T14`.

Exit gate: all three staged profiles generate exact successor VC/skeleton
bytes and required checks with unchanged Go/Rust obligation semantics.

### CSHARP-02-T16 Stage policy, evidence, and certificate integration

Status: Pending.

Depends on: `CSHARP-02-T15`.

Exit gate: all three staged profiles reach policy/evidence and unchanged
Certificate v0, including a representative C# certificate accepted by both
checkers from identical bytes.

### CSHARP-02-T17 Stage successor AI explanation integration

Status: Pending.

Depends on: `CSHARP-02-T16`.

Exit gate: closed profile-aware AI explanation staging accepts only validated
successor evidence/context, applies the C# redaction contract, and cannot
affect proof acceptance.

### CSHARP-02-T18 Stage successor AI API integration

Status: Pending.

Depends on: `CSHARP-02-T17`.

Exit gate: successor API staging binds sessions to exact semantic contexts and
accepts no old, crossed, or proof-bypassing helper input.

### CSHARP-02-T19 Complete cross-profile hardening and release rehearsal

Status: Pending.

Depends on: `CSHARP-02-T18`.

Exit gate: all corpus, differential, fuzz, isolation, determinism, upgrade,
checker, axiom, and staged installed-release gates pass with an empty findings
ledger.

### CSHARP-02-T20 Perform the atomic successor cutover and C# release

Status: Pending.

Depends on: `CSHARP-02-T19` and transitively every earlier CSHARP-02 task.

Exit gate: one release activates revision 2 and successor schemas for
Go/Rust/C#, removes every old public helper path, passes both checkers and the
installed release gates, and makes `JAVA-03` eligible.

## JAVA-03: Implement and release Java

Entry gate: `CSHARP-02` complete.

Only after the entry gate, freeze the Java subset/profile/toolchain package
using the pinned JDK Compiler Tree and language-model APIs, exact `--release`
target, system modules, source/class/module paths, and an empty annotation-
processor set. Then implement the same common deliverables and gates as C#.

Exit gate: Java passes the common definition of done without reinterpreting C#,
Go, or Rust semantics.

## DART-04: Implement and release Dart

Entry gate: `JAVA-03` complete.

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
- Every design, specification, and production phase began only after its
  immediate predecessor's exit gate; no multi-language phases overlapped.
- Languages were admitted in the recorded order, or an explicit governance
  amendment changed the future sequence before the affected phase began.
- Every active language has one distinct semantic profile and complete vectors.
- No mixed-language VIR, runtime plugin, raw executable path, or new language-
  axiom category exists.
- Certificate v0 and both source-free checker acceptance rules remain
  unchanged.
- All active frontend bundles, compilers, targets, manifests, and evidence are
  reproducible but remain labeled untrusted.
