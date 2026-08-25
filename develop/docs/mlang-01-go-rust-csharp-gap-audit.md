# MLANG-01-T01 Implemented Go/Rust to C# Gap Audit

Status: complete non-normative implementation audit for `MLANG-01-T01`.

Prepared: 2026-08-25.

## 1. Scope and authority

This record compares the implemented Go/Rust VIR, VC, source-map, manifest,
runner, selection, release, policy, evidence, and AI boundaries with the 18-row
C# candidate subset from `mlang-00-semantic-comparison-matrix.md` and
`mlang-00-compiler-integration-feasibility.md`. The implementation baseline is
commit `d9fc6775997b`, after the completed `MLANG-00-T03` remediation and
repeated shared-boundary audit.

The frozen Go/Rust specifications and implemented validators remain
authoritative. This audit classifies gaps; it does not activate C#, select the
successor extension mechanism, freeze a C# profile, amend a serialized schema,
or authorize production code. In particular, it adds no C# language/profile
ID, selection branch, executable, bundle, policy strategy, evidence route, AI
registration, Certificate v0 rule, checker input, or axiom category.

The comparison deliberately uses the current Rust implementation and its
tests, not the pre-implementation Rust design. Candidate behavior is reusable
only where the implemented operation, validation, VC generation, and checked
foundation agree.

## 2. Result

The implemented logical core is sufficient for all 18 C# rows that received a
T02 `GO` verdict. They need exact C# profile rules and a reviewed way to carry
the profile identity, but no new VIR value/control-flow operation and no new
checked foundation or theory.

The 34 semantic rows classify as follows:

| Class | Rows | Count | Initial C# consequence |
| --- | --- | ---: | --- |
| `P` — profile-only | `M01`, `M02`, `M07`-`M14`, `M16`, `M18`, `M19`, `M21`, `M27`, `M29`, `M33`, `M34` | 18 | Candidate for the T03 specification; still inactive. |
| `S` — registry/selection shape | none; structural gaps are recorded separately in section 5 | 0 | T02 must freeze the successor shape before T03. |
| `V` — VIR operation | `M22` | 1 | Excluded from the initial profile. |
| `F` — checked foundation/theory | `M04`, `M05` | 2 | Excluded from the initial profile. |
| `U` — unsupported | `M03`, `M06`, `M15`, `M17`, `M20`, `M23`-`M26`, `M28`, `M30`-`M32` | 13 | T03 must freeze deterministic rejection. |
| **Total** | `M01`-`M34` exactly once | **34** | No semantic row is unclassified. |

The important correction to the T01 proposal is `M22`. The implementation
already has fixed-BV carriers, total `Convert`, signed/unsigned comparisons,
and checked proof routes. A range-checked conversion therefore does not need a
new mathematical carrier or checked foundation. It would need a new closed
VIR safety-check/VC operation whose range predicate is regenerated from the
validated source and target types. Current `Convert` is Go-only and requires
an empty check array. `M22` is consequently a `V` gap and remains excluded; no
such operation is added by T02 or T03.

## 3. Classification and ownership contract

Every gap has exactly one of these classes and one next normative owner:

| Code | Class | Exact meaning | Next normative owner |
| --- | --- | --- | --- |
| `P` | profile-only | Existing implemented VIR/VC/foundation behavior is sufficient. Missing work is exact C# admission, lowering, pin, limit, diagnostic, or rejection content under a successor identity. | `MLANG-01-T03` freezes the content; `CSHARP-02` may later implement only that frozen content. |
| `S` | registry/selection shape | A current closed Go/Rust serialized union, identifier, or dispatch shape cannot carry a C# profile or selection. The gap concerns representation and fail-closed activation, not C# semantics. | `MLANG-01-T02` alone freezes the successor mechanism and atomic migration. |
| `V` | VIR operation | Existing foundations can express the proposition, but the closed VIR/VC operation or safety-check vocabulary cannot carry and validate the behavior. | `MLANG-01-T03` freezes rejection. Admission requires a separately named future successor task after governance approval. |
| `F` | checked foundation/theory | A required value domain or operation theory is absent from the checked program foundation. | `MLANG-01-T03` freezes rejection. Admission requires a separately named future foundation and successor-profile task. |
| `U` | unsupported | The behavior is intentionally outside the initial C# profile and is neither approximated nor given an unconstrained encoding. | `MLANG-01-T03` owns the explicit rejection set. |

Semantic rows are classified after assuming that a future valid C# identity
can be transported. Structural transport and dispatch gaps are separate `S`
records in section 5. This prevents the same semantic row from being labeled
both profile-only and registry-shaped. T02 owns shape only; T03 owns C#
content only; neither task owns production implementation.

## 4. Implemented evidence

| Boundary | Implemented evidence used by this audit | Reuse finding |
| --- | --- | --- |
| Semantic identity and VIR | `crates/mpk-vc/src/semantic_profile.rs`, `crates/mpk-vc/src/vir.rs`, and `crates/mpk-vc/src/vir_validate.rs` implement closed Go/Rust identities, Bool/BV types, ordered CFG, `Convert`, direct acyclic `CallStatic`, contracts, exact profile operations, and fail-closed validation. `crates/mpk-vc/tests/vir_validation.rs`, `safety_profiles.rs`, `rust_positive_corpus.rs`, and `rust_calls.rs` exercise those rules. | The operations needed by the 18 C# candidates exist; identity and profile branches are closed. |
| Safety and VC | `crates/mpk-vc/src/safety_check.rs` derives exact profile-owned check arrays and regenerates their propositions. `crates/mpk-vc/src/program_encode.rs`, `program_wp.rs`, `call_wp.rs`, `vc.rs`, `vc_canonical.rs`, and `vc_skeleton.rs` lower validated Bool/BV operations, contracts, calls, and safety groups into the checked program boundary. | Existing zero-axiom, bitvector-theory, and `Std.Program.Base` routes cover the candidate rows. |
| Source map | `crates/mpk-vc/src/source_map.rs` and `crates/mpk-vc/tests/source_map.rs` validate one source-neutral reference/origin union, normalized paths, hashes, spans, and total reference mapping, while selecting Go/Rust function-ID grammars by source language. | The map model is reusable; C# coordinate and identifier rules are profile content behind a successor identity. |
| Manifest and selection | `crates/mpk-vc/src/source_manifest.rs` and `crates/mpk-vc/tests/source_manifest.rs` implement staged source/final manifests, complete input hashes, release linkage, and a closed Go-package/Rust-library selection union with language-specific unit kinds and configuration. | Staging and hash linkage are reusable; selection/identity shape and exact C# input closure are missing. |
| Protocol, runner, and sandbox | `crates/mpk-cli/src/frontend_protocol.rs`, `frontend_runner.rs`, and `frontend_sandbox.rs`, with `crates/mpk-cli/tests/frontend_runner.rs` and `frontend_limits.rs`, implement bounded envelopes, immutable registered snapshots, closed environments, and exact Go/Rust executable branches. | Isolation concepts are reusable; the active protocol, executable allowlist, arguments, environment, mounts, and diagnostics are Go/Rust-closed. |
| Release | `crates/mpk-vc/src/release_bundle.rs` and `crates/mpk-vc/tests/release_bundle.rs` implement content-addressed frontend/toolchain inventories, compiler identities, runtime/native-runtime profiles, tuple hashing, and installed-tree validation with exact Go/Rust validators. | Inventory and closure concepts are reusable; a managed Roslyn/.NET descriptor and C# tuple need successor shape plus exact profile content. |
| Policy and evidence | `crates/mpk-cli/src/policy_schema.rs`, `policy_scan.rs`, `policy_verify/v1.rs`, and `crates/mpk-api/src/policy_strategy.rs`, with policy/profile/schema tests, preserve and validate the complete Go/Rust tuple, strategy, recipe, proof route, and certificate linkage. | Certificate/checker trust remains source-neutral; registrations and validators are closed over Go/Rust. |
| AI | `crates/mpk-cli/src/ai_explain.rs` and `crates/mpk-api/src/vc_api.rs`, with `crates/mpk-cli/tests/ai_explain_v1.rs` and `ai_foundation.rs`, validate and redact evidence before untrusted explanation. | Prompt/projection/redaction behavior is reusable; accepted semantic tuples and strategies are Go/Rust-closed. |

The evidence establishes a reusable checked core, not a generic plugin model.
All current language matches remain deliberate fail-closed validators until a
reviewed successor schema replaces them atomically.

## 5. Structural and profile-record gap ledger

### 5.1 Registry and selection shape

| Gap | Exact missing shape | Current evidence | Owner and required disposition |
| --- | --- | --- | --- |
| `S01` | One successor semantic-identity shape and hash/version root for language, profile, semantic parameters, and target across VIR, VC, map, manifest, frontend protocol, release, policy, evidence, and AI. | Typed identity parsers and validators in `semantic_profile.rs` and every downstream schema accept only the exact Go/Rust pairs. | `MLANG-01-T02`: choose one closed tagged-union or hash-pinned compiled-registry revision, reject unknown/crossed identities, and define atomic Go/Rust migration. |
| `S02` | One successor selection shape, canonical selection ID, unit-kind ownership, and repetition rule for a C# compilation and selected source methods. | `ManifestSelection`, policy selection, source-map function IDs, and CLI selection paths encode Go package or Rust library forms. | `MLANG-01-T02`: freeze ownership and versioning of language-specific selection payloads without pre-registering C# content. |
| `S03` | One non-plugin compiler/frontend/toolchain/runtime descriptor and runner-dispatch shape capable of representing a managed Roslyn/.NET closure. | Release compiler identities, tuple validators, protocol branches, executable allowlists, sandbox paths, and issue markers are exact Go/Rust matches despite reusable inventory primitives. | `MLANG-01-T02`: freeze declarative descriptor limits and compiled validators; executable validator/checker callbacks remain forbidden. |
| `S04` | One closed consumer-registration shape for policy strategy, program-certificate eligibility, evidence recipe/profile linkage, and AI projection. | Policy/evidence/AI implementations dispatch on hard-coded Go/Rust language/profile/selection combinations. | `MLANG-01-T02`: freeze registration identity, unknown/cross-profile rejection, and atomic consumer migration; do not add C# entries yet. |

These four shape gaps are disjoint: `S01` owns semantic identity, `S02` owns
source selection, `S03` owns release/execution descriptors, and `S04` owns
post-VC consumer registration. Their exact C# records belong to T03.

### 5.2 Exact C# profile records

| Gap | Exact record still required | Semantic rows | Owner |
| --- | --- | --- | --- |
| `P01` | Language/compiler/runtime/architecture/target-framework/reference-assembly/options/nullable/preprocessor pins and their semantic-parameter projection. | `M33` | `MLANG-01-T03` |
| `P02` | Accepted fixed integer types; promotions; predefined operators; signedness; checked/unchecked context; conversions; and exact ordered safety-check matrix. | `M01`, `M02`, `M08`, `M09`, `M13`, `M14`, `M16`, `M18`, `M19`, `M21` | `MLANG-01-T03` |
| `P03` | Compilation/source/method selection grammar, normalized identifiers, overload identity, selected closure, and deterministic ordering. | none beyond the row mappings; this is a source-selection record | `MLANG-01-T03` |
| `P04` | Roslyn operation/CFG whitelist, left-to-right lowering, short-circuit/return lowering, definite assignment, direct-call sequencing, purity, acyclicity, and inert containing-type initialization proof. | `M07`, `M10`-`M12`, `M27`, `M29` | `MLANG-01-T03` |
| `P05` | MPK-owned C# contract grammar, source/sidecar attachment, type rules, pure operation set, normalization, limits, and diagnostics. | `M34` | `MLANG-01-T03` |
| `P06` | Exact source encoding and Roslyn span-to-UTF-8-byte mapping, line/column rules, generated-source/directive admission or rejection, diagnostic locale/content, and total origin mapping. | none; source-map contract record | `MLANG-01-T03` |
| `P07` | Exact manifest source/contract/reference/toolchain input closure, language configuration, staged hash ownership, and no-project/no-restore/no-ambient-reference rules. | `M33` | `MLANG-01-T03` |
| `P08` | Exact `csharp2vir`/Roslyn/.NET invocation, arguments, closed environment, mounts, managed/native runtime inventory, resource limits, output protocol, and status precedence. | `M33` | `MLANG-01-T03` |
| `P09` | C# policy strategy, required checks, axiom report, program-certificate eligibility, evidence linkage, and structured reproduction recipe, with no new axiom category. | all accepted rows through their emitted evidence | `MLANG-01-T03` |
| `P10` | C# AI projection registration, profile-aware labels, redaction, and source-free prompt boundary; AI remains unable to affect acceptance. | none; AI contract record | `MLANG-01-T03` |

No `P` record can be frozen before T02 chooses the successor shape. No `P`
record requires a new checked carrier or theory for the 18 candidate rows.

## 6. Complete C# semantic-row ledger

The `T01/T02` column preserves the prior feasibility disposition. The class in
this table is the result of inspecting the implementation. Every row has one
class and one owner.

| Row | T01/T02 | Implemented gap decision | Class | Exact owner |
| --- | --- | --- | --- | --- |
| `M01` | `E` / `GO` | Bool literals, parameters, locals, results, and `Const` exist; T03 must admit only exact C# `bool`. | `P` (`P02`) | `MLANG-01-T03` |
| `M02` | `E` / `GO` | Signed/unsigned BV8/16/32/64 carriers exist; T03 must choose source types and promotion boundaries. | `P` (`P02`) | `MLANG-01-T03` |
| `M03` | `R` / `NO-GO` | No target-sized carrier is needed for the initial profile; `nint` and `nuint` reject. | `U` | `MLANG-01-T03` rejection set |
| `M04` | `F` / `NO-GO` | Binary floating carriers, operations, rounding, NaN/infinity/signed-zero policy, and checked encoding are absent. | `F` | `MLANG-01-T03` rejection set; future named foundation task only |
| `M05` | `F` / `NO-GO` | The C# `decimal` carrier, scale/rounding/arithmetic/conversion semantics, and checked encoding are absent. | `F` | `MLANG-01-T03` rejection set; future named foundation task only |
| `M06` | `R` / `NO-GO` | C# aggregates and arrays require held-out value/heap behavior; no approximation is permitted. | `U` | `MLANG-01-T03` rejection set |
| `M07` | `E` / `GO` | Ordered instructions and CFG already preserve accepted pure left-to-right evaluation. | `P` (`P04`) | `MLANG-01-T03` |
| `M08` | `E` / `GO` | Existing exact-Bool not/equality operations suffice after predefined-operator resolution. | `P` (`P02`) | `MLANG-01-T03` |
| `M09` | `E` / `GO` | Existing BV equality and signed/unsigned comparisons suffice after exact type/promotion resolution. | `P` (`P02`) | `MLANG-01-T03` |
| `M10` | `E` / `GO` | Existing `Branch` graphs express exact-Bool short circuiting. | `P` (`P04`) | `MLANG-01-T03` |
| `M11` | `E` / `GO` | Existing `Branch`, `Jump`, and `Return` express conditionals, joins, and early return. | `P` (`P04`) | `MLANG-01-T03` |
| `M12` | `E` / `GO` | Existing `Copy` plus dominance validation expresses explicitly assigned scalar locals. | `P` (`P04`) | `MLANG-01-T03` |
| `M13` | `E` / `GO` | Total BV add/sub/mul/neg already provide same-width unchecked wrapping behavior. | `P` (`P02`) | `MLANG-01-T03` |
| `M14` | `C` / `GO` | Existing BV operations, `integer_no_overflow`, proposition regeneration, and checked proof routes suffice; T03 must map exact C# checked context. | `P` (`P02`) | `MLANG-01-T03` |
| `M15` | `R` / `NO-GO` | No arbitrary-precision primitive is admitted; no bounded-width substitution is allowed. | `U` | `MLANG-01-T03` rejection set |
| `M16` | `C` / `GO` | Existing signed/unsigned div/rem, `divisor_nonzero`, and `signed_divrem_representable` checks cover the candidate failure exclusions. | `P` (`P02`) | `MLANG-01-T03` |
| `M17` | `R` / `NO-GO` | Floor-division/modulo semantics are not a C# primitive candidate. | `U` | `MLANG-01-T03` rejection set |
| `M18` | `E` / `GO` | Existing BV not/and/or/xor operations suffice after exact operand resolution. | `P` (`P02`) | `MLANG-01-T03` |
| `M19` | `E` / `GO` | Existing `bv_and` and shifts express C#'s width-minus-one count masking. | `P` (`P02`) | `MLANG-01-T03` |
| `M20` | `R` / `NO-GO` | Unbounded shifts are outside the fixed-BV profile. | `U` | `MLANG-01-T03` rejection set |
| `M21` | `E` / `GO` | Total BV `Convert` exists. Its current Go-only validator restriction becomes a C# profile-admission rule after successor activation, not a new operation. | `P` (`P02`) | `MLANG-01-T03` |
| `M22` | `F` / `NO-GO` | Foundations can express range predicates, but current `Convert` requires no checks and the closed safety vocabulary has no checked-conversion operation. | `V` | `MLANG-01-T03` rejection set; future named successor-operation task only |
| `M23` | `R` / `NO-GO` | Dynamic, user-defined, boxing, nullable, and runtime conversion protocols execute behavior outside the candidate core. | `U` | `MLANG-01-T03` rejection set |
| `M24` | `R` / `NO-GO` | Null/reference/nullable carriers and dereference behavior are absent and remain rejected. | `U` | `MLANG-01-T03` rejection set |
| `M25` | `R` / `NO-GO` | Allocation, identity, aliasing, fields/properties, and mutation require a heap model. | `U` | `MLANG-01-T03` rejection set |
| `M26` | `R` / `NO-GO` | General exception, handler, finally, and abrupt-completion propagation are not represented. | `U` | `MLANG-01-T03` rejection set |
| `M27` | `C` / `GO` | Implemented `CallStatic`, contract hashes, WP, and acyclic call validation suffice for a proved pure same-module static closure. | `P` (`P04`) | `MLANG-01-T03` |
| `M28` | `R` / `NO-GO` | Virtual/interface/delegate/dynamic/operator dispatch remains unsupported. | `U` | `MLANG-01-T03` rejection set |
| `M29` | `C` / `GO` | Dominance/definition validation and explicit `Copy` suffice after Roslyn and MPK prove definite assignment. | `P` (`P04`) | `MLANG-01-T03` |
| `M30` | `R` / `NO-GO` | Field defaults, type initialization, and static constructors remain unsupported; `M27` may use only a proved inert initialization closure. | `U` | `MLANG-01-T03` rejection set |
| `M31` | `R` / `NO-GO` | Threads, locks, shared memory, atomics, and .NET memory-model behavior remain unsupported. | `U` | `MLANG-01-T03` rejection set |
| `M32` | `R` / `NO-GO` | Tasks, async/await, iterators, suspension, and continuations remain unsupported. | `U` | `MLANG-01-T03` rejection set |
| `M33` | `C` / `GO` | Existing release/manifest hash closure and semantic parameters are reusable; exact Roslyn/.NET/target/options pins remain to freeze. | `P` (`P01`) | `MLANG-01-T03` |
| `M34` | `C` / `GO` | `VirContract`, contract expressions, call/WP integration, and VC encoding exist; C# grammar and attachment remain to freeze. | `P` (`P05`) | `MLANG-01-T03` |

## 7. Required contract-surface coverage

| Required surface | Reusable implementation | Classified gaps and owner |
| --- | --- | --- |
| VIR | Bool/BV carriers, ordered CFG, total operations, `Convert`, `CallStatic`, contracts, and exact safety metadata. | `S01`/T02 for identity; `P01`, `P02`, `P04`, `P05`/T03 for C# rules; `M22` is `V`, `M04`/`M05` are `F`, and all `U` rows reject. |
| VC | Source-neutral WP, contract/call obligations, profile-derived safety propositions, skeletons, and checked Bool/BV foundations. | `S01`/T02 for identity; `P02`, `P04`, `P05`/T03. No candidate-row foundation gap. |
| Source map | Normalized source refs/origins, paths, hashes, spans, and total mapping. | `S01` and `S02`/T02 for identity/selection shape; `P03` and `P06`/T03 for C# IDs and coordinates. |
| Manifest | Staged source/final records, complete hash linkage, release identity, and deterministic input inventory. | `S01`-`S03`/T02 for identity, selection, and descriptor shape; `P03` and `P07`/T03 for exact C# records. |
| Runner | Installed-registry resolution, immutable snapshots, bounded protocol streams, closed environment, and sandbox lifecycle. | `S03`/T02 for descriptor/dispatch shape; `P08`/T03 for exact managed-runtime execution. |
| Selection | One canonical language-owned selection repeated through current artifacts. | `S02`/T02 for successor payload ownership; `P03`/T03 for C# compilation/method content. |
| Release | Content-addressed inventories, compiler identity concept, runtime/native-runtime closure, tuple hashing, and installed-tree checks. | `S03`/T02 for declarative managed-runtime shape; `P01`, `P07`, `P08`/T03 for exact pins and files. |
| Policy | Explicit tuple validation, closed strategies, required-check evaluation, status precedence, program-certificate route, and reproduction recipe. | `S01`, `S02`, `S04`/T02 for closed registration shape; `P09`/T03 for C# strategy content. |
| Evidence | Source-neutral proof/certificate/checker linkage, axiom reporting, profile identity, and structured recipes. | `S01`, `S02`, `S04`/T02 for identity/registration shape; `P09`/T03. Certificate v0 and four axiom categories do not change. |
| AI | Validated evidence projection, redaction, bounded prompt, and untrusted explanatory output. | `S01`, `S04`/T02 for registration shape; `P10`/T03 for labels/redaction. AI receives no authority or source access. |

## 8. Previously blocked C# questions

| Question | Audit disposition | Owner after this audit |
| --- | --- | --- |
| `Q-CS-01` | Still profile-only. T03 must choose exact default/explicit checked-context admission and ordered checks. `M22` stays rejected regardless. | `P01`/`P02`, `MLANG-01-T03` |
| `Q-CS-02` | Still profile-only. Existing BV operations are sufficient, but no source type, promotion, `char`, or result-conversion default is inferred. | `P02`, `MLANG-01-T03` |
| `Q-CS-03` | Still profile-only after the feasible public Roslyn boundary. T03 must freeze every package/runtime/TFM/reference/architecture/option pin. | `P01`, `P07`, `P08`, `MLANG-01-T03` |
| `Q-X-01` | Resolved for the initial C# profile: no checked-conversion foundation is justified or added. `M22` is a `V` gap and rejects. | Rejection: `MLANG-01-T03`; later admission: a new named task only |
| `Q-X-02` | `CallStatic` is implemented and profile-only; T03 decides whether the first C# subset admits it and, if so, freezes the exact closed call/purity/init rules. | `P04`, `MLANG-01-T03` |
| `Q-X-03` | Contracts are implemented and profile-only; T03 must freeze the C# grammar and attachment. | `P05`, `MLANG-01-T03` |

## 9. Exit evidence and handoff

The ledger closes `MLANG-01-T01` because:

- all 34 C# semantic rows occur exactly once and have one class;
- all 18 T02 `GO` rows are profile-only against the implemented core;
- all 16 T02 `NO-GO` rows are exactly one `V`, two `F`, or 13 `U` gaps;
- the four structural gaps are each one `S` record with T02 ownership;
- every required VIR, VC, source-map, manifest, runner, selection, release,
  policy, evidence, and AI surface has an explicit reuse decision and gap
  owner;
- no candidate row needs a new checked foundation/theory, and no gap has an
  ambiguous owner; and
- no production source, schema, vector, registry, executable, or bundle was
  changed.

The next task is `MLANG-01-T02`: freeze one successor extension mechanism and
the atomic Go/Rust migration. `MLANG-01-T03` and all C# production work remain
blocked behind it.
