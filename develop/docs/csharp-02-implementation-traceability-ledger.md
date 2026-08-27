# CSHARP-02 Implementation Decomposition and Traceability Ledger

Status: `CSHARP-02-T01` complete (2026-08-25), `CSHARP-02-T02` through
`CSHARP-02-T03` complete (2026-08-26), and `CSHARP-02-T04` through
`CSHARP-02-T11` complete (2026-08-27). `CSHARP-02-T12` is the next task.
Canonical C# success and non-success envelopes now exist only within the
inactive candidate and its test harness. No production C# frontend, released
successor route, installed registry root, release tuple, policy route,
evidence route, or AI route is active.

This document is the non-normative execution plan for implementing the frozen
C# package. `CSHARP_PROFILE_V0.md` and
`SEMANTIC_PROFILE_REGISTRY_V1.md` remain normative and take precedence over
this ledger. This plan assigns implementation work; it does not change a
semantic value, hash, vector, schema, or activation rule.

## 1. Scope and authority

The implementation consumes these immutable inputs:

- `develop/specs/CSHARP_PROFILE_V0.md`;
- `develop/specs/SEMANTIC_PROFILE_REGISTRY_V1.md`;
- `develop/specs/vectors/csharp-profile-v0.json`;
- `develop/specs/vectors/semantic-profile-registry-v1.json`;
- `develop/specs/vectors/semantic-profile-registry-v2.json`; and
- the active Go/Rust implementation and release gates at commit `d93bbbf`.

The decomposition obeys these rules:

1. Tasks execute serially in ID order. A task starts only after its declared
   dependency has a clean review and committed result.
2. T02 through T19 may add candidate code and test-only staging, but the
   current Go/Rust schemas remain the sole production path.
3. No intermediate task installs revision 2, registers a C# release tuple,
   exposes a successor parser through a public command or API, or accepts both
   old and successor helper schemas in one production binary.
4. T20 is the sole owner of production activation. It switches all producers,
   consumers, registries, bundles, fixtures, routes, and documentation in one
   release change.
5. Each frozen specification section, vector field group, semantic row,
   compiled-profile contract, and successor artifact has one primary task
   owner below. A downstream task may consume an output but may not reinterpret
   it.
6. A discovered ambiguity or mismatch stops the owning task with a recorded
   blocker. Implementations do not infer a default or silently edit a frozen
   input.
7. Certificate v0, both source-free checker inputs, the four axiom categories,
   and the prohibition on mixed-language VIR and FFI remain unchanged.
8. Java and later-language work remains blocked until T20 completes.

Out of scope for T01 are production code, build inputs, generated artifacts,
schema/vector changes, release registry changes, and toolchain provisioning.

## 2. Ordered task graph and current status

```text
MLANG-01-T03
  -> CSHARP-02-T01
  -> CSHARP-02-T02
  -> CSHARP-02-T03
  -> CSHARP-02-T04
  -> CSHARP-02-T05
  -> CSHARP-02-T06
  -> CSHARP-02-T07
  -> CSHARP-02-T08
  -> CSHARP-02-T09
  -> CSHARP-02-T10
  -> CSHARP-02-T11
  -> CSHARP-02-T12
  -> CSHARP-02-T13
  -> CSHARP-02-T14
  -> CSHARP-02-T15
  -> CSHARP-02-T16
  -> CSHARP-02-T17
  -> CSHARP-02-T18
  -> CSHARP-02-T19
  -> CSHARP-02-T20
  -> JAVA-03
```

| Task | Status | Bounded outcome |
| --- | --- | --- |
| `CSHARP-02-T01` | complete | implementation decomposition and closed traceability ledger |
| `CSHARP-02-T02` | complete | pinned, reproducible, unregistered `csharp2vir` project and build closure |
| `CSHARP-02-T03` | complete | inactive semantic-registry/context implementation |
| `CSHARP-02-T04` | complete | inactive successor source-artifact models and hash domains |
| `CSHARP-02-T05` | complete | C# selection, structural preflight, and immutable capture |
| `CSHARP-02-T06` | complete | exact public Roslyn parse/compilation session |
| `CSHARP-02-T07` | complete | declaration/subset/closure/purity/initialization gate |
| `CSHARP-02-T08` | complete | typed C# contract parsing, attachment, normalization, and hashing |
| `CSHARP-02-T09` | complete | scalar/control/conversion lowering and exact required checks |
| `CSHARP-02-T10` | complete | static calls, stable IDs, maps, manifests, and successful envelope emission |
| `CSHARP-02-T11` | complete | frontend diagnostics, limits, and source-case vector executor |
| `CSHARP-02-T12` | ready | deterministic C# candidate bundles and staged sandbox runner |
| `CSHARP-02-T13` | blocked on T12 | staged Go producer migration to successor artifacts |
| `CSHARP-02-T14` | blocked on T13 | staged Rust producer/private-driver migration to successor artifacts |
| `CSHARP-02-T15` | blocked on T14 | staged successor VC and skeleton integration |
| `CSHARP-02-T16` | blocked on T15 | staged policy, evidence, and program-certificate integration |
| `CSHARP-02-T17` | blocked on T16 | staged AI explanation integration |
| `CSHARP-02-T18` | blocked on T17 | staged AI API integration |
| `CSHARP-02-T19` | blocked on T18 | complete cross-profile hardening corpus and release rehearsal |
| `CSHARP-02-T20` | blocked on T19 | atomic successor cutover and C# production release |

The status tokens above are planning state only. They are not registry or
release status values.

## 3. Inactive staging contract

Intermediate implementation uses one fail-closed staging model:

- `csharp-tools/csharp2vir/` may contain reviewed candidate source after T02,
  but no active command discovers or launches it by path.
- `release/build-inputs/csharp/` may contain reviewed immutable build-input
  descriptors. A host cache under `release/build-input-cache/csharp/` is never
  a tracked or trusted artifact and no check command downloads into it.
- generated successor fixtures and whole-release candidates live only under
  `develop/migrations/csharp-02-staging/` until T20. No production resolver
  searches that directory.
- shared successor types added before T20 remain private to tests or an
  explicitly injected candidate harness. Released CLI/API routes continue to
  expose only the active Go/Rust schemas.
- candidate bundle descriptors are resolved only from an injected staging
  root. T12 does not write them to the active
  `release/bundles/bundle-registry.json`.
- T20 regenerates every active artifact and requires byte equality with the
  reviewed staging output before installing it. It removes the staging gates
  and old public helper-schema parsers in the same change.

An intermediate task fails if a production test can select C#, revision 2, a
successor schema, or a staging root without the private test harness.

## 4. Task contracts

### CSHARP-02-T01 Create the C# implementation decomposition and traceability ledger

Depends on: `MLANG-01-T03`.

Owns:

- this ordered DAG;
- one primary implementation owner for every frozen input below;
- the inactive staging and final activation boundaries; and
- routing updates in the multi-language design, todo, and developer index.

Exit gate:

- all T02 through T20 tasks are bounded, dependency ordered, and test-owned;
- all three governing vector containers are exhaustively assigned;
- no normative or production artifact changes; and
- the T01 review ledger is empty.

### CSHARP-02-T02 Create the isolated pinned C# frontend project

Depends on: `CSHARP-02-T01`.

Owns:

- the `csharp-tools/csharp2vir/` project and deterministic build entrypoint;
- exact SDK, runtime, Roslyn package/assembly, analyzer-metadata, reference
  pack, native-host, archive extraction, file-mode, and inventory validation;
- an offline, no-restore, no-network build using only frozen bytes;
- reproducible `csharp2vir.dll`, dependency, runtime-configuration, and notice
  candidate inventories; and
- two-clean-build byte equality and hostile ambient build tests.

It does not parse source, emit a frontend envelope, register a bundle, or add
an active release tuple. The primary implementation test owner is
`crates/mpk-cli/tests/csharp_build_inputs.rs`; the frozen specification owner
remains unchanged.

Exit gate: the project builds twice to identical reviewed bytes from the exact
input closure, and changing any archive, selected assembly, reference, mode,
or build option fails before candidate publication.

Completion evidence (2026-08-26):

- `release/build-inputs/csharp/build-inputs.json` binds the frozen vector,
  six project inputs, direct pinned-compiler recipe, 13 notice sources, and
  unregistered candidate inventory;
- the check-only build validates exact archive hashes, SDK/runtime file counts
  and modes, package graphs, Roslyn assemblies, and the 167-file reference
  projection before compiling;
- two independent fresh extractions build and launch in separate network/user
  namespaces with a closed environment, no restore, and byte-equal candidate
  trees and deterministic archives;
- `csharp2vir.dll` is 5,120 bytes with SHA-256
  `76aadd20282a655783089cf8148ef3fc627b73f26da7fcc48a653c844ca63b26`;
  only the exact `--version` probe succeeds; and
- `crates/mpk-cli/tests/csharp_build_inputs.rs` closes descriptor, project,
  recipe, inventory, hostile-ambient, ignored-cache, and no-active-route
  behavior. The raw archive cache is ignored and untracked.

### CSHARP-02-T03 Implement the inactive semantic-profile registry core

Depends on: `CSHARP-02-T02`.

Owns:

- strict registry transport, root/entry hashing, revision, ordering, limits,
  status precedence, and installed-identity assertions;
- `SemanticContext`, parameter, `SelectionEnvelope`, and compiled-profile
  envelope models;
- compiled Go, Rust, and C# parameter/selection/profile-contract dispatch with
  no executable callback, plugin, path, URI, or fallback;
- exact revision-1 and revision-2 lookup, append-only, and unknown/later-
  language rejection; and
- runtime execution of registry transport/hash/context/profile/limit vectors
  and revision-2 hash/append-only vectors.

The implementation is private/test-injected until T20. The primary test owner
is `crates/mpk-vc/tests/semantic_profile_registry_runtime.rs`; T03 adds that
runtime owner to both registry-vector manifest entries without changing either
frozen vector or existing specification owner.

Exit gate: every registry/context/profile-envelope vector reaches production
validation code in tests, while the released binary still rejects every
successor identity.

Completion evidence (2026-08-26):

- `crates/mpk-vc/src/semantic_profile_registry.rs` implements the closed
  revision-1/revision-2 transport, shape, scalar, limit, order, hash,
  contract, invariant, identity, context, selection, profile-envelope,
  linkage, status, and append-only validation phases;
- all profile, parameter, selection, and contract dispatch is a finite Rust
  enum/table with no callback, executable, path, URI, plugin, or fallback;
- `crates/mpk-vc/tests/semantic_profile_registry_runtime.rs` executes all 87
  T03-owned revision-1 cases, both revision-2 hashes, all eight append-only
  cases, exact C# parameters, and all nine C# compiled-profile payloads;
- the runtime owner is appended to both vector-manifest records while the
  frozen vector bytes and their existing `owner_test` values remain exact;
  and
- current `SemanticProfile` and VIR v0 imports continue to reject C# and
  successor identities; no active consumer calls the injected registry core.

### CSHARP-02-T04 Stage successor source-artifact models and hash domains

Depends on: `CSHARP-02-T03`.

Owns inactive implementations of:

- `MPK-CONTRACT-1.0` and `mpk.vir.v1` / `MPK-VIR-1.0`;
- `mpk.frontend.cli.v1`;
- `mpk.source_map.v1` / `MPK-SOURCE-MAP-1.0`; and
- `mpk.source_manifest.v1` / `MPK-SOURCE-MANIFEST-1.0`.

It replaces flat language/profile fields with exact semantic contexts and
untagged selections with `SelectionEnvelope` only in staged parsers. It adds
strict cross-artifact linkage and bidirectional old/new rejection without an
adapter. The primary test owners are
`crates/mpk-vc/tests/successor_source_artifacts.rs` and
`crates/mpk-cli/tests/successor_frontend_protocol.rs`.

Exit gate: staged source artifacts round-trip canonically under the new hash
domains, all member mismatches fail, and no active producer or consumer has
changed schema.

Completion evidence (2026-08-27):

- `crates/mpk-vc/src/successor_source_artifacts.rs` seals registry-injected
  successor VIR/contract, source-map, and source-manifest models and computes
  `MPK-CONTRACT-1.0`, `MPK-VIR-1.0`, `MPK-SOURCE-MAP-1.0`, and
  `MPK-SOURCE-MANIFEST-1.0` without a compatibility representation;
- `crates/mpk-cli/src/successor_frontend_protocol.rs` validates only explicit
  `mpk.frontend.cli.v1` staging calls with exact `SemanticContext`,
  `SelectionEnvelope`, canonical LF framing, status/exit agreement, and
  linked validated artifacts;
- the two primary implementation tests perform canonical round trips, execute
  the four T04-owned hash-domain rows and frozen normalized C# contract hash,
  mutate every common context/link member, and prove bidirectional v0/v1
  rejection; and
- the source-artifact owner is appended to the registry-vector manifest while
  frozen vector bytes, active Go/Rust producers and consumers, registered
  bundles, release tuples, and public commands remain unchanged.

### CSHARP-02-T05 Implement C# selection, path preflight, and immutable capture

Depends on: `CSHARP-02-T04`.

Owns:

- the exact private C# CLI argument grammar and required release assertions;
- selection-envelope validation and canonical method-ID parsing;
- canonical selection serialization and `MPK-CSHARP-SELECTION-0.1` hash;
- descriptor-relative path/type/link/collision/inventory preflight;
- one-read immutable source and contract capture with all file/snapshot limits;
- strict C# source transport before Roslyn starts; and
- deterministic capture/source-phase issues with no partial artifact.

It does not invoke Roslyn or inspect C# syntax. The primary test owner is
`crates/mpk-cli/tests/csharp_capture.rs`.

Exit gate: every capture/source-transport mutation has the frozen phase,
status, and code, and no successful path rereads original input after capture.

Completion evidence: `csharp2vir` now accepts only the exact frozen private
`lower` argv shape and release assertions, validates and canonically hashes the
closed C# selection, inventories only selected files and implied directories,
opens regular files with Linux `O_NOFOLLOW`, rejects link/identity/type/path
collisions, and retains each source or contract byte sequence once. Source
transport consumes only those retained buffers and rejects BOM, malformed
UTF-8, NUL, CR/non-LF line endings, lone surrogates, and noncharacters before
any Roslyn API is called. The pinned offline harness executes exact selection
bytes/hash, mutation precedence, immutable-after-mutation, and inclusive
file/total/entry limit cases. Its reviewed candidate DLL is 32,256 bytes with
SHA-256 `64479185e557f455367f161c7c66fd0d61fb660cf5756c84b972721f25e7782d`.

### CSHARP-02-T06 Build and validate the exact Roslyn compilation session

Depends on: `CSHARP-02-T05`.

Owns:

- in-process loading of only the two frozen Roslyn managed assemblies;
- exact `SourceText`, parse options, syntax-tree order, metadata references,
  compilation options, and cancellation-token arguments;
- syntax then compilation diagnostic collection at their owning phases;
- public semantic-model, symbol, operation, and CFG API adapters;
- reference/session/API drift rejection with no syntax-text fallback; and
- toolchain row `M33`.

It does not decide subset admission or emit VIR. The primary test owner is
`crates/mpk-cli/tests/csharp_roslyn_session.rs`.

Exit gate: getter-level session values and public API shapes match the frozen
vector, and every changed option/reference/API case fails closed before
lowering.

Completion evidence: `csharp2vir` validates that only the two frozen Roslyn
5.6.0 assemblies are loaded, constructs exact strict-UTF-8 SHA-256 source text
and C# 14 parse options, preserves selected tree order, validates the complete
canonical reference projection, and builds the exact x64 Release compilation.
It collects syntax diagnostics before metadata references or compilation
diagnostics, treats active warnings/errors at their owning source/metadata
phase, and exposes direct public semantic-model, symbol, type, conversion,
operation, and method-body CFG adapters with `CancellationToken.None` and
`ignoreAccessibility=false`. The executable pinned harness validates every
public getter family and fail-closed option/reference/API mutation. The
reviewed candidate DLL is 48,128 bytes with SHA-256
`7ef39a41f4d11e02d3bc85cf06d351b363130260bba3edcbc55810ba09d494ad`.

### CSHARP-02-T07 Enforce the C# subset, closure, purity, and initialization

Depends on: `CSHARP-02-T06`.

Status: Complete (2026-08-27).

Owns:

- declaration, type-spelling, literal, statement, and control-flow admission;
- exact predefined symbol/operator/conversion origin checks;
- conservative all-selected-method closure, acyclicity, and source-call
  resolution;
- purity, definite-assignment, immutable-parameter, and inert-initialization
  proofs;
- operation/CFG accounting before lowering; and
- the 16 `reject_before_vir` semantic rows assigned in section 7.

It does not normalize contracts or publish a partial graph. The primary test
owner is `crates/mpk-cli/tests/csharp_subset.rs`.

Exit gate: every omitted construct family rejects at its exact earliest phase,
while accepted closure metadata is deterministic and complete.

Completion evidence: `SubsetSymbols.cs` closes namespaces, static classes,
ordinary methods, source spellings, predefined carriers, and exact source
symbol origins. `SubsetOperations.cs` closes statements, literals, operators,
conversions, calls, purity, abrupt completion, local assignment, public
operation kinds, CFG regions/branches, and reference-union accounting.
`SubsetValidator.cs` discovers the conservative multi-root source-call graph
lazily, rejects the first excess closure member before analysis retention,
rejects cycles and unrelated methods, and stores the complete graph in
deterministic callee-first order. The executable pinned harness owns all 16
T07 semantic rows, source-dead and multi-file calls, exact boundary counters,
and a real 128/129-method closure. The reviewed candidate DLL is 86,528 bytes
with SHA-256
`251508958b2754fda74349a5cd89575dbf9fd86f7f7899458cbd0f07f664d5e6`.

### CSHARP-02-T08 Implement typed C# contracts and attachment

Depends on: `CSHARP-02-T07`.

Status: Complete (2026-08-27).

Owns:

- strict sidecar JSON shape, duplicate-key rejection, typed integer grammar,
  expression typing, operator arity, limits, and stable diagnostics;
- exact one-to-one contract-to-closure attachment and unused/missing refusal;
- sidecar and normalized `VirContract` hashes, consuming the T05 selection
  hash;
- total/forbidden/no-modifies profile checks; and
- semantic row `M34`.

It adds no source-language contract syntax. The primary test owner is
`crates/mpk-cli/tests/csharp_contracts.rs`.

Exit gate: every closure method has one exact normalized contract, all contract
vectors execute against implementation code, and no raw sidecar claim becomes
proof evidence.

Completion evidence: `ContractParser.cs` owns the strict closed JSON union,
recursive duplicate rejection, exact canonical integer grammar/ranges, stable
contract diagnostics, and checked 64-clause, 1,024-node/method,
8,192-node/closure, and depth-32 counters. `ContractAttachment.cs` verifies the
T05 selection digest, refuses unused, duplicate, and missing sidecars, attaches
in deterministic closure order, and type-checks the exact successor operator
union without conversion. `ContractCanonical.cs` emits the complete frozen
semantic context and reproduces the 440-byte sidecar and 1,151-byte normalized
contract hash payloads. Raw file and canonical sidecar digests remain separate
private input identities. The primary Rust owner and pinned executable C#
harness cover M34, all diagnostics/operators/types, one-to-one attachment, the
frozen vectors, and exact boundaries. The reviewed candidate DLL is 109,056
bytes with SHA-256
`2ab139c21fd011755b9de5e1602d3f4aa883f4c28b454659e1dc185769416659`.

### CSHARP-02-T09 Lower scalar C# operations and required checks

Depends on: `CSHARP-02-T08`.

Status: Complete (2026-08-27).

Owns:

- Bool and signed/unsigned BV32/BV64 constants, locals, copies, and returns;
- left-to-right expression evaluation, short circuiting, branches, joins, and
  early returns;
- 34 non-call operation mappings, all 12 Roslyn checked-state cases, and all
  20 conversion rules;
- explicit checked/unchecked overflow behavior, division/remainder guards,
  shift masking, and exact ordered required checks; and
- the 15 accepted scalar/control semantic rows assigned in section 7.

It does not lower `CallStatic` or emit public artifacts. The primary test owner
is `crates/mpk-cli/tests/csharp_lowering.rs`.

Exit gate: complete internal lowering is deterministic, every required check
is present exactly once in canonical order, and missing/extra/reordered checks
fail before emission.

Completion evidence at the T09 exit gate: `LoweringModel.cs`,
`LoweringBuilder.cs`, and `LoweringValidation.cs` defined and validated the
private typed call-free portion for all five frozen scalar types, stable
locals/copies/results, false-before-true CFG traversal, block-parameter joins,
early returns, all 34 non-call operation mappings, 12 Roslyn checked-state
records, and 20 identity/implicit/explicit conversion rules. Explicitly
written widening numeric casts were admitted as the frozen conversion table
requires. Checked add/subtract/multiply/negate, signed and unsigned
division/remainder, and masked shifts regenerated their complete per-
instruction checks; a separately retained canonical ledger rejected missing,
extra, duplicate, and reordered entries before T10 emission. The primary Rust
owner and pinned executable harness covered all 15 T09 semantic rows,
deterministic IDs/evaluation/control flow, diagnostic mutations, and the then-
closed `CallStatic` boundary. The candidate remained unregistered and
unavailable after private lowering. Its reviewed DLL was 158,720 bytes with
SHA-256
`bf94b267c3a67af9057ce103cbffbe3bebfeb307f4ebe7c3335a2756d94bc81e`.

### CSHARP-02-T10 Lower static calls and emit stable VIR, maps, and manifests

Depends on: `CSHARP-02-T09`.

Status: Complete (2026-08-27).

Owns:

- the `direct_static_call` operation mapping, `CallStatic`, callee-first
  declaration dependencies, and semantic row `M27`;
- canonical function/value/block IDs independent of Roslyn ordinals/captures;
- exact UTF-16-boundary to UTF-8-byte source mapping with no synthetic origin;
- complete staged VIR v1, source-map v1, frontend-stage source-manifest v1,
  and frontend-success envelope bytes; and
- `vir`, `source_map`, and `manifest` compiled-profile contracts.

The primary test owner is `crates/mpk-cli/tests/csharp_emission.rs`.

Exit gate: every successful source case has complete canonical artifacts and
hashes, every emitted instruction/terminator has a faithful origin, and the
staged shared validators accept the bytes without a C# special case.

Completion evidence: `LoweringBuilder.cs` and `LoweringValidation.cs` lower
exact-signature source static calls in left-to-right evaluation order, bind
every edge to the normalized callee contract hash, and regenerate the frozen
callee-first lexical topological order. Stable IDs derive only from canonical
method identity and deterministic structural traversal. `VirEmitter.cs`,
`SourceMapEmitter.cs`, `SourceManifestEmitter.cs`, and
`FrontendSuccessEmitter.cs` serialize complete canonical successor artifacts
in memory and compute all domain-separated self-excluding hashes. The source
mapper builds an exact per-file UTF-16-boundary-to-UTF-8-byte table, cross-
checks Roslyn line positions, emits only captured-source origins, and owns all
six frozen map cases. The pinned executable harness covers static-call
linkage/order, ID stability under trivia and Unicode shifts, canonical byte
determinism, exact profile contracts, and closed map failures. The primary
Rust owner validates the emitted VIR, map, frontend manifest, and complete
success envelope with the shared successor validators and no C# branch. The
candidate remains unregistered; its reviewed DLL is 190,464 bytes with
SHA-256
`c2a999ded31b8825670fcc824719162efb5c5d40ae7c93140411242820bb43ee`.

### CSHARP-02-T11 Close C# frontend diagnostics, limits, and source-case vectors

Depends on: `CSHARP-02-T10`.

Status: Complete (2026-08-27).

Owns:

- exact issue normalization, two-stage sorting, redaction, status/exit/phase
  precedence, and all 44 stable diagnostics;
- every C# profile counter and operational output/diagnostic budget;
- materialization and execution of all 30 accepted and 88 rejected cases;
- complete hashes, mutations, multi-fault precedence, and artifact-free
  non-success assertions; and
- the frontend-owned portion of the C# profile implementation executor.

The primary aggregate owner is
`crates/mpk-cli/tests/csharp_frontend_vectors.rs`. Launcher/isolation,
compiled-profile integration, upgrade, and whole-release activation remain
explicitly owned by T12, T15-T19, and T20.

Exit gate: every frontend-owned accepted/rejected, diagnostic, precedence,
limit, and hash vector executes against candidate implementation code;
deliberate removal of any source case or required check fails the aggregate
test; and the active CLI still has no C# route.

Completion evidence: `FrontendDiagnostics.cs`, `FrontendLimits.cs`, and
`FrontendProtocol.cs` close the 44-code registry, invariant Roslyn diagnostic
normalization, raw-record and public-Issue sorting, redaction, status/phase/
exit precedence, all 32 exact-boundary counters, bounded canonical artifact
writes, and artifact-free canonical non-success envelopes. The pinned
aggregate harness executes all 30 accepted source programs through complete
capture-independent frontend stages, projects every required operation and
the exhaustive canonical check ledger, and binds all 88 rejection IDs to the
already executed capture, Roslyn, subset, contract, lowering, emission, and
build-input mutation owners. It also executes every diagnostic, precedence,
limit, semantic-row, and hash record and emits an exact report. The primary
Rust owner rejects missing or reordered vectors and validates every reported
non-success envelope with the shared successor protocol. All prior pinned C#
owner harnesses run in the same aggregate build. The candidate remains
unregistered; its reviewed DLL is 208,384 bytes with SHA-256
`0783dc269c152ad1b13e77f42f9eff6f6891002c65890bc1445f2fe1a1a0410d`.

### CSHARP-02-T12 Assemble C# candidate bundles and the staged sandbox runner

Depends on: `CSHARP-02-T11`.

Owns:

- successor release-registry, frontend-bundle, toolchain-bundle,
  bundle-candidate, and release-tuple models in staging;
- deterministic C# frontend/toolchain candidate inventories and content
  hashes;
- exact .NET launcher argv/environment/runtime/native/reference layout;
- descriptor-relative registry/bundle resolution, immutable launch identity,
  filesystem/process/network/cgroup/tmpfs/stream controls, and protocol
  validation; and
- all launcher and isolation vectors plus the `frontend` and `release`
  compiled-profile contracts.

No candidate is installed in the active bundle registry. The primary test
owner is `crates/mpk-cli/tests/csharp_frontend_runner.rs`.

Exit gate: the staged installed-tree fixture launches only the reviewed C#
bytes under the frozen environment, all hostile isolation cases fail closed,
and production tuple resolution remains Go/Rust-only.

### CSHARP-02-T13 Migrate the Go producer to successor artifacts in staging

Depends on: `CSHARP-02-T12`.

Owns staged migration of:

- `go2vir` frontend envelopes, semantic contexts, selections, VIR, contracts,
  source maps, manifests, hash domains, and candidate bundle descriptor;
- regenerated Go staging fixtures and exact old/new rejection; and
- a semantic-difference report proving no Go source behavior, required check,
  VC input intent, or deterministic diagnostic changed.

The active Go binary and fixtures remain unchanged. The primary test owner is
`crates/mpk-vc/tests/successor_go_migration.rs`.

Exit gate: staged `go2vir` emits only the successor source-artifact family,
the complete Go corpus remains semantically equal to the active baseline,
staged validators reject current artifacts, and active validators reject the
staged artifacts.

### CSHARP-02-T14 Migrate the Rust producer and private driver in staging

Depends on: `CSHARP-02-T13`.

Owns staged migration of:

- `rust2vir` frontend envelopes, semantic contexts, selections, VIR,
  contracts, source maps, manifests, hash domains, and candidate descriptor;
- Rust private driver request, result, raw-lowering, and raw-source-map schemas
  plus their new request/payload hash domains;
- regenerated Rust staging fixtures and exact old/new rejection; and
- a semantic-difference report proving no Rust source behavior, required
  check, target behavior, or deterministic diagnostic changed.

The active Rust binaries and fixtures remain unchanged. The primary test owner
is `crates/mpk-vc/tests/successor_rust_migration.rs`.

Exit gate: staged `rust2vir` and its subordinate driver emit only successor
artifacts, the complete Rust corpus remains semantically equal to the active
baseline, the subordinate protocol accepts only its successor identities, and
active/staged validators reject one another's public artifacts.

### CSHARP-02-T15 Stage successor VC and skeleton integration

Depends on: `CSHARP-02-T14`.

Owns staged implementations of:

- `mpk.vc.v2`, `mpk.vc.cert_skeleton.v2`, and `MPK-VC-2.0`;
- exact semantic-context, source-VIR, manifest, contract, check, group, and
  declaration linkage for Go, Rust, and C#;
- unchanged checked Bool/BV foundations and profile-selected required-check
  regeneration; and
- the `vc` C# compiled-profile contract.

The primary owner is `crates/mpk-vc/tests/successor_vc.rs`.

Exit gate: all three staged profiles generate canonical successor VC/skeleton
bytes, Go/Rust obligation semantics remain unchanged, and each C# required
check reaches the same checked foundation path as its frozen mapping.

### CSHARP-02-T16 Stage policy, evidence, and certificate integration

Depends on: `CSHARP-02-T15`.

Owns staged implementations of:

- `mpk.policy.scan.v2`, `mpk.policy.evidence.v2`, and
  `mpk.program_certificate.alpha.v1` over unchanged Certificate v0;
- profile-selected Go, Rust, and C# strategy/checker/axiom/recipe contracts;
- C# policy `payment-policy-csharp-alpha`, certificate-stage manifests,
  source-free dual-checker execution, and structured reproduction recipes;
  and
- the `policy` and `evidence` C# compiled-profile contracts.

The primary owner is `crates/mpk-cli/tests/csharp_policy_verify.rs`.

Exit gate: representative C# obligations and certificates pass both checkers
from identical bytes, Go/Rust staged verdicts remain unchanged, and no helper
or compiler result is promoted to trusted evidence.

### CSHARP-02-T17 Stage successor AI explanation integration

Depends on: `CSHARP-02-T16`.

Owns staged implementations of:

- `mpk.ai.explain.request.v2` and `mpk.ai.explanation.v2`;
- exact profile/context/strategy selection from validated evidence;
- C# display/redaction projection with no source, contract, compiler prose, or
  unsanitized identifier leakage; and
- the `ai` C# compiled-profile contract.

The primary owner is `crates/mpk-cli/tests/csharp_ai_explain.rs`. Released AI
explanation routes remain on their current schemas.

Exit gate: all three staged profiles produce deterministic sanitized requests,
AI output remains untrusted, and no response changes local evidence or proof
acceptance.

### CSHARP-02-T18 Stage successor AI API integration

Depends on: `CSHARP-02-T17`.

Owns staged implementation of `mpk.ai.api.v2`, including exact
profile/context/selection-aware session linkage, successor VIR/VC operations,
old/new route rejection, and unchanged certificate-checking authority.

The primary owner is `crates/mpk-api/src/v2_tests.rs`. The released API route
remains on its current schema.

Exit gate: sessions cannot mix profiles or contexts, old/crossed helper
artifacts reject, failed operations do not mutate accepted state, and API
success cannot bypass canonical certificate checking.

### CSHARP-02-T19 Complete cross-profile hardening and release rehearsal

Depends on: `CSHARP-02-T18`.

Owns:

- complete positive, negative, boundary, mutation, hash, isolation, fuzz, and
  adversarial corpora across the three profiles;
- bounded C# differential execution against runtime 10.0.11;
- parser, contract, protocol, compiler-output, and resource fuzz targets with
  checked-in regression seeds;
- two-clean-build and two-run equality for every bundle and canonical
  artifact;
- all 12 C# upgrade cases, old/new cross-rejection, no-plugin scans,
  credential/path/network leakage scans, and a zero-new-category axiom review;
- the complete candidate implementation executor for every top-level field in
  `csharp-profile-v0.json`; and
- a complete staged installed-release rehearsal and empty findings ledger.

The primary aggregate owner is
`crates/mpk-cli/tests/csharp_profile_vectors.rs`, with release orchestration in
`crates/mpk-cli/tests/csharp_release_gate.rs` and
`scripts/check-csharp-frontend.sh`. T19 adds the aggregate implementation owner
to the C# vector-manifest entry without changing the frozen vector or existing
specification owner. The gate performs no provisioning or network access.

Exit gate: the staged whole release passes every fast/full/C# gate twice from
reviewed offline inputs, all expected artifacts are byte-identical, and the
implementation review ledger has zero findings.

### CSHARP-02-T20 Perform the atomic successor cutover and C# release

Depends on: `CSHARP-02-T19` and transitively every earlier CSHARP-02 task.

Owns one indivisible release change that:

- installs the exact revision-2 semantic registry and successor bundle
  registry beside one successor `bin/mpk`;
- activates the exact Go, Rust, and C# release tuples and all nine compiled C#
  profile contracts;
- switches every producer/consumer, CLI/API route, fixture, example, report,
  recipe, and active document to the sole successor identities;
- regenerates active bytes and requires equality with the reviewed staging
  set;
- removes old public helper-schema parsers, staging gates, compatibility
  flags, old hash fallbacks, and any dual-input route;
- removes the executable staging tree after byte-equal installation, retaining
  only a non-production reviewed migration report when required; and
- runs both source-free checkers, complete installed-release gates, axiom
  review, and final zero-finding code review.

The primary cutover owner is
`crates/mpk-cli/tests/successor_atomic_cutover.rs` plus the installed release
gate. T20 adds that implementation owner to both registry-vector manifest
entries without changing either frozen vector or existing specification
owner. No rollback mixes release generations; rollback replaces the whole
release image.

Exit gate: C# is active only through the shared registered path, Go/Rust are
active only through the successor path, old and crossed identities reject,
Certificate v0 remains unchanged, both checkers agree, and `JAVA-03` becomes
eligible.

## 5. Normative specification traceability

The primary owner proves admission or serialization. Listed integration
consumers must reuse that result and may not implement a second interpretation.

| Normative section | Primary task | Required integration consumer |
| --- | --- | --- |
| C# 1 and 1.1: trust, identity, revision-2 entry/root | T03 | T12, T20 |
| C# 2.1: semantic parameters | T03 | T05, T12, T20 |
| C# 2.2: selection and canonical method IDs | T05 | T07, T08, T10, T12 |
| C# 3.1: frozen versions, archives, references, host layout | T02 | T06, T12, T19 |
| C# 3.2 and 3.3: public Roslyn boundary and exact session | T06 | T07, T09, T19 |
| C# 3.4: build isolation | T02 | T19 |
| C# 3.4: execution isolation and launcher | T12 | T19, T20 |
| C# 4: source transport and immutable compilation closure | T05 | T07, T10 |
| C# 5.1 and 5.2: declarations, source types, literals | T07 | T09 |
| C# 5.3: statements, evaluation, and control flow | T07 | T09 |
| C# 5.4 and 5.5: overflow, operators, conversions, checks | T09 | T11, T15 |
| C# 5.6: calls, purity, and abrupt completion | T07 | T10, T15 |
| C# 5.7: closed rejected semantic rows | T07 | T11, T19 |
| C# 6: contract sidecars and normalized contracts | T08 | T10, T15, T16 |
| C# 7: operation/CFG adapter admission | T06 | T07, T09, T10 |
| C# 8: VIR/map/manifest emission | T10 | T12, T15, T16, T19 |
| C# 8: runner/release mapping | T12 | T20 |
| C# 8: VC mapping | T15 | T20 |
| C# 8: policy/evidence mapping | T16 | T20 |
| C# 8: AI mapping | T17 | T20 |
| C# 9 and 10: phases, diagnostics, status, limits | T11 | T12, T19 |
| C# 11: compiled profile payloads | field owners in section 8 | T03, T19, T20 |
| C# 12: frontend semantic/case/diagnostic/limit/hash vectors | T11 | T19, T20 |
| C# 12: launcher and isolation vectors | T12 | T19, T20 |
| C# 12: profile-contract and upgrade aggregate | T19 | T20 |
| C# 13: upgrade procedure | T19 | T20 |
| C# 14 and registry 12: atomic activation/no dual IR | T20 | none |
| Registry 1 through 6 and 10 through 11 | T03 | T04, T12-T20 |
| Registry 7: release binding | T12 | T20 |
| Registry 8: successor source artifacts | T04 | T10, T13-T20 |
| Registry 8: release artifacts | T12 | T13-T20 |
| Registry 8: VC artifacts | T15 | T20 |
| Registry 8: policy/evidence/certificate artifacts | T16 | T20 |
| Registry 8: AI explanation artifacts | T17 | T20 |
| Registry 8: AI API artifacts | T18 | T20 |
| Registry 9: contract/VIR/map/manifest hash domains | T04 | T19, T20 |
| Registry 9: release-registry hash domain | T12 | T19, T20 |
| Registry 9: Rust private-driver hash domains | T14 | T19, T20 |
| Registry 9: VC hash domain | T15 | T19, T20 |
| Registry 13: transport/registry/context/profile/limit vectors | T03 | T19, T20 |
| Registry 13: source-artifact hash-domain vectors | T04 | T19, T20 |
| Registry 13: release-registry hash-domain vector | T12 | T19, T20 |
| Registry 13: Rust-driver hash-domain vectors | T14 | T19, T20 |
| Registry 13: VC hash-domain vector | T15 | T19, T20 |
| Registry 13: whole-release migration vectors | T20 | none |
| Registry 14: exact C# handoff and predecessor preservation | T03 | T19, T20 |

## 6. Complete vector-field ownership

The field groups in each table are disjoint and their union is the exact
top-level key set of that vector container. Container metadata remains frozen;
an implementation owner adds executable coverage without changing the existing
`owner_test` value.

### 6.1 `csharp-profile-v0.json`

| Exact top-level field(s) | Primary task | Required executable owner/result |
| --- | --- | --- |
| `schema`, `owner_test`, `spec_schema`, `mechanism_schema` | T19 | complete aggregate implementation container validation |
| `profile_identity`, `semantic_parameters` | T03 | runtime registry/context validation |
| `selection_fixture`, `selection_sha256` | T05 | capture/selection implementation test |
| `contract_fixture`, `contract_sidecar_sha256`, `normalized_contract_fixture` | T08 | contract implementation test |
| `toolchain_inputs` | T02 | build-input/inventory implementation test |
| `compiler_session` | T06 | Roslyn session implementation test |
| `launcher_contract` | T12 | installed-tree runner implementation test |
| `case_harness` | T11 | exact accepted/rejected materializer |
| `source_map_cases` | T10 | complete UTF-16/UTF-8 map executor |
| `profile_contracts` | T19 | aggregate nine-contract staging check using section 8 owners; T20 activates |
| `type_mappings`, `roslyn_checked_state_cases`, `conversion_rules` | T09 | scalar/control/conversion lowering and check implementation test |
| `operation_mappings` | T10 | T09 executes 34 non-call mappings; T10 executes `direct_static_call` and closes the array |
| `semantic_rows` | T11 | aggregate closure using section 7 primary owners |
| `accepted_cases`, `rejected_cases` | T11 | all 118 source cases executed |
| `precedence_cases`, `diagnostic_registry`, `diagnostic_normalization` | T11 | exact issue/status/phase executor |
| `limit_cases` | T11 | exact boundary executor |
| `hash_cases` | T11 | implementation hash recomputation |
| `isolation_cases` | T12 | sandbox/launcher executor |
| `upgrade_cases` | T19 | upgrade and old/new cross-rejection executor |

### 6.2 `semantic-profile-registry-v1.json`

| Exact top-level field(s) | Primary task | Required executable owner/result |
| --- | --- | --- |
| `schema`, `spec_schemas`, `owner_test`, `fixtures` | T03 | runtime container/fixture validation |
| `transport_cases`, `hash_cases`, `registry_cases` | T03 | strict registry parser and hash executor |
| `context_cases`, `profile_envelope_cases`, `limit_cases` | T03 | runtime context/contract executor |
| `hash_domain_migration_cases` | T20 | T04 owns contract/VIR/map/manifest, T12 release registry, T14 Rust driver, T15 VC, and T19 aggregates staging |
| `migration_cases` | T20 | T19 rehearses whole-release cross-rejection; T20 installs it |

### 6.3 `semantic-profile-registry-v2.json`

| Exact top-level field(s) | Primary task | Required executable owner/result |
| --- | --- | --- |
| `schema`, `owner_test`, `mechanism_spec`, `profile_spec` | T03 | runtime container/authority validation |
| `predecessor`, `csharp_entry`, `registry`, `hash_cases`, `append_only_cases` | T03 | revision-2 runtime and append-only executor |
| `activation_cases` | T20 | T19 rehearses every case; T20 executes the installed-release activation/no-dual-input gate |

## 7. Complete semantic-row ownership

The five rows below are disjoint and cover `M01` through `M34` exactly once.

| Primary task | Disposition | Exact rows |
| --- | --- | --- |
| T06 | accept under profile restrictions | `M33` |
| T07 | reject before VIR | `M03`, `M04`, `M05`, `M06`, `M15`, `M17`, `M20`, `M22`, `M23`, `M24`, `M25`, `M26`, `M28`, `M30`, `M31`, `M32` |
| T08 | accept under profile restrictions | `M34` |
| T09 | accept under profile restrictions | `M01`, `M02`, `M07`, `M08`, `M09`, `M10`, `M11`, `M12`, `M13`, `M14`, `M16`, `M18`, `M19`, `M21`, `M29` |
| T10 | accept under profile restrictions | `M27` |

T11 is the aggregate executor and fails if a row is missing, duplicated, moved
to a different disposition, or accepted without its primary owner's checks.

## 8. Compiled-profile and successor-artifact ownership

### 8.1 Nine C# compiled-profile contracts

| Contract field | Exact contract ID | Primary implementation task |
| --- | --- | --- |
| `ai` | `mpk.profile.ai.csharp_scalar.v0` | T17 |
| `evidence` | `mpk.profile.evidence.csharp_scalar.v0` | T16 |
| `frontend` | `mpk.profile.frontend.csharp_scalar.v0` | T12 |
| `manifest` | `mpk.profile.manifest.csharp_scalar.v0` | T10 |
| `policy` | `mpk.profile.policy.csharp_scalar.v0` | T16 |
| `release` | `mpk.profile.release.csharp_scalar.v0` | T12 |
| `source_map` | `mpk.profile.source_map.csharp_scalar.v0` | T10 |
| `vc` | `mpk.profile.vc.csharp_scalar.v0` | T15 |
| `vir` | `mpk.profile.vir.csharp_scalar.v0` | T10 |

T03 owns closed ID/field dispatch and payload-shape validation. The field owner
implements the consumer behavior. T20 proves all nine are compiled and active
for the unchanged C# entry hash.

### 8.2 Twenty-two successor identities

| Surface | Current active identity | Sole successor identity | Primary task | Activation |
| --- | --- | --- | --- | --- |
| semantic registry | none | `mpk.semantic_profile.registry.v1` | T03 | T20 |
| VIR | `mpk.vir.v0` | `mpk.vir.v1` | T04 | T20 |
| frontend envelope | `mpk.frontend.cli.v0` | `mpk.frontend.cli.v1` | T04 | T20 |
| Rust driver request | `mpk.rust.driver.request.v0` | `mpk.rust.driver.request.v1` | T14 | T20 |
| Rust driver result | `mpk.rust.driver.v0` | `mpk.rust.driver.v1` | T14 | T20 |
| Rust raw lowering | `mpk.rust.driver.lowering.v0` | `mpk.rust.driver.lowering.v1` | T14 | T20 |
| Rust raw source map | `mpk.rust.driver.raw_source_map.v0` | `mpk.rust.driver.raw_source_map.v1` | T14 | T20 |
| source map | `mpk.source_map.v0` | `mpk.source_map.v1` | T04 | T20 |
| source manifest | `mpk.source_manifest.v0` | `mpk.source_manifest.v1` | T04 | T20 |
| release registry | `mpk.release.bundle_registry.v0` | `mpk.release.bundle_registry.v1` | T12 | T20 |
| release registry ID | `mpk.release.registry.v0` | `mpk.release.registry.v1` | T12 | T20 |
| frontend descriptor | `mpk.release.frontend_bundle.v0` | `mpk.release.frontend_bundle.v1` | T12 | T20 |
| toolchain descriptor | `mpk.release.toolchain_bundle.v0` | `mpk.release.toolchain_bundle.v1` | T12 | T20 |
| source-only candidate | `mpk.release.bundle_candidate.v0` | `mpk.release.bundle_candidate.v1` | T12 | T20 |
| VC | `mpk.vc.v1` | `mpk.vc.v2` | T15 | T20 |
| VC skeleton | `mpk.vc.cert_skeleton.v1` | `mpk.vc.cert_skeleton.v2` | T15 | T20 |
| policy scan | `mpk.policy.scan.v1` | `mpk.policy.scan.v2` | T16 | T20 |
| policy evidence | `mpk.policy.evidence.v1` | `mpk.policy.evidence.v2` | T16 | T20 |
| program-certificate assembly | `mpk.program_certificate.alpha.v0` | `mpk.program_certificate.alpha.v1` | T16 | T20 |
| AI API profile | `mpk.ai.api.v1` | `mpk.ai.api.v2` | T18 | T20 |
| AI sanitized request | `mpk.ai.explain.request.v1` | `mpk.ai.explain.request.v2` | T17 | T20 |
| AI explanation report | `mpk.ai.explanation.v1` | `mpk.ai.explanation.v2` | T17 | T20 |

Every old discriminator or flat field has a negative staging test in its
primary task and an installed-release rejection test in T20.

## 9. Required implementation-test ownership

These are minimum owner paths. Focused tests may be added, but none replaces
the aggregate owner named for its task.

| Task | Minimum primary test or gate owner | Minimum targeted command |
| --- | --- | --- |
| T02 | `crates/mpk-cli/tests/csharp_build_inputs.rs` | `./scripts/build-csharp-frontend.sh --check`; `cargo test -p mpk-cli --test csharp_build_inputs` |
| T03 | `crates/mpk-vc/tests/semantic_profile_registry_runtime.rs` | `cargo test -p mpk-vc --test semantic_profile_registry_runtime` |
| T04 | `crates/mpk-vc/tests/successor_source_artifacts.rs`, `crates/mpk-cli/tests/successor_frontend_protocol.rs` | `cargo test -p mpk-vc --test successor_source_artifacts`; `cargo test -p mpk-cli --test successor_frontend_protocol` |
| T05 | `crates/mpk-cli/tests/csharp_capture.rs` | `cargo test -p mpk-cli --test csharp_capture` |
| T06 | `crates/mpk-cli/tests/csharp_roslyn_session.rs` | `cargo test -p mpk-cli --test csharp_roslyn_session` |
| T07 | `crates/mpk-cli/tests/csharp_subset.rs` | `cargo test -p mpk-cli --test csharp_subset` |
| T08 | `crates/mpk-cli/tests/csharp_contracts.rs` | `cargo test -p mpk-cli --test csharp_contracts` |
| T09 | `crates/mpk-cli/tests/csharp_lowering.rs` | `cargo test -p mpk-cli --test csharp_lowering` |
| T10 | `crates/mpk-cli/tests/csharp_emission.rs` | `cargo test -p mpk-cli --test csharp_emission` |
| T11 | `crates/mpk-cli/tests/csharp_frontend_vectors.rs` | `cargo test -p mpk-cli --test csharp_frontend_vectors` |
| T12 | `crates/mpk-cli/tests/csharp_frontend_runner.rs` | `./scripts/build-release-bundles.sh --check csharp`; `./scripts/check-release-bundles.sh --fixture csharp`; `cargo test -p mpk-cli --test csharp_frontend_runner` |
| T13 | `crates/mpk-vc/tests/successor_go_migration.rs` | `cargo test -p mpk-vc --test successor_go_migration` |
| T14 | `crates/mpk-vc/tests/successor_rust_migration.rs` | `cargo test -p mpk-vc --test successor_rust_migration` |
| T15 | `crates/mpk-vc/tests/successor_vc.rs` | `cargo test -p mpk-vc --test successor_vc` |
| T16 | `crates/mpk-cli/tests/csharp_policy_verify.rs` | `cargo test -p mpk-cli --test csharp_policy_verify` |
| T17 | `crates/mpk-cli/tests/csharp_ai_explain.rs` | `cargo test -p mpk-cli --test csharp_ai_explain` |
| T18 | `crates/mpk-api/src/v2_tests.rs` | `cargo test -p mpk-api v2_tests` |
| T19 | `crates/mpk-cli/tests/csharp_profile_vectors.rs`, `crates/mpk-cli/tests/csharp_release_gate.rs`, `scripts/check-csharp-frontend.sh` | `cargo test -p mpk-cli --test csharp_profile_vectors`; `./scripts/check-csharp-frontend.sh` |
| T20 | `crates/mpk-cli/tests/successor_atomic_cutover.rs`, installed release gates | `cargo test -p mpk-cli --test successor_atomic_cutover`; `sudo ./scripts/check-all.sh` |

No .NET test framework or NuGet package outside the frozen build graph may be
introduced merely to host tests. Host-side integration tests may launch the
candidate through the private staging harness.

## 10. Implementation-surface ownership

This table bounds normal source ownership. It does not authorize unrelated
cleanup in a listed directory, and a task must inspect the current tree before
choosing exact files.

| Task | Primary implementation surfaces |
| --- | --- |
| T02 | `csharp-tools/csharp2vir/`, `release/build-inputs/csharp/`, C# offline build/check scripts |
| T03 | semantic-profile registry/context models and validators in `mpk-vc` |
| T04 | successor VIR/contract/map/manifest models in `mpk-vc` and private frontend protocol models in `mpk-cli` |
| T05 | C# CLI, path, snapshot, capture, source-transport, and hash modules under `csharp-tools/csharp2vir/` |
| T06 | C# Roslyn loader/session/reference/diagnostic-adapter modules under `csharp-tools/csharp2vir/` |
| T07 | C# declaration/subset/symbol/closure/purity/CFG validation modules under `csharp-tools/csharp2vir/` |
| T08 | C# sidecar JSON/typecheck/attachment/hash modules under `csharp-tools/csharp2vir/` |
| T09 | C# scalar/control/conversion/check lowering modules under `csharp-tools/csharp2vir/` |
| T10 | C# call/ID/emitter/source-map/manifest modules under `csharp-tools/csharp2vir/` |
| T11 | C# issue/limit/protocol modules and candidate vector harnesses |
| T12 | `mpk-cli` release registry, bundle, runner, sandbox code plus bundle build/check scripts and staging descriptors |
| T13 | `go-tools/go2vir/` and Go artifacts under `develop/migrations/csharp-02-staging/` |
| T14 | `rust-tools/rust2vir/`, its private protocols, and Rust staging artifacts |
| T15 | `mpk-vc` successor VC/skeleton models, validation, generation, grouping, and hashes |
| T16 | `mpk-cli` policy/evidence/program-certificate staging and C# policy fixtures |
| T17 | `mpk-cli` AI explanation staging and sanitized request/report fixtures |
| T18 | `mpk-api` successor route, session, VIR, and VC staging |
| T19 | C# fixtures/fuzz seeds, migration reports, deterministic release rehearsal, and C# gate scripts |
| T20 | active release registries/bundles, public producers/consumers/routes, fixtures/examples/reports, and active documentation |

If an implementation task requires a surface assigned to a later task, it
must stop or use the private injected boundary already delivered by an earlier
task; it does not pull the later task forward.

## 11. Common definition-of-done traceability

| Language-phase requirement | Primary closing task |
| --- | --- |
| frozen subset/profile/toolchain/selection/contract/diagnostic/limit/upgrade contracts | MLANG-01-T03 input; executable closure in T11 and T19 |
| complete vector ownership | field owners in sections 6 through 8; aggregate in T19; activation in T20 |
| pinned build and execution inputs | T02 and T12 |
| deterministic registered bundles | candidate in T12; registration in T20 |
| source-to-VIR lowering and exact checks | T09 and T10 |
| source maps and both manifest stages | T10 and T16 |
| VC and skeleton generation | T15 |
| policy, evidence, reporting, and recipes | T16 |
| exact omitted-feature rejection | T07 and T11 |
| differential execution | T19 |
| two-build/two-run determinism | T02, T12, and T19 |
| path, credential, network, and ambient-state isolation | T05, T12, and T19 |
| parser/protocol/compiler-output/resource fuzzing | T19 |
| canonical C# certificates accepted by both checkers | T16; installed proof in T20 |
| zero-new-category axiom review | T16 and T20 |
| no dual helper schema and atomic Go/Rust migration | T13-T14 staging; T20 activation |
| empty implementation review ledger | T19 rehearsal; T20 final release |

Every implementation task must run its targeted owner plus:

```sh
cargo fmt --all -- --check
git diff --check
./scripts/check-fast.sh
```

T02 and T12 through T20 are compiler/release or cross-boundary work and
additionally run the applicable installed-tree, checker-agreement, and full
repository gates. A gate never provisions, restores, upgrades, or downloads a
toolchain.

## 12. Blocker and review ledgers

The implementation blocker format is:

| Blocker | Detected by | Frozen requirement | Why implementation cannot decide | Required owner | Status |
| --- | --- | --- | --- | --- | --- |

T01 through T08 have no blocker. A later task records a blocker before changing
scope. It may be closed only by a separately named, serial governance/
specification task followed by regeneration and review of every affected
frozen input.

The T01 review ledger is empty. Its closed traceability inventory contains:

- 20 ordered CSHARP-02 tasks, with T02 the sole ready successor at T01 close;
- all 31 top-level C# profile-vector fields;
- all 12 registry-v1 and 10 registry-v2 top-level fields;
- all 34 semantic rows;
- all nine C# compiled-profile contracts;
- all 22 successor identities; and
- every common language definition-of-done requirement.

No production source, normative specification, vector, hash, bundle,
registry, fixture, or accepted identity changed in T01.

The T02 review ledger is empty. Its closed implementation inventory contains:

- one source-inert C# project with six hash-pinned build inputs;
- one canonical build descriptor and one canonical 18-file candidate/notice
  inventory;
- all six frozen archive records, exact SDK/runtime extraction counts and
  modes, four package records, two managed projections, three reference
  metadata files, and 167 reference assemblies;
- two path-independent clean builds and runtime probes under closed,
  network-isolated environments; and
- the primary Rust implementation owner plus synthetic hostile-archive and
  hostile-ambient checks.

No production parser, frontend envelope, successor schema implementation,
bundle registration, release tuple, normative artifact, or accepted identity
changed in T02. T03 was the sole ready successor at T02 close.

The T03 review ledger is empty. Its closed implementation inventory contains:

- one inactive successor registry module with exact revision-1 and revision-2
  embedded identities and immutable Go, Rust, and C# entry dispatch;
- strict registry transport plus 21 stable error/status codes, ten common
  limits, context/selection/profile-envelope models, and linkage checks;
- exact C# semantic parameters and nine declarative compiled-profile payload
  validators, with detailed C# selection/capture semantics still owned by
  T05;
- runtime execution of every T03-owned registry vector and append-only
  assertion; and
- an append-only implementation-owner manifest mechanism that preserves each
  vector-declared owner as the ordered prefix.

No normative specification or vector byte, active Go/Rust schema/parser,
installed registry, C# release tuple, bundle, frontend route, checker input,
or accepted identity changed in T03. T04 is the sole ready successor.

The T04 review ledger is empty. Its closed implementation inventory contains:

- four sealed successor source-artifact models and their exact new hash
  domains, with complete semantic-context, selection, release-root, input,
  source-reference, and artifact-root linkage;
- one hidden, explicitly injected successor frontend-protocol validator with
  canonical transport, issue-shape, request-identity, status/exit, and success-
  artifact validation;
- canonical staging round trips derived in memory from unchanged active
  fixtures, the exact frozen normalized C# contract hash, all T04-owned domain
  migration rows, member-mismatch rejection, and bidirectional old/new parser
  rejection; and
- one append-only registry-vector implementation owner with no vector-byte or
  declared specification-owner change.

No active producer, consumer, runner, command, bundle, release tuple, registry
root, normative specification, frozen vector, or accepted production identity
changed in T04. T05 was the sole ready successor.

The T05 review ledger is empty. Its closed implementation inventory contains:

- one exact ordered private CLI grammar with fixed C# semantic, target,
  registry, entry, source-root, and toolchain-root assertions and bounded
  release-identity scalars;
- immutable validated selection and canonical-method models, exact portable
  source/contract paths, canonical JSON, and the frozen
  `MPK-CSHARP-SELECTION-0.1` digest;
- one Linux x86-64 no-follow inventory/capture path that closes selected files
  and implied directories, rejects symbolic/hard links and path/type/identity
  collisions, checks every file/snapshot counter before retaining excess, and
  copies each selected byte sequence exactly once;
- one source transport gate over captured buffers only, with strict UTF-8,
  no BOM/NUL/CR/noncharacter, and final-LF enforcement; and
- the primary Rust owner plus a pinned executable C# mutation harness run by
  the offline two-build gate.

No Roslyn parse/compilation API, syntax inspection, active registry/bundle,
released runner or command, successor success artifact, normative vector byte,
or production identity changed in T05. T06 is the sole ready successor.

The T06 review ledger is empty. Its closed implementation inventory contains:

- one exact source session with strict UTF-8/SHA-256 text, C# 14 regular parse
  options, stored selection paths/order, and syntax-first diagnostics;
- one canonical hash-validated 167-file metadata projection and exact x64
  Release compilation-option object, with every publicly observable getter
  checked before compilation;
- direct public Roslyn semantic-model, symbol, type, conversion, operation,
  and method-body CFG adapters with no compiler driver, emit, reflection over
  nonpublic members, speculative model, or syntax-text fallback;
- fail-closed release/source/metadata/lowering toolchain drift classification;
  and
- the primary Rust owner plus a pinned executable getter/mutation/API harness
  run by the offline two-build gate.

No subset admission, closure decision, contract parsing, lowering, VIR or
success-artifact emission, active registry/bundle, released runner or command,
normative vector byte, or production identity changed in T06. T07 is the sole
ready successor.

The T07 review ledger is empty. Its closed implementation inventory contains:

- exact declaration, ASCII source-spelling, predefined type/literal,
  statement/control, built-in operator, and numeric-conversion admission;
- one conservative all-selected-root source-call closure with deterministic
  callee-first ordering, source-dead call retention, acyclicity, exact source
  resolution, and unrelated-method rejection;
- exhaustive operation-based purity, immutable-parameter, inert static-type,
  and CFG definite-assignment checks over only public Roslyn APIs;
- checked syntax-node, per-method/closure operation, CFG-block, and 128-method
  counters that reject before retaining an excess item; and
- the primary Rust owner plus a pinned executable C# harness covering all 16
  held-out semantic rows and accepted/boundary/mutation cases in the offline
  two-build gate.

No contract JSON parsing or normalization, lowering, required-check creation,
VIR/map/manifest or success-artifact emission, active registry/bundle,
released runner or command, normative vector byte, or production identity
changed in T07. T08 was the sole ready successor.

The T08 review ledger is empty. Its closed implementation inventory contains:

- one strict streaming sidecar parser with recursive duplicate detection,
  canonical typed integers, the exact closed expression union, and checked
  clause/node/closure/depth bounds before excess retention;
- exact successor expression typing with no implicit or mixed conversion,
  declaration-position `argN` normalization, and complete frozen semantic
  context generation without folding or source-language contract syntax;
- one selection-hash-bound attachment set with exact closure coverage,
  deterministic callee-first storage, and missing, unused, and duplicate
  refusal;
- separate raw-input, canonical-sidecar, and normalized-contract identities
  reproducing both frozen hash vectors exactly; and
- the primary Rust owner plus a pinned executable C# harness covering M34,
  all stable contract diagnostics and operators, exact hashes, and inclusive
  64/1,024/8,192/32 boundaries in the offline two-build gate.

No lowering, required-check creation, VIR/map/manifest or success-artifact
emission, active registry/bundle, released runner or command, normative vector
byte, proof-acceptance input, or production identity changed in T08. T09 is the
sole ready successor.
