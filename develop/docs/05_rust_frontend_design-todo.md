# Rust Verification Frontend and Unified VIR Migration Todo

Source design: `develop/docs/05_rust_frontend_design.md`

Status: implementation-ready task breakdown

This document turns the breaking GIR-to-VIR migration and Rust frontend design
into independently implementable milestones. Source-design phase IDs remain
`VIR-00`, `VIR-01`, `GO-VIR-02`, and `RUST-03` through `RUST-07`;
implementation milestones add a `-Tnn` suffix so one implementation pass can
select exactly one bounded unit of work.

## Scope

In scope:

- freeze every versioned contract needed by the shared Go/Rust helper path;
- build the language-neutral VIR, VC v1, source-map, manifest, release-bundle,
  policy v1, and AI v1 foundations;
- migrate the accepted Go subset and all downstream consumers from GIR to VIR
  in one atomic cutover;
- add a pinned, fail-closed, untrusted Rust frontend over checked MIR;
- generate property, call, loop, and runtime-safety obligations and group them
  into deterministic contract and panic-free declarations;
- attach traceability metadata without expanding the proof trust boundary;
- prove the resulting certificate corpus with both source-free checkers;
- complete negative, adversarial, deterministic, fuzz, differential, CI, and
  release gates for both migrated Go and Rust.

Out of scope:

- runtime compatibility with GIR v0, old policy payloads, or the AI API v0 GIR
  route after the cutover;
- changing certificate v0 binary encoding, the core calculus, definitional
  equality, checker semantics, or axiom-category encoding;
- treating source, compiler output, frontend success, VIR, VC JSON, source
  maps, policy prose, solver answers, or CI status as proof evidence;
- Rust features excluded by `mpk.rust.checked.v0`, including references,
  pointers, allocation, traits, generics, enums, loops, recursion, external
  dependencies, macros, build scripts, floating point, and 128-bit integers;
- mixed-language VIR modules or cross-language calls in v0.

## Execution Rules

1. `VIR-00` is a hard specification gate. No serialized producer or consumer
   may be merged before the specification that owns its bytes, hashes, limits,
   failure classes, and canonical test vectors is frozen.
2. `VIR-01` may add internal VIR code alongside the current GIR implementation,
   but it must not expose a second production CLI input. The released path
   remains Go/GIR until `GO-VIR-02-T12` completes the atomic cutover.
3. `GO-VIR-02-T12` is the only milestone allowed to remove the active GIR path.
   It updates producers, consumers, fixtures, examples, scripts, and active
   user documentation in the same change. There is no merge state in which the
   public CLI accepts both GIR and VIR, and no `mpk.gir.v1` successor is
   introduced.
4. Rust work starts only after the Go cutover gate passes. Rust must use the
   already-active generic runner and VIR/VC/policy v1 interfaces; it must not
   add a Rust-only route through `mpk-cli` or `mpk-vc`, parse textual
   `--emit=mir` output, or replace the pinned rustc callback/query boundary.
5. Every parser denies unknown fields and duplicate object names, applies its
   byte/count/depth limits before full allocation where specified, and returns
   a deterministic structured error. Unknown semantics always reject.
6. All canonical JSON follows the narrowed RFC 8785 rules in the source design.
   Object-name ordering follows RFC 8785; schema-declared unordered arrays are
   normalized separately using their specification-defined order. Transport LF
   bytes never enter a nested hash preimage.
7. Every hash implementation has domain-separation vectors for empty, minimal,
   normal, boundary, non-ASCII-key, and mutation cases. Producers and consumers
   recompute rather than trust repeated hashes.
8. Evidence-producing routes accept registered bundle IDs only. Raw frontend,
   helper, driver, compiler, toolchain, registry-path, or sandbox-path flags are
   not public escape hatches.
9. A source frontend may return `ir-lowered` even when its generated safety or
   property obligations are not yet proved. Non-strict policy output then stays
   `proof_pending`; strict verification fails without relabeling the source as
   rejected.
10. Each milestone preserves a green default branch, updates directly affected
    documentation and fixtures, and stages only its own files.
11. Before RUST-03-T12, Rust bundle tests use only the unregistered injected
    candidate inventories from RUST-03-T01. RUST-03-T12 performs the first
    release registration. After that point, any milestone that changes bytes
    of `rust2vir`, `rust2vir-driver`, the pinned toolchain, or their release
    build inputs must atomically rebuild the complete bundles, update and
    review the registry root, rebuild `mpk-cli` with that root, validate the
    installed tree, and rerun the registered Go path. A tracked descriptor may
    never describe an older build of the current registered sources.

## Component Ownership

| Concern | Owner after cutover | Required boundary |
| --- | --- | --- |
| Strict JSON, JCS, hash helpers used by Rust consumers | `mpk-vc` | Untrusted helper code; no certificate acceptance authority. |
| VIR, semantic profiles, contracts, source map, source manifest, VC v1 | `mpk-vc` | Strictly validated untrusted artifacts. |
| Release registry parsing and bundle inventory models | `mpk-vc` | Portable data validation only. |
| Installed bundle resolution, immutable snapshots, process limits, frontend protocol consumption | `mpk-cli` | No user-selected executable path on evidence routes. |
| Go snapshot, source gate, contract parsing, SSA lowering, envelope emission | `go-tools/go2vir` | Separate untrusted Go executable. |
| Rust preflight, Cargo orchestration, contract parsing, envelope emission | `rust-tools/rust2vir` main binary | Isolated pinned project, outside the root Cargo workspace. |
| rustc callbacks, HIR/MIR validation and raw lowering | `rust2vir-driver` | Private `mpk.rust.driver.v0` protocol; never a public evidence format. |
| Policy scan, verification orchestration, evidence, report, explainer | `mpk-cli` | JSON is the stable product API; Markdown and AI output are derived helper views. |
| Strategy tuple metadata | `mpk-api` | Keeps strategy, checker, semantic, and axiom profiles distinct. |
| Certificate checking | Existing `mpk-kernel` and Go reference checker | Source-free and unchanged. |

The target Rust-side APIs are named consistently throughout the tasks:

```text
import_vir_json
validate_vir
canonical_vir_json
vir_hash
validate_source_map
validate_source_manifest
generate_program_vcs
emit_grouped_theorem_skeleton
```

Exact signatures and error enums are frozen by the corresponding `VIR-00`
specification milestone before implementation. The old exported
`import_gir_json`, `generate_straight_line_vcs`, `generate_branch_vcs`,
`generate_loop_vcs`, and `generate_safety_vcs` APIs remain active only until the
atomic Go cutover and are then removed rather than deprecated.

## Phase Mapping and Critical Path

| Source-design phase | Implementation milestones | Phase exit |
| --- | --- | --- |
| VIR-00 | VIR-00-T01 through VIR-00-T10 | All bytes, semantics, limits, status precedence, and migration removals are normative and testable. |
| VIR-01 | VIR-01-T01 through VIR-01-T12 | Shared VIR/VC foundations handle both profiles and all required checks without a Rust semantic axiom. |
| GO-VIR-02 | GO-VIR-02-T01 through GO-VIR-02-T12 | Go uses VIR exclusively; GIR-era active interfaces are absent. |
| RUST-03 | RUST-03-T01 through RUST-03-T12 | Basic Rust functions emit deterministic valid frontend envelopes without executing user build code. |
| RUST-04 | RUST-04-T01 through RUST-04-T05 | Checked arithmetic and modeled panic conditions produce complete safety VCs. |
| RUST-05 | RUST-05-T01 through RUST-05-T05 | Aggregates and acyclic contracted calls produce stable grouped declarations. |
| RUST-06 | RUST-06-T01 through RUST-06-T04 | Rust policy example produces evidence and a certificate accepted by both checkers. |
| RUST-07 | RUST-07-T01 through RUST-07-T05 | Adversarial, determinism, fuzz, CI, upgrade, and release gates pass from a clean checkout. |

The critical path is:

```text
VIR-00-T01..T10
  -> VIR-01-T01..T12
  -> GO-VIR-02-T01..T12
  -> RUST-03-T01..T12
  -> RUST-04-T01..T05
  -> RUST-05-T01..T05
  -> RUST-06-T01..T04
  -> RUST-07-T01..T05
```

Tasks within a phase may run in parallel only when their explicit dependencies
allow it. A phase exit gate still requires every task in that phase.

## Common Definition of Done

Every milestone satisfies all applicable items below in addition to its own
acceptance criteria:

- public schemas and error codes have positive, negative, unknown-field,
  duplicate-key, wrong-schema, boundary-limit, and deterministic-byte tests;
- trusted and untrusted artifacts are labeled consistently with
  `develop/specs/TRUST_BOUNDARY_V0.md`;
- unsupported input fails closed and no operational failure is converted into
  a successful, ready, verified, or proof-pending result;
- canonical outputs contain no absolute source, toolchain, workspace, home, or
  temporary path and no timestamp, hostname, random ID, or raw compiler prose;
- test fixtures are generated only through an explicit update mode and ordinary
  tests compare without rewriting tracked files;
- Rust workspace changes pass `cargo fmt --all -- --check`, targeted tests, and
  `cargo clippy --workspace --all-targets -- -D warnings` when the touched code
  is in the root workspace;
- Go frontend changes pass `go test -count=1 ./...` in the affected Go module;
- isolated Rust frontend changes pass `cargo fmt --all -- --check` and
  `cargo test --locked` from `rust-tools/rust2vir` under its pinned toolchain;
- documentation changes pass `git diff --check` and contain no unresolved
  design placeholders;
- the default release path remains source-free for proof checking and both
  checker verdicts remain the only accepted checker evidence.

## Milestones

### VIR-00-T01 Freeze the GIR Baseline and Removal Inventory

Status: Pending

Depends on: none.

Inputs:

- design sections 1-6, 15.1, 19.4, 20, and 21;
- `develop/specs/GIR_V0.md`, `develop/specs/GO_SUBSET_V0.md`, and
  `develop/specs/AI_API_V0.md`;
- `crates/mpk-vc/src`, `crates/mpk-cli/src`, `crates/mpk-api/src`,
  `go-tools/go2gir`, examples, fixtures, scripts, and active documentation.

Likely touched files:

- `develop/migrations/gir-to-vir-inventory.md`
- `develop/migrations/gir-to-vir-obsolete-terms.txt`
- `develop/migrations/go-gir-semantic-baseline.json`
- `scripts/check-no-active-gir.sh`

Tasks:

1. Inventory every GIR schema, hash domain, binary wrapper, Rust/Go type,
   public export, CLI flag, fixture field, helper-kind enum, AI route, prompt
   schema, documentation example, and release-script assumption listed in
   design section 19.4. The exact obsolete-schema list includes
   `mpk.gir.v0`, `mpk.gir.emit.v0`, `mpk.go2gir.cli.v0`,
   `mpk.go.source_manifest.v0`, `mpk.vc.cert_skeleton.v0`,
   `mpk.policy.scan.v0`, `mpk.policy.evidence.v0`,
   `mpk.ai.explain.request.v0`, `mpk.ai.explanation.v0`, and
   `mpk.evidence-explainer.v0`.
2. Classify each hit as remove, rename/regenerate, or historical-allowlist.
   Historical hits are limited to frozen v0 specs, the checked-in migration
   report, and the exact migration-design records
   `develop/docs/05_rust_frontend_design.md` and
   `develop/docs/05_rust_frontend_design-todo.md`. The allowlist records exact
   paths, not broad directories, and accepts the two design paths only after
   GO-VIR-02-T12 adds a status marker saying the Go cutover is complete, Rust
   phases remain active, and every retained old identifier is historical
   migration terminology.
3. Capture the pre-cutover positive and negative Go corpus, obligation kinds,
   theorem intent, contract and loop behavior, checker verdicts, and hashes in a
   machine-readable semantic baseline. Hash bytes are recorded for audit but
   are explicitly not expected to remain equal.
4. Add a search gate that fails on an obsolete term outside the exact
   historical allowlist and supports a pre-cutover audit mode plus a strict
   post-cutover mode.
5. Record the owner milestone for every removal so the final cutover cannot
   close with an unassigned item.

Deliverables:

- complete, reviewed migration inventory and semantic baseline;
- executable obsolete-interface search gate with an exact historical allowlist.

Acceptance criteria:

- every design-section 19.4 term appears in the inventory;
- every current production hit has one removal owner;
- the baseline covers all 100 Go alpha functions, five positive payment-policy
  examples, current negative corpora, loops, conversions, and runtime checks;
- running strict mode before cutover fails, proving the gate detects the
  current GIR path.

Verification:

```sh
./scripts/check-no-active-gir.sh --audit
cargo test -p mpk-vc --test alpha_corpus
(cd go-tools/go2gir && go test -count=1 ./...)
git diff --check
```

Notes:

- This milestone records behavior; it does not change the active GIR path.

### VIR-00-T02 Freeze VIR v0 and the Two Semantic Profiles

Status: Pending

Depends on: VIR-00-T01.

Inputs:

- design sections 9, 10, 11, 12, 16, and 18;
- the baseline from VIR-00-T01;
- `develop/specs/GIR_V0.md` and `develop/specs/GO_SUBSET_V0.md`.

Likely touched files:

- `develop/specs/VIR_V0.md`
- `develop/specs/vectors/vir-v0.json`
- `develop/specs/vectors/vir-hash-v0.json`

Tasks:

1. Freeze exact JSON field names, tagged-union discriminators, required and
   forbidden fields, ordering, identifier grammar, type declarations,
   constants, values, blocks, terminators, contracts, and `safety_checks` for
   `mpk.vir.v0`.
2. Copy the total fixed-size bitvector equations into the spec, including zero
   divisors, signed minimum divided or remainder by negative one, full-width
   shift counts, and signed versus unsigned right shift.
3. Freeze `mpk.go.fixed.v0` and `mpk.rust.checked.v0` language/profile pairing,
   semantic-parameter shapes, operation/type matrix, exact required check set,
   extra-check rejection, target-width behavior, CFG/call-graph rules, and loop
   policy.
4. Freeze normalized contract representation and `MPK-CONTRACT-0.1`, including
   aggregate equality, ordered expression trees, callee hash binding, and raw
   input hash separation.
5. Freeze identifier traversal/renaming, all shared deterministic limits, and
   `MPK-VIR-0.1` hash input bytes.
6. Add valid and invalid conformance vectors for every instruction,
   terminator, type, safety check, profile difference, boundary, and hash
   mutation.

Deliverables:

- normative `VIR_V0.md` and cross-language conformance vectors.

Acceptance criteria:

- every accepted current Go operation and proposed Rust operation has exactly
  one value semantics, profile-required check set, contract rule, and rejection
  rule;
- no schema choice remains delegated to a frontend implementation;
- vectors distinguish Go wrapping from Rust checked overflow and Go from Rust
  over-width shifts.

Verification:

```sh
rg -n "mpk\.go\.fixed\.v0|mpk\.rust\.checked\.v0|MPK-VIR-0\.1|MPK-CONTRACT-0\.1" develop/specs/VIR_V0.md
rg -n "[T]ODO|[T]BD|未[定]|[P]LACEHOLDER" develop/specs/VIR_V0.md develop/specs/vectors/vir-v0.json develop/specs/vectors/vir-hash-v0.json
git diff --check
```

### VIR-00-T03 Freeze Release Registry and Bundle Installation Contracts

Status: Pending

Depends on: VIR-00-T02.

Inputs:

- design sections 5.2, 8.1, 13, 14, 18, and 23;
- VIR profile and target identifiers from VIR-00-T02.

Likely touched files:

- `develop/specs/RELEASE_BUNDLES_V0.md`
- `develop/specs/vectors/release-bundles-v0.json`
- `release/bundles/bundle-registry.json`
- `release/bundles/README.md`

Tasks:

1. Freeze exact registry, frontend-bundle, toolchain-bundle, component, and
   canonical inventory schemas as `mpk.release.bundle_registry.v0`,
   `mpk.release.frontend_bundle.v0`, and
   `mpk.release.toolchain_bundle.v0`, including uniqueness rules, tuple keys,
   sort order, the `distribution_sha256` field, and
   `MPK-BUNDLE-REGISTRY-0.1`/`MPK-BUNDLE-CONTENT-0.1` preimages.
2. Freeze the installation layout: a release root contains `bin/mpk`, the
   installed registry at `share/mpk/bundle-registry.json`, and bundles at
   `libexec/mpk/bundles/BUNDLE_ID`. The runner derives that root only from the
   already opened `mpk` executable, opens the registry without following
   links, and compares its canonical ID/hash with build-embedded expected
   constants generated from `release/bundles/bundle-registry.json`.
   Environment, project, adjacent files outside that exact location, and user
   CLI values cannot replace the installed registry or root algorithm.
3. Freeze open-before-hash, immutable-handle, executable-bit, regular-file,
   link/reparse-point, hard-link-alias, unlisted-file, and complete-inventory
   validation rules.
4. Define build-time checks that recompute the source registry hash and embed
   only the expected registry ID/hash constants without trusting hand-copied
   values; runtime still validates the separately installed bytes and can
   report a missing registry.
5. Define a test-only dependency-injected bundle resolver that cannot be
   enabled in non-test builds; production evidence routes never accept paths.
6. Add canonical vectors for missing, extra, duplicate, reordered, mutated,
   oversized, and unsupported tuple cases.

Deliverables:

- normative release-bundle contract, initial canonical registry, and vectors.

Acceptance criteria:

- a caller can select only registered IDs compatible with the exact
  language/profile/target tuple;
- changing any descriptor, inventory entry, executable, library, or target
  standard library changes the validated registry or content identity;
- the specification leaves no search-path or environment fallback.

Verification:

```sh
rg -n "MPK-BUNDLE-REGISTRY-0\.1|MPK-BUNDLE-CONTENT-0\.1|bundle_registry|frontend_bundle|toolchain_bundle" develop/specs/RELEASE_BUNDLES_V0.md
python3 -m json.tool release/bundles/bundle-registry.json
git diff --check
```

### VIR-00-T04 Freeze the Frontend, Source Map, and Source Manifest Protocols

Status: Pending

Depends on: VIR-00-T02, VIR-00-T03.

Inputs:

- design sections 7, 8.3, 12.4-12.5, 13, 14, 17, and 18.

Likely touched files:

- `develop/specs/FRONTEND_PROTOCOL_V0.md`
- `develop/specs/SOURCE_MAP_V0.md`
- `develop/specs/SOURCE_MANIFEST_V0.md`
- `develop/specs/vectors/frontend-protocol-v0.json`
- `develop/specs/vectors/source-map-v0.json`
- `develop/specs/vectors/source-manifest-v0.json`

Tasks:

1. Freeze exact `mpk.frontend.cli.v0` status-tagged payloads, language-specific
   selection union, allowed artifact presence, status/exit pairs, stdout/LF
   rules, strict canonical parsing, repeated identities, and phase precedence.
2. Freeze the exact distinction among CLI configuration error, `rejected`,
   `source-error`, and `frontend-error`, including consumer classification for
   missing, killed, truncated, malformed, noncanonical, and oversized children.
   Noncanonical protocol bytes use `FRONTEND_PROTOCOL_NONCANONICAL`.
3. Freeze `mpk.source_map.v0` reference tags, ordering, total mapping rules,
   UTF-8 ranges, input-kind linkage, synthetic-node policy,
   `MPK-SOURCE-MAP-0.1` hash preimage, and limits.
4. Freeze `mpk.source_manifest.v0`, its language-specific configuration union,
   release/toolchain/frontend identities, units, inputs, `MPK-INPUT-SET-0.1`,
   and the existing `MPK-SOURCE-MANIFEST-0.1` domain.
5. Define frontend-stage and certificate-stage manifest lifecycle validation:
   final assembly may add only `vc_hash` and recompute
   `source_manifest_hash`.
6. Freeze normalized diagnostic fields, source spans, sort/truncation behavior,
   stable code ownership, and raw compiler/stderr exclusion.
7. Add status, identity-mismatch, path-leak, mapping, lifecycle, and exact-limit
   vectors.

Deliverables:

- three normative shared protocol specs and conformance vectors.

Acceptance criteria:

- every status has one exact exit, payload shape, phase, and artifact-presence
  rule;
- source map and manifest hashes can be recomputed without original source;
- no public canonical field can contain an absolute machine path.

Verification:

```sh
rg -n "ir-lowered|source-error|frontend-error|MPK-SOURCE-MAP-0\.1|MPK-INPUT-SET-0\.1|MPK-SOURCE-MANIFEST-0\.1" develop/specs/FRONTEND_PROTOCOL_V0.md develop/specs/SOURCE_MAP_V0.md develop/specs/SOURCE_MANIFEST_V0.md
git diff --check
```

### VIR-00-T05 Freeze VC v1 and Grouped Declaration Semantics

Status: Pending

Depends on: VIR-00-T02, VIR-00-T04.

Inputs:

- design sections 10.5, 16, and 18;
- current `crates/mpk-vc/src/vc.rs` and `obligation_emit.rs`.

Likely touched files:

- `develop/specs/VC_V1.md`
- `develop/specs/vectors/vc-v1.json`
- `develop/specs/vectors/vc-hash-v1.json`
- `develop/specs/vectors/vc-skeleton-v1.json`

Tasks:

1. Freeze exact `mpk.vc.v1` and `mpk.vc.cert_skeleton.v1` schemas with
   `schema`, source IR/input/profile/limit identities, self-hash fields, member
   obligations, theorem groups, and declaration dependencies.
2. Freeze source-language-neutral obligation IDs and kinds for postcondition,
   callee precondition, loop initialization/preservation/exit/decreases,
   operation safety, and callee panic-free obligations.
3. Freeze exhaustive partitioning into exactly one `FUNCTION_ID.contract` or
   `FUNCTION_ID.panic_free` group and require policy member rows to bind to the
   containing declaration name and hash.
4. Freeze parameter binders, outer and member implications, balanced ordered
   conjunction, empty/singleton cases, group/dependency sort order, callee-first
   topological order, and contract-before-panic-free order.
5. Freeze `MPK-VC-1.0`, repeated-field validation, skeleton linkage, streaming
   counters, exact `verification_limit_profile = mpk.verify.limits.v0`, and
   downstream resource-limit precedence.
6. Add vectors for missing, duplicate, ungrouped, wrongly grouped, reordered,
   cyclic, extra-edge, hash-mismatched, and boundary-depth cases.

Deliverables:

- normative VC/skeleton v1 contract and complete grouping vectors.

Acceptance criteria:

- every obligation has one group and every generated dependency edge is both
  necessary and sufficient under design section 10.5;
- equivalent but differently associated propositions cannot change the
  canonical theorem type;
- the old `schema_version` and `source_gir_hash` spellings are explicitly
  invalid in v1.

Verification:

```sh
rg -n "mpk\.vc\.v1|mpk\.vc\.cert_skeleton\.v1|MPK-VC-1\.0|panic_free|balanced" develop/specs/VC_V1.md
git diff --check
```

### VIR-00-T06 Freeze Policy v1, Profile Tuples, and Reproduction Recipes

Status: Pending

Depends on: VIR-00-T03, VIR-00-T04, VIR-00-T05.

Inputs:

- design section 15;
- current `policy_scan.rs`, `policy_evidence.rs`, `policy_verify.rs`, and
  `policy_report.rs`.

Likely touched files:

- `develop/specs/POLICY_V1.md`
- `develop/specs/vectors/policy-scan-v1.json`
- `develop/specs/vectors/policy-evidence-v1.json`
- `develop/specs/vectors/policy-recipes-v1.json`

Tasks:

1. Freeze exact `mpk.policy.scan.v1`/`mpk.policy.evidence.v1` top-level
   identities, the shared selection union, release registry, frontend,
   toolchain, manifest lifecycle hashes, source IR/VC identities, helper
   kinds, contract raw/normalized hashes, trusted evidence, properties, and
   unknown-field behavior.
   The lifecycle field names are exactly `frontend_source_manifest_hash` and
   `certificate_source_manifest_hash`; only evidence also carries `vc_hash`.
2. Freeze the strategy registry tuples for Go and Rust and keep semantic,
   strategy, checker, and axiom profiles independent fields. A crossed known
   tuple rejects before frontend launch.
3. Freeze explicit `axiom_profile`, removal of
   `mpk.policy.evidence.v0`'s `allowed_axiom_profiles`, and the
   package/release cross-check ownership.
4. Freeze the generic CLI option set and validation precedence, including
   mandatory `--language`, `--semantic-profile`,
   `--require-release-registry-id`, `--require-release-registry-sha256`,
   `--frontend-bundle`, `--toolchain-bundle`, `--target`, `--package`,
   `--function`, repeatable `--contract`, and route output options. Verification
   additionally freezes `--strategy-profile`, `--checker-profile`, mandatory
   `--axiom-profile`, `--strict`, and `--update-fixtures`. Reject raw
   `--frontend`, `--frontend-helper`, `--driver`, toolchain-root, and registry-
   path options.
5. Freeze exactly one scan and one verify structured recipe, canonical argv
   order, relative source root `.`, normalized contract order, fixed output
   names, safe fixture-update behavior, and POSIX rendering.
6. Freeze policy/member/declaration consistency and the rule that
   `mpk_verified` requires the containing accepted declaration and all
   transitive generated declaration dependencies.

Deliverables:

- normative policy v1 schemas, CLI contract, and vectors.

Acceptance criteria:

- neither schema contains Go-only target fields, GIR fields, a free-form shell
  command, or an implicit axiom allowlist;
- scan does not claim checker/strategy/axiom selections it has not used;
- recipes contain no machine-local path or unresolved substitution.

Verification:

```sh
rg -n "mpk\.policy\.scan\.v1|mpk\.policy\.evidence\.v1|reproduction_recipes|axiom_profile|working_directory_role" develop/specs/POLICY_V1.md
git diff --check
```

### VIR-00-T07 Freeze AI API v1 and AI Explanation v1

Status: Pending

Depends on: VIR-00-T05, VIR-00-T06.

Inputs:

- design sections 15.1-15.2;
- `develop/specs/AI_API_V0.md`;
- `crates/mpk-cli/src/ai_explain.rs` and
  `docs/vertex-ai-gemini-assistant-design.md`.

Likely touched files:

- `develop/specs/AI_API_V1.md`
- `develop/specs/AI_EXPLAIN_V1.md`
- `develop/specs/vectors/ai-api-v1.json`
- `develop/specs/vectors/ai-explain-v1.json`

Tasks:

1. Copy unchanged session, term, proof, and non-import VC operations into API
   v1 and replace only the active import boundary with `POST /vir/import` over
   canonical validated VIR and VC v1 identities.
2. State that `POST /gir/import` is an unknown route and that no endpoint can
   bypass canonical certificate checking.
3. Freeze `mpk.ai.explain.request.v1`, `mpk.ai.explanation.v1`,
   `mpk.evidence-explainer.v1`, and `minimal-v1`; retain only the unchanged
   `mpk.ai.explanation.response.v0` provider-response schema.
   Explicitly retire `mpk.ai.explain.request.v0`, `mpk.ai.explanation.v0`,
   `mpk.evidence-explainer.v0`, and `minimal-v0` without aliases.
4. Freeze the redaction projection: preserve non-sensitive language, semantic
   parameters, strategy, checker, and axiom profile; replace helper kinds with
   `source` and `verification_ir`; exclude raw selection identity, paths,
   spans, compiler prose, and source text.
5. Freeze recognized Go/Rust strategy tuples, crossed-known-tuple rejection,
   unknown-future-strategy normalization, ordering, limits, prompt hash, and
   deterministic dry-run request bytes.

Deliverables:

- normative API and explainer v1 specs with fixtures.

Acceptance criteria:

- all v1 hashes and schema references are explicit;
- evidence v0 and explanation v0 reject without adapters;
- redaction tests can prove a sentinel source string and selection identity
  never enter the remote request.

Verification:

```sh
rg -n "POST /vir/import|mpk\.ai\.explain\.request\.v1|mpk\.ai\.explanation\.v1|mpk\.evidence-explainer\.v1|minimal-v1" develop/specs/AI_API_V1.md develop/specs/AI_EXPLAIN_V1.md
git diff --check
```

### VIR-00-T08 Freeze the Go VIR Profile

Status: Pending

Depends on: VIR-00-T01, VIR-00-T02, VIR-00-T04.

Inputs:

- design sections 12.2-12.3, 14, and 19.4;
- historical `develop/specs/GO_SUBSET_V0.md`;
- current `go-tools/go2gir` loader, feature, contract, and lowering behavior.

Likely touched files:

- `develop/specs/GO_VIR_PROFILE_V0.md`
- `develop/specs/vectors/go-vir-profile-v0.json`

Tasks:

1. Restate the accepted and rejected Go subset without GIR terminology and
   freeze the normative ID `mpk.go.fixed.v0`.
2. Freeze immutable input capture, module/workspace policy, exact source and
   contract discovery, and continued use of `mpk.go.contract.v0` as the Go
   source-side contract schema; also freeze standard-library/module-cache
   treatment, Go toolchain identity, explicit `GOOS`/`GOARCH`,
   `CGO_ENABLED=0`, loader flags, empty or allowlisted environment, and
   read-only resolution.
3. Freeze Go source selection, manifest language configuration, canonical
   target ID/pointer width, identifier rules, source-map coverage, diagnostic
   families, and deterministic limits.
4. Freeze Go operation/check requirements, including wrapping arithmetic,
   divisor, signed shift, over-width shift, array index, conversions, calls,
   loops, and contract semantics.
5. Map every historical positive and negative rule to a VIR representation or
   exact rejection; unexplained semantic widening is forbidden.

Deliverables:

- normative Go/VIR profile and migration vectors.

Acceptance criteria:

- the profile enumerates every file and external byte that may affect package
  loading;
- host target and inherited environment cannot affect a successful hash;
- every current Go baseline item has an explicit preservation rule.

Verification:

```sh
rg -n "mpk\.go\.fixed\.v0|GOOS|GOARCH|CGO_ENABLED|wrapping|loop" develop/specs/GO_VIR_PROFILE_V0.md
git diff --check
```

### VIR-00-T09 Freeze the Rust Subset and Driver Protocol

Status: Pending

Depends on: VIR-00-T02, VIR-00-T03, VIR-00-T04.

Inputs:

- design sections 7-11, 13-14, 17-19;
- the pinned-target and bundle contracts from prior VIR-00 tasks.

Likely touched files:

- `develop/specs/RUST_SUBSET_V0.md`
- `develop/specs/RUST_DRIVER_PROTOCOL_V0.md`
- `develop/specs/vectors/rust-subset-v0.json`
- `develop/specs/vectors/rust-driver-v0.json`

Tasks:

1. Copy the closed accepted/rejected source, AST, HIR, MIR, type, operation,
   purity, contract, target, path, module, visibility, attribute, call, and
   panic rules into `RUST_SUBSET_V0.md` with exact diagnostic codes and
   same-phase precedence.
2. Freeze portable path grammar, module-closure discovery, immutable-read
   rules, Cargo manifest allowlists, environment profile, rustc argument
   allowlist, MIR query/callback, the exact required toolchain component list
   (including a normative yes/no decision for `rust-src`), target allowlist
   `mpk.rust.targets.v0`, and every Rust-specific deterministic limit.
3. Freeze `mpk.rust.driver.v0` status-tagged schemas, request fingerprint,
   repeated identities, normalized inventory, raw lowered payload, diagnostics,
   JCS+LF transport, output-directory rules, and non-success artifact absence.
4. Define exact cross-process comparison responsibility between runner,
   `rust2vir`, driver, manifest, VIR, and source map.
5. Add positive and negative vectors for every accepted construct family,
   rejected family, status, identity mismatch, compiler change, and exact
   boundary.

Deliverables:

- normative Rust subset and private driver protocol.

Acceptance criteria:

- omitted compiler forms reject by default;
- no implementer must infer whether a construct belongs to preflight, source,
  HIR, MIR, contract, semantics, limit, or frontend failure;
- the private artifact can never be mistaken for a public frontend response or
  certificate input.

Verification:

```sh
rg -n "mpk\.rust\.checked\.v0|mir_drops_elaborated_and_const_checked|mpk\.rust\.driver\.v0|RUST_PREFLIGHT_|RUST_MIR_" develop/specs/RUST_SUBSET_V0.md develop/specs/RUST_DRIVER_PROTOCOL_V0.md
git diff --check
```

### VIR-00-T10 Amend Governance Documents and Close the Specification Gate

Status: Pending

Depends on: VIR-00-T01 through VIR-00-T09.

Inputs:

- all new normative specifications and vectors;
- `develop/specs/CERT_V0.md`, `TRUST_BOUNDARY_V0.md`,
  `AXIOM_POLICY_V0.md`, `develop/README.md`, roadmap, release gates, and
  templates.

Likely touched files:

- `develop/specs/CERT_V0.md`
- `develop/specs/TRUST_BOUNDARY_V0.md`
- `develop/specs/AXIOM_POLICY_V0.md`
- `develop/README.md`
- `develop/roadmap/RELEASE_GATES.md`
- `develop/templates/certificate_manifest.json`
- `develop/templates/module_manifest.yaml`
- `develop/specs/vectors/manifest.json`

Tasks:

1. Replace Go/GIR-specific active examples with language-neutral VIR/source
   manifest wording while preserving certificate v0 bytes and the four axiom
   categories unchanged.
2. State explicitly that registry bytes, toolchains, compilers, frontends,
   source, contracts, VIR, maps, manifests' internal claims, VCs, policy and AI
   output remain untrusted helper artifacts.
3. Record that no `RustSemanticsAxiom` exists and that `Std.Program.Base`
   aliases introduce no axiom; the existing `GoSemanticsAxiom` category is
   neither renamed nor broadened.
4. Route every new normative spec and vector from `develop/README.md`; label
   GIR, Go subset, and AI API v0 documents historical only after the atomic
   cutover, not before it.
5. Add a vector manifest containing schema ID, file path, SHA-256, owning spec,
   and implementation test owner for every vector set.
6. Run a cross-document review for schema IDs, field names, status/exit pairs,
   hash domains, limits, profile tuples, and ownership until no finding remains.

Deliverables:

- governance-approved language-neutral amendments and a closed specification
  gate.

Acceptance criteria:

- certificate v0 encoding and checker acceptance inputs are unchanged;
- all new schemas and domains have one normative owner and vector set;
- no active document contradicts the implementation sequence or trust boundary;
- the review ledger is empty.

Verification:

```sh
rg -n "VIR_V0|VC_V1|FRONTEND_PROTOCOL_V0|RELEASE_BUNDLES_V0|SOURCE_MAP_V0|SOURCE_MANIFEST_V0|GO_VIR_PROFILE_V0|RUST_SUBSET_V0|RUST_DRIVER_PROTOCOL_V0|POLICY_V1|AI_EXPLAIN_V1|AI_API_V1" develop/README.md
rg -n "RustSemanticsAxiom" develop/specs develop/docs
git diff --check
```

### VIR-01-T01 Implement Strict JSON, JCS, and Domain-Separated Hash Primitives

Status: Pending

Depends on: VIR-00-T10.

Inputs:

- canonical JSON and hash rules in all VIR-00 specifications;
- `crates/mpk-vc/Cargo.toml` and `crates/mpk-vc/src/lib.rs`.

Likely touched files:

- `crates/mpk-vc/src/canonical_json.rs`
- `crates/mpk-vc/src/hash.rs`
- `crates/mpk-vc/tests/canonical_json.rs`

Tasks:

1. Add a strict JSON value/parser that rejects duplicate object names before
   map construction, invalid Unicode, floats, non-integral exponent forms,
   integers outside the safe JSON range, trailing bytes, BOMs, and configured
   byte/node/depth limits.
2. Implement narrowed RFC 8785 encoding, including string escaping and
   UTF-16-code-unit object-key ordering, with no BOM, whitespace, or LF.
3. Keep schema-defined array ordering outside the generic encoder; provide
   explicit helpers for specification-owned unordered-set normalization.
4. Add `hash_canonical_json(domain, value_without_hash)` and raw-file/inventory
   SHA-256 helpers that accept an explicit static domain and cannot silently
   hash pretty JSON.
5. Load every VIR-00 JSON/hash vector and test byte equality, mutation
   sensitivity, cross-domain separation, and rejection behavior.

Deliverables:

- one shared Rust canonical JSON and hash implementation for all new untrusted
  artifacts.

Acceptance criteria:

- no new Rust consumer hashes `serde_json::to_vec` output directly;
- duplicate keys are observable and rejected before deserialization loses
  them;
- vector bytes and hashes match every normative fixture exactly.

Verification:

```sh
cargo test -p mpk-vc --test canonical_json
cargo test -p mpk-vc canonical_json
cargo clippy -p mpk-vc --all-targets -- -D warnings
git diff --check
```

Notes:

- This is helper-layer infrastructure. It does not change certificate v0
  canonical binary encoding.

### VIR-01-T02 Implement Release Registry and Bundle Validation Models

Status: Pending

Depends on: VIR-01-T01.

Inputs:

- `develop/specs/RELEASE_BUNDLES_V0.md` and its vectors.

Likely touched files:

- `crates/mpk-vc/src/release_bundle.rs`
- `crates/mpk-vc/tests/release_bundle.rs`
- `release/bundles/bundle-registry.json`

Tasks:

1. Add strict schema types for the registry, tuple key, frontend bundle,
   toolchain bundle, components, and inventories with all unknown fields
   denied.
2. Validate registry ID/hash, unique tuple and bundle IDs, exact language,
   profile and target compatibility, component/name ordering, portable paths,
   limits, and complete inventory hash equations.
3. Expose pure resolution APIs returning validated descriptors by registered
   ID. Do not add process execution or filesystem search to `mpk-vc`.
4. Add a fixture helper that recomputes expected registry ID/hash constants for
   the later `mpk-cli` build script without embedding registry bytes in this
   crate.
5. Test all normative vectors and prove a descriptor cannot add a subordinate
   executable or toolchain component after registry validation.

Deliverables:

- portable, side-effect-free release registry validation and descriptor
  resolution.

Acceptance criteria:

- registry validation is size-bounded and strict before tuple resolution;
- ambiguous or duplicate tuples reject;
- an unknown or incompatible caller ID is distinguishable from mutated
  installed bundle bytes for the outer runner's classification.

Verification:

```sh
cargo test -p mpk-vc --test release_bundle
cargo test -p mpk-vc release_bundle
cargo clippy -p mpk-vc --all-targets -- -D warnings
git diff --check
```

### VIR-01-T03 Implement Semantic Profiles and the VIR Data Model

Status: Pending

Depends on: VIR-01-T01.

Inputs:

- `develop/specs/VIR_V0.md` and VIR structure vectors;
- current GIR structures in `crates/mpk-vc/src/gir.rs`.

Likely touched files:

- `crates/mpk-vc/src/semantic_profile.rs`
- `crates/mpk-vc/src/vir.rs`
- `crates/mpk-vc/src/lib.rs`
- `crates/mpk-vc/tests/vir_model.rs`

Tasks:

1. Add closed enums for source language, semantic profile, semantic parameters,
   VIR type/value/instruction/terminator/safety-check kinds, contract
   expressions, declarations, blocks, functions, and units.
2. Model each instruction as a Rust tagged enum so inapplicable JSON fields are
   unrepresentable after strict deserialization; do not reproduce GIR's generic
   optional-field record.
3. Model nominal struct IDs, array length/type nesting, typed constants, block
   parameters, call contract hashes, loop cutpoints, and ordered source-neutral
   contracts exactly as the spec.
4. Add `import_vir_json` as strict parse plus structural validation entry point,
   but keep it internal to tests until the Go cutover.
5. Add conversion-free negative tests proving a GIR document and a
   wrong-language profile cannot deserialize as VIR.

Deliverables:

- exact in-memory representation of `mpk.vir.v0` and both initial profiles.

Acceptance criteria:

- every normative instruction has one Rust variant and every forbidden field
  combination rejects;
- unknown enum values and semantic-parameter fields fail closed;
- no public type branches on descriptive `source_language` to infer value
  semantics.

Verification:

```sh
cargo test -p mpk-vc --test vir_model
cargo test -p mpk-vc vir
cargo clippy -p mpk-vc --all-targets -- -D warnings
git diff --check
```

### VIR-01-T04 Implement Complete VIR Validation, Canonicalization, and Hashing

Status: Pending

Depends on: VIR-01-T03.

Inputs:

- VIR validation and stable-ID rules in design sections 12.4-12.5;
- VIR canonical and hash vectors.

Likely touched files:

- `crates/mpk-vc/src/vir_validate.rs`
- `crates/mpk-vc/src/vir_canonical.rs`
- `crates/mpk-vc/tests/vir_validation.rs`
- `crates/mpk-vc/tests/vir_hash.rs`

Tasks:

1. Implement `validate_vir` for unique IDs, closed value references, declaration
   order, exact types, entry/reachability, successor arguments, terminators,
   contract completeness, call resolution, loop cutpoints, and all shared
   limits.
2. Validate exact language/profile/parameter pairing; Rust CFGs and both call
   graphs are acyclic, while only Go profile cycles with valid loop contracts
   are accepted.
3. Recompute profile-required safety checks from operation and operand types and
   reject missing, extra, duplicate, reordered, wrong-signedness, or wrong-op
   entries.
4. Implement profile-independent canonical field/collection ordering and
   `MPK-VIR-0.1` hashing that excludes only `vir_hash`.
5. Enforce that absolute paths, compiler-local IDs, timestamps, hostnames, and
   temporary locators cannot appear in any canonical identity field.
6. Load all valid/invalid/hash vectors and add mutation tests for every repeated
   semantic context field.

Deliverables:

- complete validator, canonical encoder, and hash recomputation path.

Acceptance criteria:

- `import_vir_json` returns only a fully validated module;
- changing only profile, target, pointer width, safety checks, contract hash,
  or unit identity changes `vir_hash`;
- missing and extra Rust safety checks reject before VC generation.

Verification:

```sh
cargo test -p mpk-vc --test vir_validation
cargo test -p mpk-vc --test vir_hash
cargo clippy -p mpk-vc --all-targets -- -D warnings
git diff --check
```

### VIR-01-T05 Implement Source Map and Two-Stage Source Manifest Validation

Status: Pending

Depends on: VIR-01-T02, VIR-01-T04.

Inputs:

- `SOURCE_MAP_V0.md`, `SOURCE_MANIFEST_V0.md`, and their vectors.

Likely touched files:

- `crates/mpk-vc/src/source_map.rs`
- `crates/mpk-vc/src/source_manifest.rs`
- `crates/mpk-vc/tests/source_map.rs`
- `crates/mpk-vc/tests/source_manifest.rs`

Tasks:

1. Add strict source-map reference variants and validate unique total mappings,
   known VIR nodes, source input kind, normalized path, byte bounds, UTF-8
   scalar boundaries, sorting, schema/hash repetition, and limits.
2. Add strict manifest types for registry, toolchain, frontend, unit, target,
   language configuration, input entries, lifecycle stage, and all repeated
   semantic identities.
3. Recompute `input_set_hash`, `source_map_hash`, and frontend-stage
   `source_manifest_hash`; validate release descriptors and exact unit/selection
   linkage without interpreting them as proof evidence.
4. Add `attach_vc_hash` that consumes validated canonical frontend-stage bytes,
   cross-checks VC/VIR/input/profile fields, adds only `vc_hash`, and recomputes
   the final manifest hash.
5. Reject an attempt to mutate any other field, confuse the two lifecycle
   hashes, introduce an unlisted source, or attach a mismatched VC.

Deliverables:

- shared validated source-map and source-manifest lifecycle models.

Acceptance criteria:

- manifest finalization is not constructible from a hash alone;
- source-free checkers remain opaque to manifest JSON internals;
- every normative vector, including path and lifecycle attacks, passes.

Verification:

```sh
cargo test -p mpk-vc --test source_map
cargo test -p mpk-vc --test source_manifest
cargo clippy -p mpk-vc --all-targets -- -D warnings
git diff --check
```

### VIR-01-T06 Add `Std.Program.Base` and Profile-Aware Type/Expression Encoding

Status: Pending

Depends on: VIR-01-T03.

Inputs:

- design sections 10.1, 11, 12.2, and 16;
- current `type_encode.rs`, `expr_encode.rs`, and `proofs/go/base`.

Likely touched files:

- `crates/mpk-vc/src/type_encode.rs`
- `crates/mpk-vc/src/expr_encode.rs`
- `proofs/program/base`
- `fixtures/program-base`
- `crates/mpk-vc/tests/program_encoding.rs`

Tasks:

1. Add checked zero-axiom `Std.Program.Base.*` aliases over Bool, BitVec,
   fixed-array, and nominal-struct foundations, with certificate and type-map
   fixtures accepted by both checkers.
2. Replace encoder entry points with VIR/profile-aware types while keeping the
   current GIR APIs intact internally until the atomic cutover.
3. Encode bool, BV8/16/32/64, target-sized integers, arrays, structs, constants,
   source operations, and contract operations with the exact total semantics
   frozen in VIR v0.
4. Implement full-count cross-width shifts, signed/unsigned comparisons,
   division/remainder corner cases, componentwise array equality, nominal
   struct equality, and ordered contract Boolean trees.
5. Add tests proving Go and Rust reuse the same value encoder and differ only
   through validated profile/check metadata.

Deliverables:

- checked program foundations and shared language-neutral encoders.

Acceptance criteria:

- active new encoding contains no `Std.Go.Base.*` reference;
- all new foundation certificates pass both checkers with the expected axiom
  report;
- no Rust semantic axiom or unchecked alias is introduced.

Verification:

```sh
cargo test -p mpk-vc --test program_encoding
cargo test -p mpk-vc type_encode
cargo test -p mpk-vc expr_encode
./scripts/checker-agreement.sh
git diff --check
```

### VIR-01-T07 Implement Checked Safety Predicates and Profile Check Validation

Status: Pending

Depends on: VIR-01-T04, VIR-01-T06.

Inputs:

- checked-operation matrix in `VIR_V0.md`;
- current `crates/mpk-vc/src/safety.rs` and checked theory interfaces.

Likely touched files:

- `crates/mpk-vc/src/safety_check.rs`
- `crates/mpk-vc/src/safety.rs`
- `proofs/program/base`
- `crates/mpk-vc/tests/safety_profiles.rs`

Tasks:

1. Implement checked propositions for signed and unsigned add/subtract/multiply,
   signed negate, divisor nonzero, signed div/rem representability, signed
   shift nonnegativity, full-count shift bound, and signed/unsigned array index
   bounds at all accepted widths.
2. Derive the exact required safety-check set from semantic profile,
   instruction, and operand/result types; do not accept a frontend-supplied
   arbitrary proposition.
3. Map each safety-check instance to a canonical source-neutral obligation kind
   and stable ID component.
4. Provide checked-definition or existing checked-theory discharge fixtures and
   review resulting axiom reports under `zero-axiom` and `mvp-theory` as
   appropriate.
5. Test every operation/profile combination plus missing, extra, duplicate,
   reordered, wrong-width, and wrong-signedness checks.

Deliverables:

- complete profile check validation and checked safety proposition encoding.

Acceptance criteria:

- Go retains its reviewed runtime checks and Rust receives every required panic
  condition;
- an unsupported theory need stops with a test failure rather than adding a
  Rust axiom;
- all safety vectors have a checked discharge path or a documented
  proof-pending result.

Verification:

```sh
cargo test -p mpk-vc --test safety_profiles
cargo test -p mpk-vc safety
cargo test -p mpk-theory
git diff --check
```

### VIR-01-T08 Implement the Unified Acyclic Program WP Engine

Status: Pending

Depends on: VIR-01-T04, VIR-01-T06, VIR-01-T07.

Inputs:

- design sections 12.3 and 16;
- current `wp.rs`, `wp_branch.rs`, and `safety.rs`.

Likely touched files:

- `crates/mpk-vc/src/program_wp.rs`
- `crates/mpk-vc/src/wp.rs`
- `crates/mpk-vc/src/wp_branch.rs`
- `crates/mpk-vc/src/safety.rs`
- `crates/mpk-vc/tests/program_wp.rs`

Tasks:

1. Add one `generate_program_vcs` entry point over validated VIR and one
   worklist/dataflow engine for arbitrary acyclic CFGs, nested branches, joins,
   block parameters, local reassignment, early returns, and unreachable-block
   omission.
2. Preserve ordered path assumptions, short-circuit branch guards, return
   result substitution, contract requires/ensures, and deterministic block and
   instruction traversal.
3. Generate postcondition and operation-safety members in one traversal so
   identical path semantics cannot drift between separate WP and safety
   implementations.
4. Enforce streaming member, assumption, node, and depth counters before
   constructing oversized expressions; return artifact-free `VC_LIMIT_*`.
5. Add diamond, nested, join, early-return, short-circuit, empty-safety, and
   proof-pending fixtures under both profiles.

Deliverables:

- one acyclic CFG WP/safety engine shared by Go and Rust.

Acceptance criteria:

- current straight-line and branch intent is preserved;
- a right-hand short-circuit safety/call obligation contains the left guard;
- separate legacy generators are no longer used by any VIR test path.

Verification:

```sh
cargo test -p mpk-vc --test program_wp
cargo test -p mpk-vc program_wp
cargo clippy -p mpk-vc --all-targets -- -D warnings
git diff --check
```

### VIR-01-T09 Port Go Loop Cutpoints to the Unified WP Engine

Status: Pending

Depends on: VIR-01-T08.

Inputs:

- `GO_VIR_PROFILE_V0.md` loop rules;
- current `crates/mpk-vc/src/loops.rs` and loop fixtures.

Likely touched files:

- `crates/mpk-vc/src/program_wp.rs`
- `crates/mpk-vc/src/loops.rs`
- `crates/mpk-vc/tests/vir_loops.rs`

Tasks:

1. Represent validated Go loop headers as explicit cutpoints in the common
   engine rather than a separate serialized input or alternate WP API.
2. Generate initialization, preservation, exit, and conditional decreases
   members with the same assumption/substitution rules and stable IDs as the
   acyclic engine.
3. Require exact loop-contract/header linkage, reject uncovered cycles,
   multiple ambiguous cutpoints, bad successor shapes, and profile-incompatible
   termination metadata.
4. Preserve partial-correctness behavior when no decreases clause is required
   and total-correctness behavior when it is required.
5. Add explicit tests that the Rust profile rejects the identical cyclic VIR
   before WP generation.

Deliverables:

- shared Go loop-cutpoint support without weakening the Rust acyclic rule.

Acceptance criteria:

- pre-cutover loop obligation intent and ordering match the baseline report;
- every cyclic edge is accounted for by a validated cutpoint;
- no loop member is duplicated or omitted.

Verification:

```sh
cargo test -p mpk-vc --test vir_loops
cargo test -p mpk-vc loops
git diff --check
```

### VIR-01-T10 Implement Contract Hashing and Static-Call WP

Status: Pending

Depends on: VIR-01-T04, VIR-01-T08.

Inputs:

- design sections 10.5 and 11;
- `VIR_V0.md` call and contract rules.

Likely touched files:

- `crates/mpk-vc/src/contract.rs`
- `crates/mpk-vc/src/call_wp.rs`
- `crates/mpk-vc/src/program_wp.rs`
- `crates/mpk-vc/tests/call_wp.rs`

Tasks:

1. Recompute every normalized `contract_hash` and require each `CallStatic` to
   match the resolved same-module callee's ID, signature, profile, semantic
   parameters, and contract hash.
2. Validate the reachable VIR call graph independently from any frontend HIR
   closure and reject recursion, mixed units/languages, dynamic targets, and
   unresolved functions.
3. At each call, generate ordered callee-precondition members, introduce a fresh
   typed result, assume callee ensures for the subsequent path, and generate a
   callee-panic-free member.
4. Deduplicate declaration dependencies by callee while retaining every
   path-specific call-site member and deterministic call occurrence order.
5. Add direct, branched, multi-call, hash-mismatch, signature-mismatch,
   recursion, and source-dead/no-reachable-call vectors.

Deliverables:

- contract-bound static-call semantics shared by Go and Rust.

Acceptance criteria:

- no caller can consume a callee postcondition without the exact checked
  contract dependency;
- call safety depends on the callee panic-free declaration;
- an HIR-only source-dead callee may remain a standalone VIR function without
  inventing a reachable call dependency.

Verification:

```sh
cargo test -p mpk-vc --test call_wp
cargo test -p mpk-vc call_wp
git diff --check
```

### VIR-01-T11 Implement VC v1 Canonical Documents and Resource Accounting

Status: Pending

Depends on: VIR-01-T08, VIR-01-T09, VIR-01-T10.

Inputs:

- `VC_V1.md` and its document/hash/limit vectors;
- current `crates/mpk-vc/src/vc.rs`.

Likely touched files:

- `crates/mpk-vc/src/vc.rs`
- `crates/mpk-vc/src/vc_canonical.rs`
- `crates/mpk-vc/src/verification_limits.rs`
- `crates/mpk-vc/tests/vc_v1.rs`

Tasks:

1. Replace the VIR-side VC model with exact `mpk.vc.v1` structures containing
   source IR schema/hash, input-set hash, profile/parameters,
   `verification_limit_profile`, functions, members, groups, and `vc_hash`.
2. Assign every generated member exactly once, validate unique IDs and closed
   references, and reject missing, duplicate, ungrouped, or extra members.
3. Implement streaming counters and deterministic `VC_LIMIT_*` failures for
   members, assumptions, nodes, depth, and canonical output bytes.
4. Canonicalize and hash with `MPK-VC-1.0`; validate every repeated identity
   against the source VIR and manifest inputs.
5. Add import/re-encode tests that reject GIR/v0 fields, `schema_version`,
   `source_gir_hash`, and noncanonical JSON.

Deliverables:

- self-validating canonical VC v1 documents with deterministic limits.

Acceptance criteria:

- repeated clean generation is byte-identical;
- a below/at/above test exists for every downstream limit family;
- failure emits no partial VC document.

Verification:

```sh
cargo test -p mpk-vc --test vc_v1
cargo test -p mpk-vc verification_limits
git diff --check
```

### VIR-01-T12 Emit Grouped Certificate Skeletons and Close the Shared Foundation Gate

Status: Pending

Depends on: VIR-01-T05, VIR-01-T07, VIR-01-T11.

Inputs:

- grouped declaration rules and skeleton vectors in `VC_V1.md`;
- current `obligation_emit.rs` and checker-agreement scripts.

Likely touched files:

- `crates/mpk-vc/src/obligation_emit.rs`
- `crates/mpk-vc/src/grouping.rs`
- `crates/mpk-vc/tests/vc_skeleton_v1.rs`
- `fuzz/Cargo.toml`
- `fuzz/fuzz_targets/vir_parser.rs`
- `fuzz/fuzz_targets/source_map_parser.rs`
- `fixtures/vir-semantics`

Tasks:

1. Emit exactly two theorem declarations per function in callee-first
   topological order: contract then panic-free.
2. Build canonical parameter binders, outer/member implications, balanced
   conjunctions, empty `True`, exact member lists, and exact sorted generated
   declaration dependencies.
3. Emit `mpk.vc.cert_skeleton.v1` with source VC/IR/input/profile/limit
   identities and recompute the referenced VC hash before serialization.
4. Reject every missing, reversed, duplicate, cyclic, or extra dependency edge
   and every policy-member binding that does not name its containing
   declaration.
5. Add handcrafted cross-language total-operation/safety vectors, both-checker
   checked foundation fixtures, and parser fuzz targets for VIR and source maps.
6. Run a phase review against all VIR-00 vectors and the Go semantic baseline;
   close only when the findings ledger is empty.

Deliverables:

- grouped v1 skeleton emitter and a complete shared-foundation conformance
  gate.

Acceptance criteria:

- every check required by both semantic profiles can be represented and
  discharged through a checked path;
- all skeleton vectors and foundation certificates pass both checkers;
- no production CLI accepts VIR yet and the existing GIR release path remains
  green pending the atomic cutover.

Verification:

```sh
cargo test -p mpk-vc --test vc_skeleton_v1
cargo test -p mpk-vc --all-targets
./scripts/checker-agreement.sh
./scripts/check-fast.sh
git diff --check
```

### GO-VIR-02-T01 Build the Executable Go Migration Harness

Status: Pending

Depends on: VIR-00-T01, VIR-01-T12.

Inputs:

- `develop/migrations/go-gir-semantic-baseline.json`;
- current Go, VC, policy, certificate, and checker fixtures.

Likely touched files:

- `scripts/compare-go-gir-vir.py`
- `develop/migrations/go-gir-to-vir-report.json`
- `develop/migrations/go-gir-to-vir-report.md`
- `crates/mpk-vc/tests/go_migration.rs`

Tasks:

1. Implement a development-only comparator that reads checked-in pre-cutover
   GIR/VC baseline data and newly generated VIR/VC v1 data without becoming an
   importer used by production code.
2. Compare accepted/rejected source cases, normalized contracts, function
   identity, operation semantics, required runtime checks, loop members,
   property intent, and both checker verdicts. Allow only explicitly mapped
   schema, identifier, declaration-group, and foundation-name changes.
3. Emit a deterministic machine-readable report with one reviewed disposition
   for every difference and a Markdown derived view.
4. Make unexplained missing/extra obligation kinds, changed safety semantics,
   changed rejection class, or checker disagreement fail the harness.
5. Mark the comparator for deletion or archival in GO-VIR-02-T12; it is never a
   CLI input or release artifact.

Deliverables:

- reproducible semantic comparison harness and report schema.

Acceptance criteria:

- the harness detects a deliberately removed runtime check and a deliberately
  changed negative rejection;
- byte/hash differences alone do not fail when their semantic mapping is
  reviewed;
- production crates do not depend on the comparator.

Verification:

```sh
python3 scripts/compare-go-gir-vir.py --help
cargo test -p mpk-vc --test go_migration
git diff --check
```

### GO-VIR-02-T02 Create `go2vir` and the Generic CLI Envelope

Status: Pending

Depends on: VIR-01-T04, VIR-01-T05.

Inputs:

- `FRONTEND_PROTOCOL_V0.md`, `VIR_V0.md`, and `GO_VIR_PROFILE_V0.md`;
- current `go-tools/go2gir/main.go` and tests.

Likely touched files:

- `go-tools/go2vir/go.mod`
- `go-tools/go2vir/main.go`
- `go-tools/go2vir/protocol.go`
- `go-tools/go2vir/canonical_json.go`
- `go-tools/go2vir/hash.go`
- `go-tools/go2vir/main_test.go`

Tasks:

1. Create a separate migration-time `go2vir` module with the exact command
   `lower SOURCE_ROOT` and required `--package`, `--semantic-profile`,
   `--target`, `--function`, `--frontend-bundle-id`, `--frontend-sha256`,
   `--release-registry-id`, `--release-registry-sha256`,
   `--toolchain-bundle-id`, `--toolchain-root`,
   `--toolchain-distribution-sha256`, and repeatable `--contract` options.
2. Validate CLI/profile/selection arguments before source reads; usage errors
   exit 2 with empty stdout.
3. Implement Go-side strict JSON/JCS/hash helpers against the shared normative
   vectors; reject duplicate keys in contract and protocol inputs.
4. Emit exact canonical `mpk.frontend.cli.v0` status envelopes and LF with no
   SSA dump, GIR compatibility object, binary wrapper, or debug field.
5. Add status/exit, canonical-byte, extra-stdout, wrong-profile, and repeated-
   identity tests using placeholder-free fixture values defined in test data.

Deliverables:

- protocol-correct `go2vir` command skeleton with no lowering yet.

Acceptance criteria:

- all JSON-bearing statuses round-trip byte-identically through the Rust
  protocol vectors;
- standalone invalid configuration produces no JSON;
- `go2vir` cannot select a default semantic profile or target.

Verification:

```sh
(cd go-tools/go2vir && go test -count=1 ./...)
(cd go-tools/go2vir && go run . --help)
git diff --check
```

### GO-VIR-02-T03 Implement Immutable Go Input Capture and Pinned Loading

Status: Pending

Depends on: GO-VIR-02-T02, VIR-01-T02.

Inputs:

- Go loader and manifest rules in `GO_VIR_PROFILE_V0.md`;
- current `go2gir/loader.go`, `manifest.go`, and contract discovery.

Likely touched files:

- `go-tools/go2vir/preflight.go`
- `go-tools/go2vir/snapshot.go`
- `go-tools/go2vir/loader.go`
- `go-tools/go2vir/manifest.go`
- `go-tools/go2vir/testdata/preflight`

Tasks:

1. Structurally validate the source root, Go module/workspace files, selected
   package, explicit target, contract paths, portable names, regular files,
   links/reparse points, case collisions, and all Go-profile input limits.
2. Open each allowed original input without following links, read once into an
   immutable buffer, hash it, and build a private snapshot without rereading
   original paths.
3. Resolve the registered Go toolchain descriptor and construct the exact
   allowlisted loader environment, explicit `GOOS`/`GOARCH`,
   `CGO_ENABLED=0`, read-only module settings, isolated cache/home, and network
   denial.
4. Run package loading only against the snapshot and require its compiled-file
   inventory to match captured inputs and documented toolchain inputs exactly.
5. Emit the frontend-stage generic source manifest with complete build
   manifest, contract, source, toolchain, frontend, target, unit, limit, and
   registry identities.
6. Test hostile ambient Go, proxy, credential, locale, workspace, symlink, and
   concurrent-source-mutation cases.

Deliverables:

- reproducible snapshot-backed Go package loader and source manifest.

Acceptance criteria:

- no successful load reads the original tree after capture;
- identical inputs and registered bundles yield identical manifest bytes under
  hostile ambient environments;
- unrecorded module-cache or standard-library input is impossible or rejects.

Verification:

```sh
(cd go-tools/go2vir && go test -count=1 ./...)
(cd go-tools/go2vir && go test -count=1 -run 'TestPreflight|TestSnapshot|TestLoader|TestManifest' ./...)
git diff --check
```

### GO-VIR-02-T04 Port Go Feature Detection, Contracts, SSA Lowering, and Source Maps

Status: Pending

Depends on: GO-VIR-02-T03, VIR-01-T04, VIR-01-T05.

Inputs:

- `go-tools/go2gir/{features,contract_sidecar,lower,lower_types,ssa}.go`;
- Go profile and VIR/source-map vectors;
- Go semantic baseline.

Likely touched files:

- `go-tools/go2vir/features.go`
- `go-tools/go2vir/contract.go`
- `go-tools/go2vir/lower.go`
- `go-tools/go2vir/lower_types.go`
- `go-tools/go2vir/source_map.go`
- `go-tools/go2vir/emit.go`
- `go-tools/go2vir/corpus_test.go`

Tasks:

1. Port the fail-closed feature detector and preserve every historical positive
   and negative decision under Go/VIR diagnostic codes.
2. Parse explicit repeatable contract paths from the immutable snapshot,
   resolve them to canonical function IDs, normalize into shared VIR contracts,
   compute contract hashes, and reject duplicate or unused inputs.
3. Lower all accepted scalar, Boolean short-circuit, conversion, local,
   branch/join, early-return, array, struct, index, static-call, and loop forms
   directly to VIR with `mpk.go.fixed.v0`; never serialize GIR internally.
4. Emit exact Go profile safety checks, stable IDs, type/const declarations,
   call contract hashes, and loop cutpoints required by the shared validator.
5. Build total source-map entries from Go token byte offsets for every required
   source-derived VIR function, instruction, and terminator; validate UTF-8
   boundaries and captured source linkage.
6. Canonicalize/hash VIR and source map, populate the successful envelope, and
   cross-check with an independent Rust import in integration tests.

Deliverables:

- complete direct Go-to-VIR frontend over the accepted Go subset.

Acceptance criteria:

- every positive Go corpus input emits schema-valid VIR, map, and manifest;
- every negative baseline case retains its reviewed classification and stable
  semantic reason;
- no output or runtime path contains `mpk.gir`, `gir_hash`, `gir_emit`, or the
  old binary wrapper.

Verification:

```sh
(cd go-tools/go2vir && go test -count=1 ./...)
cargo test -p mpk-vc --test go_migration
rg -n "mpk\.gir|gir_hash|gir_emit|MPK_GIR_V0" go-tools/go2vir
git diff --check
```

### GO-VIR-02-T05 Implement the Registry-Pinned Generic Frontend Runner

Status: Pending

Depends on: VIR-01-T02, VIR-01-T05, GO-VIR-02-T04.

Inputs:

- release/frontend protocol specs and bundle fixtures;
- current process/path logic in `policy_scan.rs`.

Likely touched files:

- `crates/mpk-cli/src/frontend_registry.rs`
- `crates/mpk-cli/src/frontend_runner.rs`
- `crates/mpk-cli/src/frontend_protocol.rs`
- `crates/mpk-cli/src/frontend_sandbox.rs`
- `crates/mpk-cli/build.rs`
- `crates/mpk-cli/tests/frontend_runner.rs`
- `release/bundles/bundle-registry.json`
- `scripts/build-release-bundles.sh` (new deterministic assembler)
- `scripts/check-release-bundles.sh` (new installed-tree validator)

Tasks:

1. Build `go2vir` and the exact Go toolchain frozen by
   `GO_VIR_PROFILE_V0.md`; assemble their complete inventories in the
   specification-defined installation layout, register every supported Go
   profile/target tuple with no subordinate frontend binary, and fail the
   assembly if generated descriptors differ from the reviewed registry.
2. Generate and embed the expected registry ID/hash at `mpk-cli` build time,
   then derive
   the exact release root from the already opened `mpk` executable, open the
   installed `share/mpk/bundle-registry.json` without following links,
   size-bound/strictly parse/canonicalize it, and require equality with the
   embedded constants before resolving any tuple.
3. Resolve `libexec/mpk/bundles` under that same validated release root,
   enumerate each expected bundle,
   reject links/aliases/unlisted entries, hash the complete inventory, and hold
   immutable file handles or equivalent identities through launch.
4. Launch only the snapshotted registered main/subordinate/toolchain set under
   process, filesystem, network, stdout/stderr, and memory controls; failure to
   establish controls is `frontend-error`, never an unsandboxed retry.
5. Capture stdout/stderr with streaming limits and require one compact canonical
   envelope plus LF, exact status/exit pairing, no extra bytes, and no partial
   artifact on non-success.
6. Recompute and cross-check VIR, map, manifest, registry, bundle, target,
   profile, semantic parameters, selection, and all repeated hashes.
7. Provide test-only injected fixture roots behind `cfg(test)`; production
   routes expose no resolver or raw path flag.

Deliverables:

- language-neutral, registry-pinned frontend runner ready for the staged v1
  policy pipeline but not attached to a released command until GO-VIR-02-T12.

Acceptance criteria:

- changing executable bytes after validation cannot change what is launched;
- an assembled installed-tree fixture validates with the reviewed registry and
  launches only the registered Go main/toolchain pair;
- exit/status/protocol/identity mismatches are deterministic and artifact-free;
- no fallback searches `PATH`, project files, environment variables, or an
  adjacent registry;
- the released CLI still has no VIR frontend route.

Verification:

```sh
./scripts/check-release-bundles.sh --fixture go
cargo test -p mpk-cli --test frontend_runner
cargo test -p mpk-cli frontend_runner
cargo clippy -p mpk-cli --all-targets -- -D warnings
git diff --check
```

### GO-VIR-02-T06 Implement Policy Scan and Evidence v1 Data Models

Status: Pending

Depends on: VIR-01-T05, VIR-01-T11.

Inputs:

- `POLICY_V1.md` and its vectors;
- current `policy_scan.rs`, `policy_evidence.rs`, and `policy_report.rs`.

Likely touched files:

- `crates/mpk-cli/src/policy_schema.rs`
- `crates/mpk-cli/src/policy_scan.rs`
- `crates/mpk-cli/src/policy_evidence.rs`
- `crates/mpk-cli/src/policy_report.rs`
- `crates/mpk-cli/tests/policy_schema_v1.rs`
- `crates/mpk-cli/tests/policy_report.rs`

Tasks:

1. Add exact `mpk.policy.scan.v1` and `mpk.policy.evidence.v1`
   source-language/selection unions, semantic identities,
   registry/frontend/toolchain records, helper artifacts, two manifest hashes,
   VC fields, trusted evidence, grouped property refs, profiles, and structured
   reproduction recipes.
2. Strictly validate unique/sorted rows, repeated cross-artifact identities,
   strategy tuples, group/member/declaration references, trusted evidence, and
   canonical hash-bearing JSON.
3. Require `mpk_verified` to reference the containing accepted declaration and
   validate all transitive generated declaration dependencies.
4. Replace free-form commands with exact argv recipes and implement the
   specification-frozen POSIX display quoting in Markdown only.
5. Make Markdown a deterministic derived view over validated evidence v1 and
   preserve explicit helper/trusted boundary language for both languages.
6. Keep v0 types in their existing files until the atomic cutover, but do not
   add a v0-to-v1 adapter.

Deliverables:

- strict policy v1 models, validation, canonical encoding, and renderer.

Acceptance criteria:

- all normative v1 vectors pass and v0 payloads reject in v1 parsers;
- schemas contain no Go-only field, GIR helper kind, implicit axiom allowlist,
  or machine-local path;
- rendered recipes reconstruct the same argv elements without shell parsing in
  the JSON source of truth.

Verification:

```sh
cargo test -p mpk-cli --test policy_schema_v1
cargo test -p mpk-cli --test policy_report
cargo clippy -p mpk-cli --all-targets -- -D warnings
git diff --check
```

### GO-VIR-02-T07 Stage Generic `policy scan` Through `go2vir`

Status: Pending

Depends on: GO-VIR-02-T05, GO-VIR-02-T06.

Inputs:

- generic scan CLI and readiness rules in `POLICY_V1.md`;
- current `main.rs` and `policy_scan.rs`.

Likely touched files:

- `crates/mpk-cli/src/policy_scan.rs`
- `crates/mpk-cli/tests/policy_cli.rs`
- `crates/mpk-cli/tests/policy_scan.rs`

Tasks:

1. Implement a `cfg(test)`-gated v1 scan pipeline and argument/parser model,
   without attaching it to the released command tree, for mandatory `--language`,
   `--semantic-profile`,
   `--require-release-registry-id`, `--require-release-registry-sha256`,
   `--frontend-bundle`, `--toolchain-bundle`, `--target`, `--package`,
   `--function`, one or more normalized relative `--contract` values, and
   `--json-out` exactly as frozen by `POLICY_V1.md`.
2. Reject missing/crossed/unknown profiles and bundle tuples plus raw frontend,
   helper, driver, toolchain-root, registry-path, and old `--go2gir` options
   before launch.
3. Invoke the generic runner once, consume its validated successful or
   non-success response, and map status/diagnostic codes to policy readiness
   without reparsing source or reconstructing helper paths.
4. Populate scan v1 from the validated envelope/manifest/VIR/map identities;
   scan records no checker, strategy, or axiom selection.
5. Emit canonical safe-write JSON, deterministic readiness, normalized
   contracts, and no proof-acceptance field.

Deliverables:

- staged generic Go/VIR scan pipeline using registered bundles, callable only
  by internal tests until GO-VIR-02-T12.

Acceptance criteria:

- a Go scan uses `mpk.go.fixed.v0` and only the registered `go2vir`/Go
  toolchain tuple;
- malformed frontend output cannot become `ready`;
- repeated clean scans are byte-identical;
- the released `mpk policy scan` command still follows only the active GIR
  path and exposes none of the staged v1 options.

Verification:

```sh
cargo test -p mpk-cli policy_scan_v1
cargo test -p mpk-cli --test policy_cli
cargo test -p mpk-cli --test policy_scan
git diff --check
```

### GO-VIR-02-T08 Stage `policy verify` Through VC v1 and Finalize Evidence

Status: Pending

Depends on: GO-VIR-02-T07, VIR-01-T12.

Inputs:

- policy/VC/manifest lifecycle specs;
- current `policy_verify.rs`, package manifest verifier, and release report.

Likely touched files:

- `crates/mpk-cli/src/policy_verify.rs`
- `crates/mpk-cli/src/policy_evidence.rs`
- `crates/mpk-cli/src/main.rs`
- `crates/mpk-cli/src/package_verifier.rs` (new extraction from `main.rs`)
- `crates/mpk-cli/tests/policy_verify.rs`
- `crates/mpk-cli/tests/policy_evidence.rs`

Tasks:

1. Add a `cfg(test)`-gated v1 verify entrypoint, not yet connected to the
   released CLI, and reuse the exact validated internal scan result and
   canonical frontend-stage manifest bytes; do not launch again, reload source,
   or reconstruct a path.
2. Import validated VIR, call `generate_program_vcs`, classify v1 members,
   generate grouped skeletons, and preserve proof-pending behavior for
   obligations without accepted proof evidence.
3. Parse mandatory `--checker-profile`, `--strategy-profile`, and single
   `--axiom-profile`, plus explicit `--evidence-json` and `--evidence-md`;
   validate the exact Go strategy tuple before launch and record the fields
   independently.
4. Finalize the certificate-stage manifest by adding only `vc_hash`, attach its
   exact canonical bytes to certificate artifacts, and populate distinct scan
   and certificate manifest hashes in evidence.
5. Bind properties to containing checked declarations and transitive callee
   dependencies; retain checked theory evidence only when it is payload-bound
   to the exact member obligation.
6. Emit canonical scan/verify recipes and enforce source-root-relative contract
   order, safe output rules, strict mode, and explicit fixture updates.
7. Add package/release orchestration checks that active checker/axiom profiles
   equal evidence and are permitted by the package manifest and recomputed
   axiom report.

Deliverables:

- end-to-end Go policy verification over VIR/VC/evidence v1.

Acceptance criteria:

- policy verify never falls back to GIR or runs a hidden second scan;
- strict mode fails on any proof-pending member after writing valid untrusted
  evidence as specified;
- final manifest bytes differ from the frontend stage only by `vc_hash` and
  derived manifest hash;
- any `main.rs` extraction preserves the released verify route and active v0
  fixtures unchanged until the atomic cutover.

Verification:

```sh
cargo test -p mpk-cli policy_verify_v1
cargo test -p mpk-cli --test policy_verify
cargo test -p mpk-cli --test policy_evidence
cargo test -p mpk-cli package
git diff --check
```

### GO-VIR-02-T09 Stage `mpk explain` for Evidence and Explanation v1

Status: Pending

Depends on: GO-VIR-02-T06, GO-VIR-02-T08.

Inputs:

- `AI_EXPLAIN_V1.md` and vectors;
- current `ai_explain.rs`, tests, prompt, and Vertex documentation.

Likely touched files:

- `crates/mpk-cli/src/ai_explain.rs`
- `crates/mpk-cli/tests/ai_explain.rs`
- `develop/migrations/go-vir-staging/docs/vertex-ai-gemini-assistant-design.md`

Tasks:

1. Add a separate `cfg(test)`-gated v1 explainer entrypoint that accepts only
   validated `mpk.policy.evidence.v1` and validates its exact known
   strategy/language/semantic/axiom tuple before redaction; do not replace the
   released v0 entrypoint yet.
2. Emit `mpk.ai.explain.request.v1` using `minimal-v1`, generic helper kinds,
   source language, non-path semantic parameters, and independent strategy,
   checker, and axiom profiles.
3. Exclude source selection, contract paths, raw source, source-map spans,
   diagnostics prose, compiler output, and sentinel secret text; retain only
   stable code/count summaries allowed by the spec.
4. Update prompt/hash, dry-run golden request, output parser, and report to
   `mpk.evidence-explainer.v1` and `mpk.ai.explanation.v1`; retain the unchanged
   provider response v0 parser only.
5. Test Go/Rust recognized tuples, unknown future strategies, crossed known
   tuple rejection, deterministic English/Japanese outputs, and v0 evidence
   rejection.
6. Stage the Vertex assistant documentation replacement under
   `develop/migrations/go-vir-staging/docs`; leave the active document and help
   text unchanged for GO-VIR-02-T12.

Deliverables:

- language-neutral explainer v1 with deterministic credential-free dry run.

Acceptance criteria:

- AI prose remains untrusted and cannot change local property IDs/statuses;
- request fixtures contain only `source` and `verification_ir` helper kinds;
- v0 evidence/explanation input has no adapter path;
- the released `mpk explain` route still accepts only its pre-cutover model.

Verification:

```sh
cargo test -p mpk-cli ai_explain_v1
cargo test -p mpk-cli --test ai_explain
cargo test -p mpk-cli ai_explain
git diff --check
```

### GO-VIR-02-T10 Stage the AI API v1 VIR Import Boundary

Status: Pending

Depends on: VIR-01-T11, GO-VIR-02-T08.

Inputs:

- `AI_API_V1.md` and vectors;
- `crates/mpk-api` session/error conventions.

Likely touched files:

- `crates/mpk-api/src/v1_router.rs`
- `crates/mpk-api/src/vir_api.rs`
- `crates/mpk-api/src/vc_api.rs`
- `crates/mpk-api/src/lib.rs`
- `crates/mpk-api/src/v1_tests.rs`

Tasks:

1. Add an isolated private v1 router plus structured request/response models and
   service operations corresponding to `POST /vir/import`, VC generation/list/
   start/attach/check over VIR and VC v1 identities; do not publish the v1
   router or any v1 type from the active API surface yet. `lib.rs` declares the
   staged modules only under `cfg(test)`; GO-VIR-02-T12 removes that gate and
   publishes the reviewed v1 exports atomically.
2. Import only strict canonical validated VIR, retain its schema/hash/profile
   identities in session state, and make candidate/session results explicitly
   helper data.
3. Reject GIR, wrong hashes, noncanonical JSON, unknown profiles, mismatched VC
   context, stale session IDs, and any candidate state mutation after a failed
   operation.
4. Preserve the existing certificate export/check acceptance boundary; an API
   success is never a theorem acceptance verdict.
5. Add route-model conformance vectors and tests that `POST /gir/import` is
   unknown at the v1 router boundary.

Deliverables:

- API v1 VIR/VC service surface with no GIR adapter.

Acceptance criteria:

- all unchanged session/term/proof operations retain behavior;
- VIR/VC context cannot be mixed across sessions;
- only canonical certificate checking can mark exported declarations accepted;
- the released API still exposes only its pre-cutover import boundary.

Verification:

```sh
cargo test -p mpk-api v1_router
cargo test -p mpk-api vir_api
cargo test -p mpk-api vc_api
cargo test -p mpk-api
git diff --check
```

### GO-VIR-02-T11 Regenerate the Go Corpus and Complete the Semantic Audit

Status: Pending

Depends on: GO-VIR-02-T04, GO-VIR-02-T08, GO-VIR-02-T09, GO-VIR-02-T10.

Inputs:

- migration harness from GO-VIR-02-T01;
- all current Go, VC, certificate, policy, AI, example, and release fixtures.

Likely touched files:

- `fixtures/vir-go` (new shared Go/VIR corpus)
- `develop/migrations/go-vir-staging/fixtures` (new staged replacements)
- `develop/migrations/go-vir-staging/examples` (new staged replacements)
- `develop/migrations/go-vir-staging/release-report.json`
- `develop/migrations/go-gir-to-vir-report.json`
- `develop/migrations/go-gir-to-vir-report.md`

Tasks:

1. Regenerate canonical frontend envelopes, VIR, source maps, both manifest
   stages, VC v1, grouped skeletons, certificates, axiom reports, policy v1,
   AI v1 dry-run/output, examples, and all recorded hashes through explicit
   fixture-update commands. Store replacements for active fixtures, examples,
   and the release report under the exact
   `develop/migrations/go-vir-staging` root; the active release must not select
   that root before GO-VIR-02-T12.
2. Run all 100 Go alpha functions, payment policies, loops, conversions,
   runtime operations, contracts, calls, negative cases, and both checkers.
3. Complete and review every semantic-difference disposition; update expected
   identifier/foundation/grouping changes and reject unexplained loss or
   widening.
4. Run two clean generations and compare every canonical byte; inspect all
   artifacts for local path, temp path, host, timestamp, and old interface
   leakage.
5. Record the intentional hash migration rather than adding compatibility
   aliases or preserving old byte forms.

Deliverables:

- reviewed regenerated Go/VIR corpus and zero-unexplained-difference report.

Acceptance criteria:

- every positive artifact passes both source-free checkers;
- every negative case preserves its reviewed source behavior;
- every expected hash change is recorded and deterministic;
- the migration report has no unresolved disposition;
- active GIR-era fixtures remain referenced until the atomic cutover, while
  the complete VIR replacement set is already testable by explicit paths.

Verification:

```sh
(cd go-tools/go2vir && go test -count=1 ./...)
cargo test -p mpk-vc --test alpha_corpus
cargo test -p mpk-vc --test payment_policy_examples
cargo test -p mpk-cli --test policy_verify
./scripts/checker-agreement.sh
python3 scripts/compare-go-gir-vir.py --check
git diff --check
```

### GO-VIR-02-T12 Perform the Atomic VIR Cutover and Remove GIR

Status: Pending

Depends on: GO-VIR-02-T01 through GO-VIR-02-T11.

Inputs:

- completed migration report and strict obsolete-interface inventory;
- all production code, fixtures, examples, scripts, templates, and active docs.

Likely touched files:

- `Cargo.toml`
- `crates/mpk-vc/src`
- `crates/mpk-cli/src`
- `crates/mpk-api/src`
- `go-tools/go2gir` removed
- active fixtures/examples/docs/scripts/README files
- `develop/README.md`
- `develop/docs/05_rust_frontend_design.md`
- `develop/docs/05_rust_frontend_design-todo.md`
- `develop/specs/GIR_V0.md`, `GO_SUBSET_V0.md`, `AI_API_V0.md`

Tasks:

1. Switch all public production consumers to the already-tested VIR, VC v1,
   generic frontend, policy v1, AI v1, and API v1 paths in one change. Remove
   the GO-VIR-02-T07 through T10 `cfg(test)` staging gates, connect the reviewed
   CLI/router exports, and expose no intermediate compatibility switch.
   Generate each active fixture/example/release-report replacement again,
   require byte equality with `develop/migrations/go-vir-staging`, then install
   the matched bytes at their final active paths.
2. Keep `go-tools/go2vir` as the sole active Go frontend location and remove
   `go-tools/go2gir`, GIR importer/emitter/canonical wrapper, legacy WP
   exports, `Std.Go.Base` active mappings, old flags, fields, enums, and
   fixtures. Retire the exact active schemas `mpk.gir.v0`, `mpk.gir.emit.v0`,
   `mpk.go2gir.cli.v0`, `mpk.go.source_manifest.v0`,
   `mpk.vc.cert_skeleton.v0`, `mpk.policy.scan.v0`,
   `mpk.policy.evidence.v0`, `mpk.ai.explain.request.v0`,
   `mpk.ai.explanation.v0`, and `mpk.evidence-explainer.v0`, plus the v0 AI
   import route.
3. Reject retired schemas, fields, routes, status names, and flags
   deterministically as unknown input; do not retain a runtime adapter or
   alias.
4. Update root/developer/ProofOps/Vertex/CI/release documentation, templates,
   `check-fast.sh`, `check-all.sh`, release report generation, and help text in
   the same change. Mark frozen GIR-era specs historical, and mark this design
   plus its todo as active Rust migration references whose retained old names
   describe only the completed Go cutover, before enabling their exact
   obsolete-search allowlist entries.
5. Archive only the reviewed semantic migration report; delete the executable
   one-off comparator unless governance requires its non-production source,
   in which case isolate and label it historical.
6. Run the strict obsolete-interface search outside historical allowlisted
   files and fix every hit.

Deliverables:

- one post-cutover codebase with VIR as the sole source-program IR.

Acceptance criteria:

- no production parser, CLI, API route, fixture, CI command, or active user doc
  consumes or emits GIR or policy/AI v0;
- all Go gates and both checkers pass from a clean checkout;
- source-free certificate semantics and certificate v0 bytes remain governed
  by the unchanged checker contract;
- `./scripts/check-no-active-gir.sh --strict` passes.

Verification:

```sh
./scripts/check-no-active-gir.sh --strict
./scripts/check-fast.sh
./scripts/check-all.sh
(cd go-tools/go2vir && go test -count=1 ./...)
cargo test --workspace
python3 scripts/generate-release-report.py --check
git diff --check
```

### RUST-03-T01 Create the Isolated Pinned Rust Frontend Project

Status: Pending

Depends on: GO-VIR-02-T12.

Inputs:

- `RUST_SUBSET_V0.md`, `RUST_DRIVER_PROTOCOL_V0.md`, and release bundle spec;
- exact nightly, compiler commit, components, targets, and bundle IDs frozen by
  VIR-00-T09.

Likely touched files:

- `Cargo.toml`
- `rust-tools/rust2vir/Cargo.toml`
- `rust-tools/rust2vir/Cargo.lock`
- `rust-tools/rust2vir/rust-toolchain.toml`
- `rust-tools/rust2vir/src/bin/rust2vir.rs`
- `rust-tools/rust2vir/src/bin/rust2vir-driver.rs`
- `rust-tools/rust2vir/testdata/bundle-candidate` (unregistered test data)
- `scripts/build-release-bundles.sh`

Tasks:

1. Add `rust-tools/rust2vir` as an explicitly excluded root-workspace package
   with two binaries and exact locked dependencies; the ordinary workspace
   continues to build on stable Rust.
2. Pin the exact nightly and install exactly the component inventory frozen by
   `RUST_SUBSET_V0.md`, plus both i686/x86_64 Linux standard-library targets.
   Record the rustc commit in a build-generated constant used by the driver
   startup check; do not decide component membership during implementation.
3. Add minimal main/driver version output and make the driver refuse a compiler
   commit other than the embedded one before analysis.
4. Define reproducible isolated release-build commands and use the deterministic
   assembler to create unregistered test candidate inventories for the main,
   exactly one driver, toolchain distribution, components, executable digests,
   both target libraries, limit profile, environment profile, and argument
   allowlist. These candidates exercise digest/inventory validation but cannot
   be selected by an evidence route; final executable registration is owned by
   RUST-03-T12 after the binaries stop changing.
5. Add a root test proving `cargo test --workspace` does not select or require
   the nightly frontend package.

Deliverables:

- isolated buildable pinned frontend project and initial release registration.

Acceptance criteria:

- stable root workspace and pinned frontend build independently;
- toolchain mismatch fails deterministically;
- Rust registration requires exactly one subordinate binary named
  `rust2vir-driver` when finalized;
- the active release registry and its Go tuples/hash remain unchanged in this
  milestone, and no candidate bundle can produce policy evidence.

Verification:

```sh
cargo test --workspace
(cd rust-tools/rust2vir && cargo fmt --all -- --check)
(cd rust-tools/rust2vir && cargo test --locked)
(cd rust-tools/rust2vir && cargo run --locked --bin rust2vir -- --version)
git diff --check
```

### RUST-03-T02 Implement Rust CLI Selection and Structural Path Preflight

Status: Pending

Depends on: RUST-03-T01.

Inputs:

- Rust invocation and path grammar in the subset/protocol specs.

Likely touched files:

- `rust-tools/rust2vir/src/cli.rs`
- `rust-tools/rust2vir/src/preflight.rs`
- `rust-tools/rust2vir/src/path.rs`
- `rust-tools/rust2vir/tests/cli.rs`
- `rust-tools/rust2vir/tests/preflight_paths.rs`

Tasks:

1. Parse exact `lower SOURCE_ROOT` options: require
   `--manifest-path Cargo.toml`, `--package`, `--semantic-profile`, `--target`,
   `--function`, `--frontend-bundle-id`, `--frontend-sha256`,
   `--release-registry-id`, `--release-registry-sha256`,
   `--toolchain-bundle-id`, `--toolchain-root`,
   `--toolchain-distribution-sha256`, `--driver`, `--driver-sha256`, and
   repeatable relative `--contract` values.
2. Validate package/crate/function identifier grammars and derive the expected
   crate and library-kind selection before reading source.
3. Validate portable path components, lengths, ASCII case-fold uniqueness,
   reserved Windows names, root containment, regular-file type, symlink and
   reparse-point rejection, and pre-parse byte/count limits.
4. Reject root/nested Cargo workspaces, target repository toolchain files,
   `.cargo` configuration, manifest/root escape, and unsupported source-root
   filesystem types before Cargo or rustc starts.
5. Implement phase-ordered CLI/preflight diagnostics and ensure exit 2 emits no
   JSON while structural profile rejection emits the exact non-success
   envelope only after CLI configuration succeeds.

Deliverables:

- deterministic request selection and structural preflight boundary.

Acceptance criteria:

- missing profile/target/bundle/registry assertions cannot default;
- caller paths never enter canonical selection or identity fields;
- path failures do not follow a link or invoke Cargo.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test cli)
(cd rust-tools/rust2vir && cargo test --locked --test preflight_paths)
(cd rust-tools/rust2vir && cargo clippy --locked --all-targets -- -D warnings)
git diff --check
```

### RUST-03-T03 Implement Module Closure, Immutable Capture, and Snapshot Creation

Status: Pending

Depends on: RUST-03-T02.

Inputs:

- design section 7 and Rust source-input rules;
- portable path and resource-limit tests.

Likely touched files:

- `rust-tools/rust2vir/src/module_closure.rs`
- `rust-tools/rust2vir/src/source_capture.rs`
- `rust-tools/rust2vir/src/snapshot.rs`
- `rust-tools/rust2vir/tests/module_closure.rs`
- `rust-tools/rust2vir/tests/snapshot.rs`

Tasks:

1. Parse the allowlisted library-root setting or documented default, capture it
   once, and recursively discover only ordinary out-of-line `mod name;` under
   pinned default module-path rules while traversing inline modules in place.
2. Reject expansion-affecting constructs before following child paths and
   reject missing/ambiguous modules, duplicate normalized paths, case-fold
   collisions, cycles, root escapes, and unrelated file discovery.
3. Open each manifest, lockfile, contract, and discovered source with safe
   descriptor-relative no-follow operations, enforce size limits during read,
   and retain one immutable byte buffer and identity per input.
4. Create a private snapshot exclusively from captured buffers, with validated
   permissions and paths; never reread an original input or copy links.
5. Detect original-tree mutation after capture in tests and prove the snapshot
   and input hashes remain bound to the captured bytes.
6. Provide a fixed cleanup guard that validates its dedicated temporary root
   and never traverses a reparse point or user path.

Deliverables:

- exact module closure and race-resistant immutable analysis snapshot.

Acceptance criteria:

- an unrelated `.rs` file is neither read, copied, nor manifested;
- every compiled Rust source must originate from one captured buffer;
- source mutation cannot produce hash/analyzed-byte disagreement.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test module_closure)
(cd rust-tools/rust2vir && cargo test --locked --test snapshot)
(cd rust-tools/rust2vir && cargo clippy --locked --all-targets -- -D warnings)
git diff --check
```

### RUST-03-T04 Implement Structural Cargo Manifest Preflight

Status: Pending

Depends on: RUST-03-T03.

Inputs:

- design section 8.2 and manifest allowlists in `RUST_SUBSET_V0.md`.

Likely touched files:

- `rust-tools/rust2vir/src/manifest.rs`
- `rust-tools/rust2vir/src/metadata_request.rs`
- `rust-tools/rust2vir/tests/manifest_preflight.rs`
- `rust-tools/rust2vir/testdata/cargo-preflight`

Tasks:

1. Strictly parse the captured TOML first and reject workspace tables/inherited
   fields, dependencies of every class, build scripts, features, explicit
   crate types, unsupported build-affecting fields, and malformed descriptive
   metadata before process execution.
2. Validate the selected package name, edition 2021, allowlisted default
   library-root declaration, absence of target-specific/development/build
   dependency tables, and required captured `Cargo.lock` before any child
   process starts.
3. Construct an immutable expected selection and exact Cargo metadata request
   model for the snapshot; it contains no original/ancestor path and cannot
   choose a manifest by working-directory search.
4. Normalize only fields documented for later manifest cross-checking; never
   retain descriptive metadata as a build input.
5. Test all dependency/workspace/feature/build-script/crate-type/unknown-field
   forms and prove every structural rejection occurs without launching Cargo.

Deliverables:

- fail-closed structural manifest/lock validation and metadata request.

Acceptance criteria:

- no child process is executed by this milestone's preflight;
- the later metadata command has one explicit snapshot manifest and expected
  package/library selection;
- unsupported manifest structure rejects before sandbox setup.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test manifest_preflight)
(cd rust-tools/rust2vir && cargo clippy --locked --all-targets -- -D warnings)
git diff --check
```

### RUST-03-T05 Implement the Sandboxed Cargo Metadata and Check Launcher

Status: Pending

Depends on: RUST-03-T01, RUST-03-T04.

Inputs:

- design section 8.3;
- specification-frozen toolchain, environment, argument, target, and sandbox
  profiles plus the unregistered test candidate from RUST-03-T01.

Likely touched files:

- `rust-tools/rust2vir/src/environment.rs`
- `rust-tools/rust2vir/src/sandbox.rs`
- `rust-tools/rust2vir/src/cargo_metadata.rs`
- `rust-tools/rust2vir/src/cargo_check.rs`
- `rust-tools/rust2vir/tests/sandbox.rs`
- `rust-tools/rust2vir/tests/cargo_preflight.rs`
- `rust-tools/rust2vir/tests/cargo_check.rs`

Tasks:

1. Implement resolution/revalidation of the launcher-supplied registered
   toolchain bundle,
   distribution digest, Cargo/rustc binaries, support files, target library,
   driver, and target tuple before process creation. Until RUST-03-T12, tests
   reach that code only through dependency injection of the unregistered
   candidate; no evidence route accepts the candidate.
2. Construct the child environment from an empty map with only the exact
   profile values, private empty home/Cargo home/temp/target, toolchain-only
   path, deterministic locale/timezone, offline/no-incremental settings,
   `RUSTC` set to the snapshotted registered rustc,
   `RUSTC_WORKSPACE_WRAPPER` set to the snapshotted registered driver, and
   `CARGO_ENCODED_RUSTFLAGS` set to the exact unit-separator encoding of the
   registered semantic arguments in their specified order. Keep `RUSTFLAGS`
   and `RUSTC_WRAPPER` absent. The initial profile sets private `HOME`,
   `CARGO_HOME`, `TMPDIR`, and `CARGO_TARGET_DIR`; toolchain-only `PATH`;
   `LC_ALL=C`, `LANG=C`, `TZ=UTC`, `TERM=dumb`,
   `CARGO_TERM_COLOR=never`, `CARGO_NET_OFFLINE=true`,
   `CARGO_INCREMENTAL=0`, and `RUST_BACKTRACE=0`; and leaves
   `RUSTC_BOOTSTRAP` plus every other `CARGO_*`/`RUST*` variable absent. Its
   encoded arguments are exactly `-C overflow-checks=yes`, `-C panic=abort`,
   `-C debug-assertions=no`, `-C opt-level=0`, `-Z mir-opt-level=0`, and
   `--remap-path-prefix=SNAPSHOT_ROOT=.` in that order, with `SNAPSHOT_ROOT`
   replaced only after boundary validation.
3. Establish the release OS sandbox with read access only to immutable
   toolchain and snapshot, write access only to private temp/target/driver
   output, no network/home/credentials, and explicit process/memory/output
   controls. Unsupported hosts or unavailable controls return the stable
   sandbox-unavailable frontend error with no fallback.
4. Run the exact pinned Cargo `metadata` command first under that same sandbox
   and environment with `--manifest-path SNAPSHOT_ROOT/Cargo.toml`,
   `--format-version 1`, `--no-deps`, `--locked`, `--offline`,
   `--no-default-features`, and `--color never` in the frozen order. Strictly
   parse its bounded JSON, select exactly one matching default library target,
   and cross-check lockfile freshness, package/crate/manifest identity,
   dependency/proc-macro/build closure, and absence of ancestor configuration.
5. Run the exact `cargo check --lib --package PACKAGE --target TARGET`
   invocation with the same explicit snapshot manifest followed by `--locked`,
   `--offline`, `--no-default-features`, `--jobs 1`,
   `--message-format json`, and `--color never`. Bound/normalize its structured
   message stream without copying raw stderr into canonical output. Use a
   registered fixture wrapper until the real private handshake lands in
   RUST-03-T06.
6. Prove effective environment, flags, target, and filesystem view are
   unchanged under hostile ambient `CARGO_*`, `RUST*`, proxy, credential,
   locale, wrapper, rustup, and working-directory inputs.

Deliverables:

- one deterministic sandboxed runner for both metadata and compilation.

Acceptance criteria:

- compiler execution cannot read the original source tree or user home;
- metadata and check use the same toolchain, environment, sandbox, limits, and
  explicit snapshot manifest;
- sandbox failure never retries with broader access;
- child output limits classify as artifact-free frontend error.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test sandbox)
(cd rust-tools/rust2vir && cargo test --locked --test cargo_preflight)
(cd rust-tools/rust2vir && cargo test --locked --test cargo_check)
(cd rust-tools/rust2vir && cargo clippy --locked --all-targets -- -D warnings)
git diff --check
```

### RUST-03-T06 Implement the Private Driver Protocol and Process Handshake

Status: Pending

Depends on: RUST-03-T05.

Inputs:

- `RUST_DRIVER_PROTOCOL_V0.md` and private artifact vectors.

Likely touched files:

- `rust-tools/rust2vir/src/driver_protocol.rs`
- `rust-tools/rust2vir/src/driver_process.rs`
- `rust-tools/rust2vir/src/bin/rust2vir-driver.rs`
- `rust-tools/rust2vir/tests/driver_protocol.rs`

Tasks:

1. Implement exact request fingerprint construction over profile, target,
   selection, option-profile IDs, registry identity, and expected binary and
   toolchain digests, excluding all runtime paths.
2. Create a fresh validated empty driver-output directory and enforce exactly
   one atomic regular artifact, no links, unexpected entries, partial files,
   duplicates, or oversize output.
3. Emit and parse exact status-tagged JCS+LF `mpk.rust.driver.v0` artifacts;
   non-success variants contain only repeated identities and bounded normalized
   diagnostics.
4. Cross-check every request, compiler, driver, toolchain, package, crate,
   function, inventory, payload, and present source identity before public
   emission; never reuse stale output.
5. Classify compiler failure without a complete artifact locally as
   `frontend-error` and test missing, partial, duplicate, noncanonical,
   mismatched, killed, and oversized cases.

Deliverables:

- strict private main/driver handshake with no ambiguous output state.

Acceptance criteria:

- zero or multiple matching artifacts reject;
- no non-success artifact contains a partial VIR, source map, manifest, or
  lowered payload;
- all private protocol vectors pass byte-for-byte.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test driver_protocol)
(cd rust-tools/rust2vir && cargo clippy --locked --all-targets -- -D warnings)
git diff --check
```

### RUST-03-T07 Implement the Pre-Expansion Source Gate and Custom File Loader

Status: Pending

Depends on: RUST-03-T03, RUST-03-T06.

Inputs:

- design section 7 and source-gate rules in `RUST_SUBSET_V0.md`.

Likely touched files:

- `rust-tools/rust2vir/src/source_gate.rs`
- `rust-tools/rust2vir/src/file_loader.rs`
- `rust-tools/rust2vir/src/bin/rust2vir-driver.rs`
- `rust-tools/rust2vir/tests/source_gate.rs`

Tasks:

1. Lex/parse every captured source buffer and reject `cfg`, `cfg_attr`, `path`,
   derive, macro definitions/invocations, `include!`, unsupported attributes,
   expansion-affecting syntax, and invalid identifiers under exact stable codes.
2. Install a rustc `FileLoader` that can return only preflight-discovered
   immutable snapshot bytes, validates each file again before return, and
   refuses external, synthetic, unexpected, unread, or unsnapshotted source.
3. Apply the same gate to the crate-root AST callback and require root plus
   loader inventory to equal the preflight closure exactly after compilation.
4. Preserve fixed classification: valid out-of-profile syntax is `rejected`,
   ordinary parse/module errors are `source-error`, path/root attacks are
   preflight rejection, and compiler/discovery disagreement is
   `frontend-error`.
5. Add macro/cfg/path/doc/lint/module inventory and phase-precedence tests.

Deliverables:

- compiler-boundary source gate that prevents expansion from hiding policy
  violations.

Acceptance criteria:

- rustc never reads an original or undiscovered source path;
- source gate runs before bytes reach compiler parsing;
- compiler/source inventory mismatch cannot trigger a broader scan.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test source_gate)
(cd rust-tools/rust2vir && cargo clippy --locked --all-targets -- -D warnings)
git diff --check
```

### RUST-03-T08 Wire rustc Callbacks and Validate the Effective Session

Status: Pending

Depends on: RUST-03-T05, RUST-03-T06, RUST-03-T07.

Inputs:

- pinned callback/query contract in `RUST_SUBSET_V0.md`;
- exact rustc argument and effective-option allowlists.

Likely touched files:

- `rust-tools/rust2vir/src/rustc_driver.rs`
- `rust-tools/rust2vir/src/session.rs`
- `rust-tools/rust2vir/src/mir_access.rs`
- `rust-tools/rust2vir/tests/session.rs`
- `rust-tools/rust2vir/tests/mir_access.rs`

Tasks:

1. Filter wrapper invocations on exact primary package, crate, lib crate type,
   manifest identity, target, and request fingerprint; nonmatching rustc
   invocations produce no artifact.
2. Reject every rustc argument outside the versioned allowlist after
   normalizing the explicitly permitted input/output paths; unapproved `-A`,
   `-W`, `-D`, `-F`, and `--cap-lints` arguments always reject.
3. Inspect the final effective session and require exact edition, target,
   pointer width, panic strategy, overflow checks, debug assertions, MIR/rustc
   optimization levels, features, and complete sorted `cfg` set.
4. Access `mir_drops_elaborated_and_const_checked` at the frozen callback/query
   point before later queries can take the body; reject query/dialect mismatch
   and never use optimized MIR.
5. Add a compatibility fingerprint/golden summary of accepted MIR enum shapes
   so a compiler update fails until adapter review and fixture regeneration.

Deliverables:

- exact pinned rustc callback and pre-optimization MIR access layer.

Acceptance criteria:

- merely passing desired flags is insufficient when effective options differ;
- a changed rustc argument, commit, query, or MIR dialect fails closed;
- no HIR/MIR/compiler-local identity is yet exposed publicly.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test session)
(cd rust-tools/rust2vir && cargo test --locked --test mir_access)
(cd rust-tools/rust2vir && cargo clippy --locked --all-targets -- -D warnings)
git diff --check
```

### RUST-03-T09 Implement the HIR Subset, Purity, and Conservative Call Closure

Status: Pending

Depends on: RUST-03-T08.

Inputs:

- Rust item/type/control-flow/purity rules in design section 9;
- HIR conformance vectors.

Likely touched files:

- `rust-tools/rust2vir/src/hir_check.rs`
- `rust-tools/rust2vir/src/call_closure.rs`
- `rust-tools/rust2vir/tests/hir_subset.rs`
- `rust-tools/rust2vir/tests/call_closure.rs`

Tasks:

1. Resolve the selected canonical free function and validate every source
   function in its compiler-resolved direct-call closure, including calls in
   source-dead branches.
2. Enforce exact accepted item visibility, identifier, function signature,
   constants, struct/type, path/module, attribute, statement, expression,
   control-flow, type, move/copy, and no-drop rules; unknown or expanded forms
   reject.
3. Enforce purity over referenced constants/types and closure members: no
   static/thread-local/external/I/O/nondeterministic/interior-mutability/drop or
   intrinsic access.
4. Build the conservative HIR call graph, reject direct/indirect recursion and
   cycles, and emit a deterministic closure set independent of later MIR dead
   code removal.
5. Record source signature names and compiler-resolved identities needed for
   contract resolution without exposing DefId/crate disambiguators.

Deliverables:

- fail-closed HIR validator and exact source dependency closure.

Acceptance criteria:

- every Rust subset positive/negative HIR vector has one deterministic result;
- inherited/private and bare `pub` helpers are accepted; restricted visibility
  rejects;
- a source-dead recursive cycle still rejects.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test hir_subset)
(cd rust-tools/rust2vir && cargo test --locked --test call_closure)
(cd rust-tools/rust2vir && cargo clippy --locked --all-targets -- -D warnings)
git diff --check
```

### RUST-03-T10 Implement Typed Rust Contract Parsing and Attachment

Status: Pending

Depends on: RUST-03-T03, RUST-03-T09.

Inputs:

- design section 11 and contract vectors;
- shared VIR contract/hash implementation.

Likely touched files:

- `rust-tools/rust2vir/src/contract.rs`
- `rust-tools/rust2vir/src/contract_typecheck.rs`
- `rust-tools/rust2vir/tests/contracts.rs`
- `rust-tools/rust2vir/testdata/contracts`

Tasks:

1. Strictly parse every repeatable captured `mpk.rust.contract.v0` sidecar,
   reject duplicate keys/unknown fields/limits, and index by canonical function
   ID independent of caller option order.
2. Require exactly one contract for every HIR closure member and reject
   duplicate, unresolved, or unused files.
3. Validate semantic profile, pointer width, function, nonempty ensures, empty
   modifies/loops, forbidden panic, total termination, variables/results,
   literal types/ranges, operator/arity/type rules, nesting, and expression
   limits against compiler-resolved signatures.
4. Normalize parameter IDs to VIR arguments, preserve ordered expression trees,
   construct the language-neutral contract, and compute/recheck
   `MPK-CONTRACT-0.1`.
5. Prove whitespace-only raw JSON changes preserve normalized contract/VIR hash
   but change raw input and source-manifest hashes.

Deliverables:

- complete typed Rust contract set attached to the source closure.

Acceptance criteria:

- contract input order cannot affect canonical output;
- local variables are not contract-visible;
- aggregate values appear only in exact-typed equality/inequality positions as
  specified.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test contracts)
(cd rust-tools/rust2vir && cargo clippy --locked --all-targets -- -D warnings)
git diff --check
```

### RUST-03-T11 Lower Basic MIR with Stable IDs and Source Mapping

Status: Pending

Depends on: RUST-03-T08, RUST-03-T09, RUST-03-T10.

Inputs:

- MIR mapping and stable-ID rules in design sections 12.3-12.4;
- `RUST_DRIVER_PROTOCOL_V0.md` raw payload shape.

Likely touched files:

- `rust-tools/rust2vir/src/mir_lower.rs`
- `rust-tools/rust2vir/src/stable_id.rs`
- `rust-tools/rust2vir/src/source_map.rs`
- `rust-tools/rust2vir/tests/mir_basic.rs`
- `rust-tools/rust2vir/tests/stable_ids.rs`

Tasks:

1. Validate reachable MIR blocks and allow only version-frozen empty/storage
   statements plus constant/use/copy/whole-place no-drop move, plain local
   assignment, Boolean not, integer/Boolean comparisons, goto, Boolean switch,
   and single modeled return forms in this milestone.
2. Reject cleanup/unwind/drop/assert/arithmetic/aggregate/call/projection and
   every unknown form until its owning later milestone implements exact
   semantics; no placeholder `Unsupported` VIR node is emitted.
3. Rename arguments/results/user locals/temporaries/blocks exactly by source
   order and breadth-first false-before-true traversal, preserving instruction
   order and omitting only validated unreachable MIR.
4. Emit one standalone VIR function for every HIR closure member even if it has
   no reachable caller edge; use only reachable MIR edges for public ordering.
5. Convert spans to captured normalized input paths and UTF-8 byte ranges,
   reject expansion/external/invalid/synthetic required mappings, and build the
   total source-map data.
6. Emit raw lowered driver payload plus inventory and hashes and test repeat
   compiler runs for byte-identical stable IDs.

Deliverables:

- deterministic basic scalar/control-flow MIR lowering and source map.

Acceptance criteria:

- simple constants, local updates, comparisons, branches, and early returns
  lower to validator-accepted VIR;
- compiler-local IDs and paths do not enter public artifacts;
- unsupported later-phase MIR rejects rather than approximates.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test mir_basic)
(cd rust-tools/rust2vir && cargo test --locked --test stable_ids)
(cd rust-tools/rust2vir && cargo clippy --locked --all-targets -- -D warnings)
git diff --check
```

### RUST-03-T12 Emit the Public Envelope, Manifest, and Deterministic Diagnostics

Status: Pending

Depends on: RUST-03-T01 through RUST-03-T11.

Inputs:

- frontend/source-map/manifest/diagnostic vectors;
- generic `mpk-cli` frontend runner.

Likely touched files:

- `rust-tools/rust2vir/src/emit.rs`
- `rust-tools/rust2vir/src/manifest.rs`
- `rust-tools/rust2vir/src/diagnostics.rs`
- `rust-tools/rust2vir/src/bin/rust2vir.rs`
- `rust-tools/rust2vir/tests/frontend_envelope.rs`
- `release/bundles/bundle-registry.json`
- `crates/mpk-cli/build.rs`
- `crates/mpk-cli/tests/rust_frontend_runner.rs`
- `scripts/build-release-bundles.sh`
- `scripts/check-release-bundles.sh`
- `fixtures/rust-basic`

Tasks:

1. Release-build the completed skeleton main/driver and pinned toolchain,
   generate complete inventories, add every Rust profile/target tuple to the
   reviewed combined Go/Rust registry, rebuild `mpk-cli` with the new expected
   registry ID/hash, and validate the installed tree. Require exactly one
   registered subordinate named `rust2vir-driver` and preserve every existing
   Go tuple byte-for-byte apart from the registry root/hash fields derived from
   adding Rust.
2. Strictly consume the validated private artifact, recompute its payload and
   inventory hashes, and construct canonical public VIR, source map, and
   frontend-stage manifest using launcher-selected identities.
3. Populate exact Rust language configuration, target/cfg set, input kinds,
   units, toolchain/frontend components, release registry, semantic parameters,
   limit profile, and selection; omit every runtime locator.
4. Normalize diagnostics to stable family code, canonical function/path/span,
   bounded message, fixed ordering, and exact truncation marker while excluding
   raw Cargo/rustc prose, snippets, commands, environment, and host paths.
5. Emit one canonical public envelope plus LF with exact status/exit and no
   partial artifacts on non-success.
6. Run the generic Rust consumer and cross-check every repeated identity/hash,
   successful selection, VIR/map/manifest validity, and output limits.
7. Add simple positive fixtures plus preflight/source/HIR/contract/MIR/toolchain
   negatives and two-run byte-determinism/path-leak tests.

Deliverables:

- complete RUST-03 frontend skeleton accepted by the shared runner.

Acceptance criteria:

- basic single-function fixtures emit schema-valid deterministic envelopes;
- all preflight negatives reject without executing user build code;
- the installed combined registry launches the Rust main/driver/toolchain set,
  and all existing registered Go frontend tests still pass;
- compiler, frontend, source, contracts, VIR, map, and manifest remain labeled
  untrusted helper data;
- the RUST-03 phase review has zero findings.

Verification:

```sh
./scripts/check-release-bundles.sh --fixture all
(cd rust-tools/rust2vir && cargo test --locked --test frontend_envelope)
(cd rust-tools/rust2vir && cargo test --locked)
cargo test -p mpk-cli --test rust_frontend_runner
cargo test -p mpk-cli --test frontend_runner
git diff --check
```

### RUST-04-T01 Lower Checked Add, Subtract, Multiply, and Negate

Status: Pending

Depends on: RUST-03-T12.

Inputs:

- Rust checked-arithmetic matrix and pinned MIR pattern vectors;
- shared VIR safety-check validation.

Likely touched files:

- `rust-tools/rust2vir/src/mir_arithmetic.rs`
- `rust-tools/rust2vir/src/mir_lower.rs`
- `rust-tools/rust2vir/tests/checked_arithmetic.rs`
- `fixtures/rust-basic/arithmetic`

Tasks:

1. Recognize only the pinned `CheckedBinaryOp`/overflow-flag/assert pattern for
   signed and unsigned add/subtract/multiply and the pinned nonconstant signed
   negate overflow pattern at widths 8/16/32/64.
2. Prove the checked result and assertion refer to the same operands, type,
   flag, branch polarity, and continuation; consume each recognized assertion
   exactly once and reject any other use of the overflow flag.
3. Lower the value operation plus the exact canonical
   `integer_no_overflow` check; reject unchecked `BinaryOp`, orphan/missing/
   duplicated assertion, changed message kind, cleanup edge, and unsupported
   width/signedness.
4. Recognize a compiler-accepted leading-unary-minus typed literal, including
   minimum values, from source/HIR provenance and lower it as one `Const` with
   no negate check; reject source-error out-of-range literals.
5. Add below/at/above boundary values and a golden test that mutating any MIR
   pattern component causes deterministic rejection.

Deliverables:

- exact checked add/subtract/multiply/negate lowering.

Acceptance criteria:

- no normal Rust arithmetic reaches VIR without its required overflow check;
- minimum literal and nonconstant negation remain distinguishable;
- every emitted VIR module passes independent profile completeness validation.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test checked_arithmetic)
cargo test -p mpk-vc --test safety_profiles
git diff --check
```

### RUST-04-T02 Lower Division and Remainder Safety Patterns

Status: Pending

Depends on: RUST-04-T01.

Inputs:

- total div/rem equations and Rust safety rules in `VIR_V0.md`;
- pinned rustc assert-kind vectors.

Likely touched files:

- `rust-tools/rust2vir/src/mir_arithmetic.rs`
- `rust-tools/rust2vir/tests/div_rem.rs`
- `fixtures/rust-basic/div-rem`

Tasks:

1. Recognize primitive signed/unsigned division and remainder with the exact
   compiler zero-divisor and, for signed types, minimum/negative-one overflow
   assertion sequence.
2. Consume and bind each assertion to the owning operation exactly once;
   preserve deterministic check order `divisor_nonzero` then
   `signed_divrem_representable` for signed operations.
3. Lower total VIR value operations even though Rust acceptance additionally
   requires safety proofs; reject library methods, overloads, casts, unknown
   intrinsic forms, missing/extra/reordered semantic assertions, and cleanup.
4. Add signed/unsigned and width matrix fixtures, zero divisor, minimum divided
   or remainder by negative one, sufficient/insufficient preconditions, and
   handwritten total-operation vectors.

Deliverables:

- exact division/remainder lowering and panic-condition binding.

Acceptance criteria:

- signed representability checks cannot be omitted or attached to unsigned
  operations;
- accepted-but-unproved source still emits VIR and proof-pending safety VCs;
- changed compiler patterns fail golden tests.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test div_rem)
cargo test -p mpk-vc --test safety_profiles
git diff --check
```

### RUST-04-T03 Lower Bitwise and Full-Count Shift Operations

Status: Pending

Depends on: RUST-04-T01.

Inputs:

- Rust bitwise/shift semantics and cross-width count vectors.

Likely touched files:

- `rust-tools/rust2vir/src/mir_arithmetic.rs`
- `rust-tools/rust2vir/tests/bitwise_shift.rs`
- `fixtures/rust-basic/bitwise-shift`

Tasks:

1. Lower primitive integer `&`, `|`, `^`, unary bit-not, left shift,
   arithmetic right shift, and logical right shift with exact result types and
   no operator overloading.
2. Preserve the RHS count's full width and signedness rather than truncating to
   the LHS width; select arithmetic/logical right shift from resolved LHS type.
3. Bind the pinned compiler shift-overflow assertion and emit
   `shift_count_less_than_width`; add `shift_count_nonnegative` first when the
   RHS type is signed.
4. Reject missing/orphan/changed assertions, unsupported RHS types, casts,
   helper methods, cleanup, and any frontend attempt to reuse Go over-width
   semantics without the Rust safety check.
5. Cover narrower/wider signed/unsigned counts, negative counts, exactly width,
   above width, and all LHS widths in source and handwritten VIR vectors.

Deliverables:

- exact bitwise and cross-width shift lowering.

Acceptance criteria:

- full-count shift behavior matches the copied VIR equations;
- emitted safety checks are complete and canonically ordered;
- Go/Rust difference vectors remain deliberate and passing.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test bitwise_shift)
cargo test -p mpk-vc --test safety_profiles
git diff --check
```

### RUST-04-T04 Lower Fixed-Array Index Reads on Both Target Widths

Status: Pending

Depends on: RUST-04-T01, RUST-03-T11.

Inputs:

- array index rules, target-sized types, and both release target descriptors.

Likely touched files:

- `rust-tools/rust2vir/src/mir_projection.rs`
- `rust-tools/rust2vir/src/mir_lower.rs`
- `rust-tools/rust2vir/tests/array_index.rs`
- `fixtures/rust-basic/array-index`

Tasks:

1. Accept only read-only fixed-array projections whose base, element, length,
   and index types are compiler-resolved and whose projected operand is Copy;
   reject mutation, references, slices, user indexing, partial moves, and
   opaque projections.
2. Bind the exact bounds-assert pattern to the owning projection and emit one
   canonical `index_in_bounds` check over the original index type and fixed
   length.
3. Encode signed index lower/upper bounds and unsigned upper bound correctly;
   derive `usize`/`isize` width only from the mandatory target.
4. Run equivalent fixtures under i686 and x86_64 registered target bundles and
   require different semantic/VIR/manifest/VC hashes where target context
   differs.
5. Test zero/last/length/negative indices, missing/changed assertions, target
   mismatch, target-library digest mismatch, and insufficient preconditions.

Deliverables:

- target-explicit fixed-array read and bounds safety lowering.

Acceptance criteria:

- host pointer width never influences the result;
- index assertion cannot be consumed by a different projection;
- both target corpora pass their exact registered component checks.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test array_index)
cargo test -p mpk-vc --test safety_profiles
git diff --check
```

### RUST-04-T05 Integrate Runtime-Safety VCs and Close the Arithmetic Gate

Status: Pending

Depends on: RUST-04-T01 through RUST-04-T04.

Inputs:

- all Rust arithmetic/index fixtures;
- common WP, safety, VC v1, grouping, and policy-member models.

Likely touched files:

- `crates/mpk-vc/tests/rust_runtime_safety.rs`
- `rust-tools/rust2vir/tests/runtime_safety_e2e.rs`
- `fixtures/rust-basic/runtime-safety`

Tasks:

1. Run every lowered arithmetic, div/rem, shift, and index instruction through
   independent VIR safety completeness validation and `generate_program_vcs`.
2. Require every operation-safety member to appear exactly once in the owning
   function's panic-free group with the correct path assumptions and stable
   source-neutral ID.
3. Add sufficient-precondition fixtures that close through checked
   definitions/theory certificates and insufficient-precondition fixtures that
   remain proof-pending in non-strict mode and fail strict mode.
4. Add two clean-run determinism checks and mutate each compiler assertion or
   VIR check to prove rejection or golden failure rather than silent omission.
5. Review axiom reports and prove the work introduced no Rust semantic axiom.

Deliverables:

- end-to-end runtime-safety obligation coverage for the Rust scalar subset.

Acceptance criteria:

- every modeled panic condition has one canonical safety member;
- removing/changing an assertion or check fails deterministically;
- arithmetic phase fixtures generate stable VIR, VC v1, and grouped skeletons;
- the RUST-04 findings ledger is empty.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test runtime_safety_e2e)
cargo test -p mpk-vc --test rust_runtime_safety
cargo test -p mpk-vc --test vc_skeleton_v1
git diff --check
```

### RUST-05-T01 Lower Primitive Constants and Fixed-Array Construction

Status: Pending

Depends on: RUST-04-T05.

Inputs:

- Rust type/constant/array rules in design sections 9.1-9.4 and 12.3.

Likely touched files:

- `rust-tools/rust2vir/src/type_lower.rs`
- `rust-tools/rust2vir/src/const_lower.rs`
- `rust-tools/rust2vir/src/mir_aggregate.rs`
- `rust-tools/rust2vir/tests/arrays_constants.rs`
- `fixtures/rust-basic/arrays`

Tasks:

1. Lower referenced primitive `const` declarations with compiler-resolved
   accepted types and literal values; reject evaluated expressions, unsupported
   values, duplicate IDs, statics, layout-dependent constants, and unused
   out-of-closure declarations entering VIR.
2. Lower fixed-array types with literal or accepted primitive-constant lengths,
   enforce length and nesting limits, and preserve the element type exactly.
3. Recognize complete explicit-list MIR array aggregates and emit
   `MakeArray` with exact arity, element type, source order, and stable ID;
   repeated `[value; length]` syntax remains rejected by HIR.
4. Accept whole-array Copy or whole-place no-drop Move according to rustc;
   reject partial/projected moves and all element mutation.
5. Integrate array construction with index reads, contract whole-value
   equality/inequality, source maps, VIR validation, and canonical hashes.

Deliverables:

- constant declarations and complete by-value fixed-array lowering.

Acceptance criteria:

- array length and elements cannot be inferred from physical layout;
- source order is preserved and every aggregate element is typed;
- array construction/read fixtures produce stable VIR and VCs.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test arrays_constants)
cargo test -p mpk-vc --test program_encoding
git diff --check
```

### RUST-05-T02 Lower Nominal Structs, Fields, and Whole-Value Moves

Status: Pending

Depends on: RUST-05-T01.

Inputs:

- nominal struct and move/copy rules in design sections 9.2-9.4 and 12.3.

Likely touched files:

- `rust-tools/rust2vir/src/type_lower.rs`
- `rust-tools/rust2vir/src/mir_aggregate.rs`
- `rust-tools/rust2vir/src/mir_projection.rs`
- `rust-tools/rust2vir/tests/structs.rs`
- `fixtures/rust-basic/structs`

Tasks:

1. Lower only named accepted structs from the selected closure, using canonical
   nominal VIR IDs and declaration-order fields without ABI/layout metadata.
2. Recognize complete by-value struct aggregates with every field explicitly
   initialized exactly once; reject update syntax, missing/duplicate fields,
   unions, tuples, enums, and unsupported nested/drop types.
3. Lower direct read-only field projections and whole-struct Copy or whole-place
   no-drop Move; require projected operands to be Copy and reject field
   mutation, partial moves, downcasts, dereferences, and subslices.
4. Implement componentwise contract equality/inequality only for identical
   nominal types and declaration-order fields.
5. Cover nested accepted aggregate depth/field limits, source-map spans,
   use-after-move compiler rejection, and deterministic canonical ordering.

Deliverables:

- nominal by-value struct construction, reads, and safe ownership erasure.

Acceptance criteria:

- no field offset, padding, alignment, endian, discriminant, or niche appears
  in VIR;
- rustc borrow checking precedes move erasure and unsupported partial moves
  reject;
- struct fixtures generate stable contract and safety groups.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test structs)
cargo test -p mpk-vc --test program_encoding
git diff --check
```

### RUST-05-T03 Lower Contract-Bound Direct Static Calls

Status: Pending

Depends on: RUST-05-T02, RUST-03-T09, RUST-03-T10.

Inputs:

- direct call and HIR/MIR graph rules in design section 10.5;
- shared VIR call validator and contract hashing.

Likely touched files:

- `rust-tools/rust2vir/src/mir_call.rs`
- `rust-tools/rust2vir/src/mir_lower.rs`
- `rust-tools/rust2vir/src/call_closure.rs`
- `rust-tools/rust2vir/tests/static_calls.rs`
- `fixtures/rust-basic/calls`

Tasks:

1. Accept only MIR direct calls whose compiler-resolved target is an accepted
   same-crate free function in the conservative HIR closure, with exact
   signature/profile/parameter compatibility and one normalized contract.
2. Lower a call terminator plus its normal destination jump to `CallStatic`
   and continuation, with canonical argument/result IDs and repeated callee
   `contract_hash`; reject unwind/cleanup, external/intrinsic/method/trait/
   function-value/closure targets.
3. Reconstruct the reachable public call graph from emitted `CallStatic`
   instructions, reject cycles again, and separate it from HIR-only dead call
   edges while retaining all HIR closure functions as standalone VIR members.
4. Require every caller/callee contract, type, semantic context, source map, and
   source-manifest unit identity to cross-check before emission.
5. Test private/bare-public cross-module helpers, reordered contract options,
   source-dead calls, source-dead recursion, hash mismatch, cycle, dynamic call,
   and call in a short-circuit branch.

Deliverables:

- exact MIR `CallStatic` lowering bound to normalized callee contracts.

Acceptance criteria:

- caller option/contract file order cannot change output bytes;
- a source-dead callee remains validated and emitted without a false reachable
  call edge;
- no call can bypass contract or panic-free reasoning.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test static_calls)
cargo test -p mpk-vc --test call_wp
git diff --check
```

### RUST-05-T04 Integrate Call WP and Topological Declaration Dependencies

Status: Pending

Depends on: RUST-05-T03, VIR-01-T12.

Inputs:

- shared `CallStatic` WP and grouped skeleton implementation;
- call/dependency vectors in `VC_V1.md`.

Likely touched files:

- `crates/mpk-vc/tests/rust_calls.rs`
- `rust-tools/rust2vir/tests/call_wp_e2e.rs`
- `fixtures/rust-basic/calls`

Tasks:

1. Run Rust `CallStatic` modules through the shared WP engine and generate a
   callee-precondition contract member plus callee-panic-free member at every
   reachable call with the correct path assumptions.
2. Use callee postconditions for subsequent values/safety only through the
   exact checked callee contract dependency.
3. Emit functions and declarations in callee-first topological/canonical-ID
   order, contract before panic-free, with the exact own/callee dependency sets
   and no reverse, duplicate, or extra edge.
4. Bind each policy-displayed member to its containing declaration name/hash
   and require both selected-function groups plus all transitive generated
   dependencies for `mpk_verified`.
5. Add multi-level, diamond-call, repeated-call, branch-call, empty-panic,
   missing-edge, reversed-edge, and extra-edge fixtures.

Deliverables:

- deterministic inter-function VC and certificate dependency integration.

Acceptance criteria:

- generated declarations are acyclic and dependency-minimal;
- no individual conjunct is mislabeled as an independently checked theorem;
- reordered input functions/contracts preserve identical output bytes.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test call_wp_e2e)
cargo test -p mpk-vc --test rust_calls
cargo test -p mpk-vc --test vc_skeleton_v1
git diff --check
```

### RUST-05-T05 Complete the Positive Rust Source-to-VC Corpus

Status: Pending

Depends on: RUST-05-T01 through RUST-05-T04.

Inputs:

- minimum positive corpus in design section 19.1;
- all Rust frontend and common VC components.

Likely touched files:

- `fixtures/rust-basic/manifest.json`
- `fixtures/rust-basic/positive`
- `crates/mpk-vc/tests/rust_positive_corpus.rs`
- `rust-tools/rust2vir/tests/positive_corpus.rs`

Tasks:

1. Add all twelve required positive categories: Boolean/short-circuit, signed
   and unsigned Max, checked addition, minimum literal/negation, division,
   cross-width shifts, array bounds, struct/move, early return, two-function
   cross-module calls, both `usize` targets, and multi-file closure with an
   unrelated file.
2. For every fixture record canonical public envelope, VIR, source map,
   frontend manifest, VC v1, certificate skeleton, and expected diagnostic/
   profile metadata. Certificate bytes are added after proof assembly in
   RUST-06.
3. Run two clean snapshot/compiler executions and compare every byte; inspect
   for absolute/source-root/toolchain/temp path leakage.
4. Cross-check all profile-required safety members, contract hashes, source-map
   coverage, group partition, and declaration dependencies.
5. Review the complete accepted MIR-form inventory against the pinned compiler
   and close every gap or reject it explicitly.

Deliverables:

- complete stable positive Rust frontend and VC corpus.

Acceptance criteria:

- every design section 19.1 category is present and deterministic;
- every complete positive fixture generates property and safety VCs;
- no unrelated file or machine locator enters captured/canonical artifacts;
- the RUST-05 findings ledger is empty.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test positive_corpus)
cargo test -p mpk-vc --test rust_positive_corpus
cargo test -p mpk-vc --test rust_calls
git diff --check
```

### RUST-06-T01 Register the Rust Policy Strategy and Axiom Tuple

Status: Pending

Depends on: RUST-05-T05.

Inputs:

- strategy/profile rules in design sections 15.2-15.3;
- `POLICY_V1.md`, `AXIOM_POLICY_V0.md`, and current policy strategy metadata.

Likely touched files:

- `crates/mpk-api/src/policy_strategy.rs`
- `crates/mpk-api/tests/policy_strategy.rs`
- `crates/mpk-cli/src/policy_profile.rs` (new shared profile validator)
- `crates/mpk-cli/tests/policy_profiles.rs`
- `develop/templates/module_manifest.yaml`

Tasks:

1. Add `payment-policy-rust-alpha` as a distinct strategy profile whose exact
   tuple is Rust, `mpk.rust.checked.v0`, and `mvp-theory`; retain the Go tuple
   `payment-policy-alpha`/Go/`mpk.go.fixed.v0`/`zero-axiom` unchanged.
2. Keep strategy profile, checker profile, semantic profile, and axiom profile
   as distinct types/fields and validate crossed known tuples before frontend
   launch in CLI, evidence parsing, and explainer parsing.
3. Define language-neutral supported obligation patterns and Rust-specific
   readiness descriptions without cloning Go source assumptions into reports.
4. Add concrete package/release manifest fixtures whose checker and allowed
   axiom profiles admit exactly the Rust evidence selections; test rejected
   active/evidence/package mismatches.
5. Prove the work does not add or reinterpret an axiom category and that
   `mvp-theory` permits only existing reviewed concrete identities.

Deliverables:

- exact Rust strategy/semantic/axiom registration and package-policy fixtures.

Acceptance criteria:

- known crossed tuples reject rather than normalize to an unknown strategy;
- evidence cannot silently broaden an axiom allowlist;
- Go policy behavior and profile tuple remain stable.

Verification:

```sh
cargo test -p mpk-api --test policy_strategy
cargo test -p mpk-cli --test policy_profiles
cargo test -p mpk-cli package
git diff --check
```

### RUST-06-T02 Route Rust `policy scan` Through the Generic Runner

Status: Pending

Depends on: RUST-03-T12, RUST-06-T01.

Inputs:

- active generic policy scan route from GO-VIR-02;
- Rust frontend release bundle and basic corpus.

Likely touched files:

- `crates/mpk-cli/src/policy_scan.rs`
- `crates/mpk-cli/src/main.rs`
- `crates/mpk-cli/tests/rust_policy_scan.rs`

Tasks:

1. Accept the Rust language-specific selection fields while reusing the exact
   generic registry/profile/target/bundle/contract parsing and runner.
2. Resolve the registered Rust main+driver frontend bundle and registered
   nightly/target toolchain bundle; require exact registry assertions and no
   raw executable or toolchain locator.
3. Populate scan v1 from the validated Rust envelope using generic selection,
   semantic parameters, source/contract/VIR/map/manifest helper identities,
   stable diagnostics, and Rust readiness text.
4. Preserve source `rejected`, `source-error`, and `frontend-error` distinctions
   and never convert a proof-pending safety condition into source unsupported.
5. Test both targets, multiple contracts, bundle/registry mismatch, raw-path
   flag rejection, unsupported Rust, compiler error, protocol failure, and
   repeated byte determinism.

Deliverables:

- Rust policy scan on the same generic path as migrated Go.

Acceptance criteria:

- Go and Rust share the runner and scan schema with only the selection union
  varying;
- Rust scan launches the same snapshotted main/driver/toolchain set later used
  by verify;
- scan contains no checker/strategy/axiom claim.

Verification:

```sh
cargo test -p mpk-cli --test rust_policy_scan
cargo test -p mpk-cli --test policy_scan
git diff --check
```

### RUST-06-T03 Assemble Checked Group Certificates and Rust Evidence v1

Status: Pending

Depends on: RUST-05-T04, RUST-06-T01, RUST-06-T02.

Inputs:

- grouped skeleton, existing checked theory certificates, certificate v0
  encoder/checkers, and policy evidence v1.

Likely touched files:

- `crates/mpk-cli/src/program_certificate.rs`
- `crates/mpk-cli/src/policy_verify.rs`
- `crates/mpk-cli/src/policy_evidence.rs`
- `crates/mpk-cli/tests/program_certificate.rs`
- `crates/mpk-cli/tests/rust_policy_verify.rs`

Tasks:

1. Reuse the exact internal Rust scan result, generate VC v1 and grouped
   skeletons, and select only proof candidates/theory certificates whose
   checked payload is bound to the exact member proposition.
2. Deterministically assemble each function's contract and panic-free theorem
   proof from member proofs in the specification-frozen balanced conjunction
   shape, respecting parameter binders, implications, checked foundation
   imports, and exact generated declaration dependencies.
3. Encode one canonical certificate v0 artifact, attach the finalized generic
   source manifest bytes, and run the existing fast kernel and independent Go
   source-free checker over the same certificate bytes.
4. Recompute certificate/export/axiom-report hashes and require checker
   agreement plus active checker/axiom equality with evidence and package/
   release policy before emitting `mpk_verified`.
5. Populate evidence v1 trusted declarations, member-to-containing-declaration
   refs, transitive callee refs, helper artifacts, both manifest lifecycle
   hashes, VC hash, registered bundles, exact profiles, and canonical recipes.
6. Preserve non-strict proof-pending output and strict failure when any member
   lacks checked proof evidence; no successful frontend/VC/classifier result is
   promoted by itself.

Deliverables:

- Rust source-to-canonical-certificate verification and evidence orchestration.

Acceptance criteria:

- one exact `.mpcert` byte sequence is accepted by both checkers;
- every verified property is covered by an accepted containing declaration and
  transitive dependencies;
- checker disagreement, axiom mismatch, package-policy mismatch, or manifest
  mutation blocks verification;
- certificate v0 encoding and checker implementation remain unchanged.

Verification:

```sh
cargo test -p mpk-cli --test program_certificate
cargo test -p mpk-cli --test rust_policy_verify
./scripts/checker-agreement.sh
git diff --check
```

### RUST-06-T04 Add the Rust Payment-Policy Example and Close the Product Gate

Status: Pending

Depends on: RUST-06-T03.

Inputs:

- positive Rust corpus and payment-policy classification patterns;
- package/release profile fixtures.

Likely touched files:

- `examples/rust-payment-policy/Cargo.toml`
- `examples/rust-payment-policy/Cargo.lock`
- `examples/rust-payment-policy/src/lib.rs`
- `examples/rust-payment-policy/contracts`
- `examples/rust-payment-policy/README.md`
- `examples/rust-payment-policy/artifacts`
- `crates/mpk-cli/tests/rust_payment_policy.rs`
- `release-report.json`

Tasks:

1. Add a self-contained dependency-free Rust library example under the closed
   subset, with contracts for the selected function and every same-crate helper
   and sufficient preconditions for all property/call/safety obligations.
2. Generate and check in canonical envelope, VIR, source map, both manifests,
   VC v1, grouped skeleton, certificate, axiom report, policy scan/evidence,
   Markdown, and reproduction fixtures through explicit update mode.
3. Verify the example under `payment-policy-rust-alpha`, `mvp-strict`, and
   `mvp-theory`; cross-check the package manifest and release gate and run both
   source-free checkers.
4. Document source/VIR/compiler traceability separately from trusted
   certificate/checker evidence and provide exact registered-bundle
   reproduction commands through structured recipes.
5. Add an insufficient-precondition sibling fixture proving non-strict
   proof-pending and strict failure without source rejection.

Deliverables:

- reproducible Rust payment-policy example with complete checked evidence.

Acceptance criteria:

- the positive example reports no proof-pending or unsupported property and is
  accepted by both checkers;
- trusted/helper report sections are accurate and language-neutral;
- artifacts are byte-identical across two clean runs and contain no local path;
- the RUST-06 findings ledger is empty.

Verification:

```sh
(cd examples/rust-payment-policy && cargo test --locked)
cargo test -p mpk-cli --test rust_payment_policy
cargo test -p mpk-cli --test rust_policy_verify
python3 scripts/generate-release-report.py --check
git diff --check
```

### RUST-07-T01 Complete the Negative and Adversarial Rust Corpus

Status: Pending

Depends on: RUST-06-T04.

Inputs:

- complete design section 19.2 list;
- phase/status/diagnostic precedence in the Rust and frontend specs.

Likely touched files:

- `fixtures/rust-basic/negative`
- `fixtures/rust-basic/adversarial`
- `fixtures/rust-basic/manifest.json`
- `rust-tools/rust2vir/tests/negative_corpus.rs`
- `crates/mpk-cli/tests/rust_frontend_negative.rs`
- `scripts/check-rust-subset-coverage.py`

Tasks:

1. Add at least one focused fixture for every rejected source, type, item,
   attribute, macro, Cargo, module/path, move/mutation, contract, call,
   recursion, MIR, target, toolchain, registry, bundle, protocol, manifest,
   source-map, VC/group, policy, profile, recipe, and evidence case enumerated in
   design section 19.2.
2. Add mixed-failure fixtures proving fixed phase and same-phase precedence and
   proving later diagnostics/artifacts are absent after an earlier failure.
3. Assert exact status, exit, stable code, normalized path/span, diagnostic
   ordering, bounded message, and artifact-presence shape; never snapshot raw
   compiler prose.
4. Add source-dead recursion, inventory disagreement, concurrent mutation,
   helper/toolchain substitution, registry mutation, manifest lifecycle, and
   grouped-declaration attack cases.
5. Maintain a machine-readable coverage manifest mapping every normative
   rejection rule/code to one or more tests; fail when a rule has no test or a
   test names an unknown rule.

Deliverables:

- complete auditable negative/adversarial corpus and rule coverage manifest.

Acceptance criteria:

- every design section 19.2 bullet is mapped and tested;
- unsupported and operational failure never become accepted/ready/verified;
- no negative result contains a partial canonical artifact.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test negative_corpus)
cargo test -p mpk-cli --test rust_frontend_negative
python3 scripts/check-rust-subset-coverage.py
git diff --check
```

### RUST-07-T02 Enforce Every Diagnostic and Resource Boundary

Status: Pending

Depends on: RUST-07-T01.

Inputs:

- design sections 17-18 and every registered limit profile.

Likely touched files:

- `rust-tools/rust2vir/src/limits.rs`
- `rust-tools/rust2vir/src/diagnostics.rs`
- `rust-tools/rust2vir/tests/limits.rs`
- `crates/mpk-vc/tests/verification_limits.rs`
- `crates/mpk-cli/tests/frontend_limits.rs`
- `crates/mpk-cli/tests/policy_limits.rs`

Tasks:

1. Audit that release registry, input capture, contracts, source, MIR, driver,
   VIR, source map, manifest, child streams, frontend streams, VC/skeleton,
   generated certificate, evidence JSON, and Markdown limits are enforced by
   streaming counters before full allocation or output write.
2. Add exact below/at/above tests for every byte, count, nesting, path,
   aggregate, message, process, and output limit in the normative profiles.
3. Implement diagnostic per-message and combined-list truncation exactly:
   normalize, sort rejected features first and diagnostics second, reserve the
   fixed final marker, retain the longest fitting prefix, and preserve status.
   The final combined-list marker code is
   `RUST_LIMIT_DIAGNOSTICS_TRUNCATED` and records the omitted count.
4. Assign each boundary to its normative owner: installed-registry violations
   use `FRONTEND_REGISTRY_LIMIT`, Rust child stdout/stderr violations use
   `RUST_FRONTEND_CHILD_OUTPUT_LIMIT`, outer frontend-stream violations use
   `FRONTEND_PROTOCOL_LIMIT`, and downstream verification/policy violations
   use their exact `VC_LIMIT_*` or `POLICY_LIMIT_*` code. Prove none
   reclassifies source status or emits a partial trusted artifact.
5. Ensure wall-clock timeout is only an operational frontend error and never an
   acceptance-dependent source or proof result.

Deliverables:

- exact deterministic boundary enforcement across the whole helper pipeline.

Acceptance criteria:

- every normative limit has a passing below/at/above test;
- successful artifacts cannot exceed declared canonical size limits;
- truncation output is byte-deterministic and contains no unnormalized path or
  compiler detail.

Verification:

```sh
(cd rust-tools/rust2vir && cargo test --locked --test limits)
cargo test -p mpk-vc --test verification_limits
cargo test -p mpk-cli --test frontend_limits
cargo test -p mpk-cli --test policy_limits
git diff --check
```

### RUST-07-T03 Complete Parser and Protocol Fuzzing

Status: Pending

Depends on: RUST-07-T01, RUST-07-T02.

Inputs:

- public frontend, private driver, VIR, source-map, contract, manifest, VC, and
  policy parsers;
- existing `fuzz` package conventions.

Likely touched files:

- `fuzz/Cargo.toml`
- `fuzz/fuzz_targets/frontend_protocol.rs`
- `fuzz/fuzz_targets/vir_parser.rs`
- `fuzz/fuzz_targets/source_map_parser.rs`
- `fuzz/fuzz_targets/source_manifest_parser.rs`
- `fuzz/fuzz_targets/policy_v1.rs`
- `rust-tools/rust2vir/fuzz`
- `rust-tools/rust2vir/fuzz/fuzz_targets/driver_protocol.rs`
- `rust-tools/rust2vir/fuzz/fuzz_targets/rust_contract.rs`
- `scripts/check-fuzz-smoke.sh`

Tasks:

1. Retain and extend the VIR/source-map harnesses created in VIR-01-T12 and add
   bounded entry points for every remaining public/shared parser and isolated
   driver/contract parser without invoking Cargo, rustc, network, or arbitrary
   filesystem input.
2. Assert no panic, abort, unbounded recursion, uncontrolled allocation,
   acceptance of noncanonical JSON, duplicate-key loss, or partial-success
   artifact on arbitrary bytes.
3. Seed corpora from all normative valid/invalid/boundary vectors and preserve
   discovered minimal regressions as deterministic unit fixtures.
4. Register every harness in its fuzz manifest and add
   `scripts/check-fuzz-smoke.sh`, which invokes each target for exactly 256
   runs with libFuzzer seed 1 from its checked-in seed corpus. Document
   unbounded `cargo fuzz run` commands separately; elapsed fuzz time never
   participates in artifact acceptance.
5. Keep frontend fuzz dependencies outside the trusted checker dependency
   graph and the isolated nightly project outside the stable workspace.

Deliverables:

- fuzz coverage for every new external or cross-process parse boundary.

Acceptance criteria:

- each parser named in design section 19.3 has a harness;
- smoke runs complete without crashes and regression seeds pass as unit tests;
- fuzz targets cannot execute a user project.

Verification:

```sh
./scripts/check-fuzz-smoke.sh
(cd rust-tools/rust2vir && cargo test --locked)
cargo test -p mpk-vc
cargo test -p mpk-cli
git diff --check
```

### RUST-07-T04 Add the VIR Interpreter and Differential Determinism Suite

Status: Pending

Depends on: RUST-05-T05, RUST-07-T02.

Inputs:

- total VIR equations, positive corpus, hostile-environment rules, and design
  section 19.3.

Likely touched files:

- `crates/mpk-vc/src/interpreter.rs`
- `crates/mpk-vc/tests/vir_differential.rs`
- `rust-tools/rust2vir/tests/differential.rs`
- `scripts/check-artifact-paths.py`

Tasks:

1. Implement a small test-only or explicitly untrusted VIR interpreter for
   accepted total scalar, branch, aggregate, projection, and static-call value
   semantics plus evaluation of modeled safety conditions.
2. Compare accepted Rust execution and panic behavior with VIR over exhaustive
   small widths and deterministic generated inputs; compare migrated Go/VIR for
   the shared operation corpus.
3. Include zero divisors, signed minimum/negative-one, over-width and negative
   shifts, bounds edges, short-circuit guards, early returns, aggregate values,
   and multi-function calls.
4. Run identical captures/lowering on two clean trees and under hostile Cargo,
   rustup, compiler, locale, timezone, proxy, credential, home, temp, and
   working-directory values; compare canonical bytes and hashes.
5. Scan canonical artifacts and normalized diagnostics for workspace, source
   root, home, toolchain, temp, hostname, timestamp, and sentinel strings.

Deliverables:

- cross-language differential and host-independence confidence suite.

Acceptance criteria:

- all accepted corpus executions agree for value and modeled panic behavior;
- known Go/Rust semantic differences match profile vectors rather than being
  treated as drift;
- repeated canonical outputs are byte-identical and path-clean.

Verification:

```sh
cargo test -p mpk-vc --test vir_differential
(cd rust-tools/rust2vir && cargo test --locked --test differential)
python3 scripts/check-artifact-paths.py
git diff --check
```

### RUST-07-T05 Add Clean CI, Compiler Upgrade Procedure, and Final Release Gate

Status: Pending

Depends on: RUST-07-T01 through RUST-07-T04.

Inputs:

- every phase gate, release report, documentation set, and obsolete-interface
  search.

Likely touched files:

- `.github/workflows/rust-frontend.yml`
- `scripts/check-rust-frontend.sh`
- `scripts/check-all.sh`
- `scripts/generate-release-report.py`
- `develop/docs/rust-frontend-toolchain-upgrade.md`
- `develop/docs/05_rust_frontend_design.md`
- `develop/docs/05_rust_frontend_design-todo.md`
- `develop/roadmap/RELEASE_GATES.md`
- `README.md`
- `develop/README.md`
- active ProofOps and integration docs
- `release-report.json`

Tasks:

1. Add a clean Linux CI job that materializes only registered bundles, uses the
   exact pinned nightly and both target libraries, disables network during
   evidence runs, and executes frontend, Go migration, policy, both-checker,
   path, determinism, limit, fuzz-smoke, and obsolete-interface gates.
2. Add a local aggregate gate with the same command ordering and no implicit
   rustup/toolchain download during verification.
3. Document compiler upgrade as a reviewed registry/toolchain/commit/argument/
   MIR-adapter change that regenerates all MIR/VIR/map/manifest/VC/certificate
   goldens, reruns differential tests, and requires a new registry root hash;
   automatic upgrades are prohibited.
4. Extend release evidence with registry/bundle/target identities, both manifest
   hashes, VIR/VC/certificate hashes, checker agreement, axiom report/profile
   equality, determinism/path gates, and the Rust example result.
5. Update active user/developer/ProofOps documentation without overstating
   source correspondence or trusting rustc/frontend/VIR; keep GIR-era docs only
   as labeled historical records.
6. Run a complete design-to-task-to-repository review, fix all findings, run
   strict obsolete-interface searches, mark the design and todo as completed
   migration records, and close only with a clean checkout and empty findings
   ledger.

Deliverables:

- reproducible clean-machine Rust release gate and reviewed upgrade process.

Acceptance criteria:

- all Rust gates, migrated Go gates, both target corpora, both checkers,
  determinism, path sanitation, limits, fuzz smoke, and release report pass;
- no active production/interface/documentation GIR or v0 policy/AI hit remains;
- certificate v0, trust boundary, checker semantics, and axiom categories are
  unchanged;
- the full review ledger contains zero findings.

Verification:

```sh
./scripts/check-rust-frontend.sh
./scripts/check-no-active-gir.sh --strict
./scripts/check-all.sh
cargo test --workspace
(cd go-tools/go2vir && go test -count=1 ./...)
(cd rust-tools/rust2vir && cargo test --locked)
python3 scripts/generate-release-report.py --check
git diff --check
```

## Requirement Traceability

| Design section | Owning implementation milestones |
| --- | --- |
| 1-4 decisions, goals, non-goals | VIR-00-T01, VIR-00-T02, VIR-00-T10, GO-VIR-02-T12 |
| 5 trust boundary | VIR-00-T10, GO-VIR-02-T06, RUST-06-T03, RUST-07-T05 |
| 6 architecture | VIR-01-T03 through VIR-01-T12, GO-VIR-02-T05, RUST-03-T01 |
| 7 source/HIR/MIR layers | RUST-03-T03, RUST-03-T07 through RUST-03-T11 |
| 8.1 pinned compiler and bundles | VIR-00-T03, VIR-01-T02, GO-VIR-02-T05, RUST-03-T01, RUST-03-T05, RUST-03-T08, RUST-03-T12, RUST-07-T05 |
| 8.2 Cargo preflight | RUST-03-T02 through RUST-03-T05 |
| 8.3 sanitized compilation | RUST-03-T05, RUST-03-T06, RUST-03-T08 |
| 9 Rust subset and purity | VIR-00-T09, RUST-03-T07, RUST-03-T09, RUST-05-T01, RUST-05-T02 |
| 10 semantic profile | VIR-00-T02, VIR-01-T06 through VIR-01-T10, RUST-04-T01 through RUST-04-T05 |
| 11 Rust contracts | VIR-00-T02, RUST-03-T10, RUST-05-T03 |
| 12 VIR | VIR-00-T02, VIR-01-T01, VIR-01-T03, VIR-01-T04, GO-VIR-02-T04, RUST-03-T11 |
| 13 frontend protocol | VIR-00-T04, GO-VIR-02-T02, GO-VIR-02-T05, RUST-03-T06, RUST-03-T12 |
| 14 source manifest | VIR-00-T04, VIR-01-T05, GO-VIR-02-T03, GO-VIR-02-T08, RUST-03-T12, RUST-06-T03 |
| 15 CLI, policy, evidence, AI | VIR-00-T06, VIR-00-T07, GO-VIR-02-T06 through GO-VIR-02-T10, GO-VIR-02-T12, RUST-06-T01 through RUST-06-T04 |
| 16 VC generation | VIR-00-T05, VIR-01-T07 through VIR-01-T12, RUST-04-T05, RUST-05-T04 |
| 17 diagnostics | VIR-00-T04, RUST-03-T12, RUST-07-T01, RUST-07-T02 |
| 18 limits | VIR-00-T02 through VIR-00-T06, VIR-01-T08, VIR-01-T11, RUST-03-T02 through RUST-03-T06, RUST-07-T02 |
| 19.1 positive corpus | RUST-05-T05, RUST-06-T04 |
| 19.2 negative/adversarial corpus | RUST-07-T01, RUST-07-T02 |
| 19.3 translation confidence | GO-VIR-02-T11, RUST-07-T03, RUST-07-T04 |
| 19.4 migration gates | VIR-00-T01, GO-VIR-02-T01, GO-VIR-02-T11, GO-VIR-02-T12, RUST-07-T05 |
| 20 sequence and exit gates | every milestone in its matching phase |
| 21 file/module impact | all implementation milestones; removals are owned by GO-VIR-02-T12 |
| 22 alternatives | execution rules 1-11; no adapter, dual release path, GIR reinterpretation, syn-only, textual MIR, LLVM, or trusted frontend task exists |
| 23 risks | bundle/compiler risks: RUST-03; semantics: VIR-01/RUST-04; migration: GO-VIR-02; hardening: RUST-07 |
| 24 completion criteria | GO-VIR-02-T12, RUST-06-T04, RUST-07-T05 |

## Final Handoff Checklist

The full program is complete only when:

- every milestone above is `Completed` and every dependency is satisfied;
- all VIR-00 specifications and vectors are normative and internally
  consistent;
- all active Go and Rust source paths use the same VIR/VC/policy interfaces;
- strict obsolete-interface searches pass outside exact historical files;
- the Rust payment-policy certificate is accepted from identical canonical
  bytes by both source-free checkers;
- active checker and axiom profiles equal evidence and are permitted by the
  package/release policy and recomputed axiom report;
- all exact limit boundaries, adversarial cases, parser fuzz smoke tests,
  differential tests, two-run determinism checks, and path scans pass;
- `./scripts/check-all.sh`, `./scripts/check-rust-frontend.sh`, and release
  report validation pass from a clean checkout with registered bundles only;
- final documentation distinguishes mathematical certificate proof from
  untrusted source/compiler/frontend/VIR traceability;
- the implementation review ledger is empty and the working tree is clean.
