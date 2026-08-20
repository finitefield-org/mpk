# GIR-to-VIR Removal Inventory

Status: pre-cutover baseline for `VIR-00-T01`. The active release still uses
GIR. This report records removals and regeneration work; it does not authorize
an adapter or change an active parser.

Baseline revision: `1b3fb3c778b9301d8a214c4bbd3cd8aafa6bb8d4`.

## Gate contract

`scripts/check-no-active-gir.sh` reads one matcher per JSON line from
`develop/migrations/gir-to-vir-obsolete-terms.txt`. Every matcher has a final
removal owner of `GO-VIR-02-T12`; preparatory work may belong to an earlier
milestone. The four frozen matcher scopes are:

| Scope | Exact behavior |
| --- | --- |
| `exact-global-token` | Matches the case-sensitive token with identifier boundaries. It is used for schema IDs, fields, flags, hash names, public symbols, and exact prose labels. |
| `exact-path` | Matches the normalized repository path, descendants of an explicitly named obsolete directory, and an exact literal reference to that path. Directory matching detects obsolete content; it never excludes a directory. |
| `schema-qualified-json-field` | Matches a field only below an object whose inherited `schema` or `schema_version` is one of the named retired schemas. Non-JSON code/documentation is matched only when the same file contains both the exact retired schema and field. |
| `type/variant-qualified-code-symbol` | Matches the complete qualified symbol, allowing whitespace only around `::`. A same-named variant on another type is not a match. |

The gate has two modes. `--audit` reports active hits and exits successfully
before cutover. `--strict` reports the same hits and fails. Both modes first
validate all exclusion classes and execute the focused positive and negative
fixtures. `GO-VIR-02-T12` must make strict mode pass; it must not weaken or
remove a matcher to do so.

## Disjoint exact-file classes

No directory is allowlisted. Every path below is a regular file, and the gate
rejects a missing, unknown, duplicate, symlinked, overlapping, or directory
entry before scanning source files.

### Scanner implementation and metadata

- `develop/migrations/gir-to-vir-obsolete-terms.txt`
- `scripts/check-no-active-gir.sh`

These files are executable scanner machinery, not historical records.

### Focused self-test fixtures

The exact list is the `fixtures` array in
`develop/migrations/gir-to-vir-search-fixtures/manifest.json`, including the
manifest itself. The gate independently enumerates every regular file below
that root and requires set equality. Positive fixtures exercise all four
matcher scopes. Negative fixtures prove that:

- `mpk.vir.v0` is not the retired GIR schema;
- `go-tools/go2vir/main.go` is not the retired frontend path;
- package-manifest `allowed_axiom_profiles` remains governed by
  `develop/specs/AXIOM_POLICY_V0.md` and is accepted;
- structured policy-v1 `reproduction_commands` are not the v0 string model;
- `GoSource` and `Gir` variants on an unrelated type are not the retired
  policy/sanitizer variants.

### Historical records

| Exact file | Activation | Classification |
| --- | --- | --- |
| `develop/specs/GIR_V0.md` | immediate | historical-allowlist |
| `develop/specs/GO_SUBSET_V0.md` | immediate | historical-allowlist |
| `develop/specs/AI_API_V0.md` | immediate | historical-allowlist |
| `develop/specs/CERT_V0.md` | immediate | historical-allowlist |
| `develop/specs/TRUST_BOUNDARY_V0.md` | immediate | historical-allowlist |
| `develop/specs/UNSAFE_POLICY_V0.md` | immediate | historical-allowlist |
| `develop/migrations/gir-to-vir-inventory.md` | immediate | historical-allowlist |
| `develop/migrations/go-gir-semantic-baseline.json` | immediate | historical-allowlist |
| `develop/docs/05_rust_frontend_design.md` | conditional | historical-allowlist after the marker below exists in both design records |
| `develop/docs/05_rust_frontend_design-todo.md` | conditional | historical-allowlist after the marker below exists in both design records |

The exact atomic marker is:

```text
GIR_CUTOVER_STATUS: complete; RUST_PHASES: active; RETAINED_GIR_TERMINOLOGY: historical
```

Until `GO-VIR-02-T12` adds that line to both design records, neither design
path is excluded from the active scan. This prevents one partially updated
document from hiding live terminology.

## Retired schema inventory

Every schema below is a case-sensitive global token. The disposition describes
the active artifact; the frozen specification itself follows the historical
allowlist above.

| Retired schema | Current producer/consumer | Disposition | Preparation owner | Removal owner |
| --- | --- | --- | --- | --- |
| `mpk.gir.v0` | `go2gir`, `mpk-vc`, examples, VC tests | remove; regenerate as `mpk.vir.v0` | VIR-00-T02, VIR-01-T03/T04, GO-VIR-02-T04/T11 | GO-VIR-02-T12 |
| `mpk.gir.emit.v0` | Go canonical JSON/binary emission | remove; no VIR compatibility wrapper | GO-VIR-02-T02/T04 | GO-VIR-02-T12 |
| `mpk.go2gir.cli.v0` | Go frontend and policy runner | remove; replace with generic frontend envelope | VIR-00-T04, GO-VIR-02-T02/T05 | GO-VIR-02-T12 |
| `mpk.go.source_manifest.v0` | Go frontend manifest | remove; regenerate generic two-stage manifest | VIR-00-T04, VIR-01-T05, GO-VIR-02-T03/T04 | GO-VIR-02-T12 |
| `mpk.vc.cert_skeleton.v0` | `mpk-vc` emitter and generated fixtures | remove; regenerate VC v1 grouped skeletons | VIR-00-T05, VIR-01-T11/T12, GO-VIR-02-T11 | GO-VIR-02-T12 |
| `mpk.policy.scan.v0` | `mpk-cli` scan JSON and tests | remove; regenerate policy scan v1 | VIR-00-T06, GO-VIR-02-T06/T07/T11 | GO-VIR-02-T12 |
| `mpk.policy.evidence.v0` | verify, report, explainer, examples | remove; regenerate evidence v1 | VIR-00-T06, GO-VIR-02-T06/T08/T09/T11 | GO-VIR-02-T12 |
| `mpk.ai.explain.request.v0` | sanitized request and golden hash | remove; regenerate request v1 | VIR-00-T07, GO-VIR-02-T09/T11 | GO-VIR-02-T12 |
| `mpk.ai.explanation.v0` | explanation report/parser | remove; regenerate explanation v1 | VIR-00-T07, GO-VIR-02-T09/T11 | GO-VIR-02-T12 |
| `mpk.evidence-explainer.v0` | prompt template identity | remove; regenerate prompt v1 | VIR-00-T07, GO-VIR-02-T09/T11 | GO-VIR-02-T12 |

`mpk.ai.explanation.response.v0` is intentionally absent: the provider response
shape remains unchanged and is a retained negative control at implementation
review.

## Hash, wrapper, field, status, and route inventory

| Obsolete interface | Matcher scope | Current locations/meaning | Disposition | Preparation owner | Removal owner |
| --- | --- | --- | --- | --- | --- |
| `MPK_GIR_V0` | exact-global-token | canonical GIR binary magic | remove without alias | GO-VIR-02-T02/T04 | GO-VIR-02-T12 |
| `gir_emit` | exact-global-token | CLI envelope binary/canonical-JSON member | rename/regenerate into generic canonical artifacts | GO-VIR-02-T02/T04/T11 | GO-VIR-02-T12 |
| `gir-lowered` | exact-global-token | Go frontend success status | remove; use frontend-v0 success status | GO-VIR-02-T02 | GO-VIR-02-T12 |
| `source_gir_hash` | exact-global-token | VC, skeleton, classifier, evidence, fixtures | rename/regenerate as schema-qualified source IR identity | VIR-00-T05/T06, VIR-01-T11, GO-VIR-02-T06/T08/T11 | GO-VIR-02-T12 |
| `gir_hash` | exact-global-token | GIR object, source manifest, helper evidence, templates | rename/regenerate as VIR/source-IR hash | VIR-01-T04/T05, GO-VIR-02-T04/T11 | GO-VIR-02-T12 |
| `gir_sha256` | exact-global-token | policy scan/evidence hash | rename/regenerate | GO-VIR-02-T06/T08/T11 | GO-VIR-02-T12 |
| `go2gir_sha256` | exact-global-token | policy scan frontend digest | replace with registered bundle identity | GO-VIR-02-T05/T06/T07 | GO-VIR-02-T12 |
| `--go2gir` | exact-global-token | released scan/verify executable-path flag | remove; no raw executable fallback | GO-VIR-02-T07/T08 | GO-VIR-02-T12 |
| `go2gir`, `Go2Gir`, `GO2GIR` | exact-global-token | executable/module/name variants | remove or rename/regenerate as `go2vir` | GO-VIR-02-T02/T04/T11 | GO-VIR-02-T12 |
| `GIR` | exact-global-token | active prose, comments, diagnostics, examples | remove from active records; keep only exact historical files | GO-VIR-02-T11 | GO-VIR-02-T12 |
| `POST /gir/import` and `/gir/import` | exact-global-token | AI API v0 import route | remove; v1 exposes only `/vir/import` | VIR-00-T07, GO-VIR-02-T10 | GO-VIR-02-T12 |
| `Std.Go.Base` | exact-global-token | VC foundational type names and fixtures | rename/regenerate as `Std.Program.Base` | VIR-01-T06, GO-VIR-02-T11 | GO-VIR-02-T12 |

Recorded hash bytes in the semantic baseline are audit anchors, not equality
requirements. The comparator must fail on lost semantics, missing obligations,
changed rejection classes, or checker disagreement—not on an explained schema
or domain-separated hash change.

## Rust and Go type/public-export inventory

The Rust GIR model in `crates/mpk-vc/src/gir.rs` exports or feeds the following
obsolete names. They are exact global tokens so a moved remnant is still
detected:

```text
GIR_SCHEMA_VERSION
GirModule GirPackage GirFunction GirContracts GirContractExpr GirLoopContract
GirBinding GirType GirTypeKind GirFieldType GirBlock GirInstruction
GirInstructionKind GirField GirTerminator GirTerminatorKind GirValue
GirIntLiteral GirRejectedFeature GirImportError GirValueAtomCount
```

The Go frontend owns the following serialized/model names:

```text
girArrayType girBinaryMagic girBinding girBlock girCanonicalModule
girContractExpr girContracts girEmission girEmitSchema girField girFieldType
girFunction girFunctionLowerer
girHash girInstruction girIntLiteral girLoopContract girModule girNamedType
girPackage girPackageLowerer girResult girSchemaVersion girStructType
girTerminator girType girTypeFromGoType girTypeOf girTypeOfObject girValue
GIRHash GIREmission
```

The old Rust GIR-specific public, associated, and internal functions are:

```text
import_gir_json encode_gir_type encode_gir_value empty_for_gir
expr_type_from_gir_type
generate_straight_line_vcs generate_branch_vcs generate_loop_vcs
generate_safety_vcs
```

The Go-only canonical/lowering helpers are:

```text
canonicalGIRJSON canonicalGIRBinary hashGIRBinary emitCanonicalGIR lowerToGIR
```

All `STD_GO_BASE_*` exported constants are independently matched in addition to
their serialized `Std.Go.Base` values. This prevents a stale public constant
name from surviving after its string value changes.

`VIR-01-T03` through `VIR-01-T10` prepare the language-neutral replacements.
`GO-VIR-02-T04` ports the Go model directly without a serialized adapter.
`GO-VIR-02-T12` owns removal of every old type/export from the public and
internal production surface.

## Policy and AI typed-context inventory

| Design term | Resolved exact context | Matcher scope | Disposition | Preparation owner | Removal owner |
| --- | --- | --- | --- | --- | --- |
| `PolicyScanTarget` | full type | exact-global-token | rename/regenerate into language-neutral selection union | GO-VIR-02-T06/T07 | GO-VIR-02-T12 |
| `PolicyEvidenceTarget` | full type | exact-global-token | rename/regenerate into language-neutral evidence target | GO-VIR-02-T06/T08 | GO-VIR-02-T12 |
| `go_source` | serialized helper kind | exact-global-token | rename/regenerate as `source` | GO-VIR-02-T06/T09/T11 | GO-VIR-02-T12 |
| design shorthand `GoSource` | `PolicyHelperArtifactKind::GoSource` | type/variant-qualified-code-symbol | rename/regenerate as generic source kind | GO-VIR-02-T06/T09 | GO-VIR-02-T12 |
| `PolicyHelperArtifactKind::Gir` | exact variant | type/variant-qualified-code-symbol | rename/regenerate as verification-IR kind | GO-VIR-02-T06/T09 | GO-VIR-02-T12 |
| `SanitizedArtifactKind::GoSource` | exact variant | type/variant-qualified-code-symbol | rename/regenerate | GO-VIR-02-T09 | GO-VIR-02-T12 |
| `SanitizedArtifactKind::Gir` | exact variant | type/variant-qualified-code-symbol | rename/regenerate | GO-VIR-02-T09 | GO-VIR-02-T12 |
| policy-evidence `allowed_axiom_profiles` | field below schema `mpk.policy.evidence.v0` only | schema-qualified-json-field | remove; replace with one explicit selected `axiom_profile` | GO-VIR-02-T06/T08/T09 | GO-VIR-02-T12 |
| v0 `reproduction_commands` | field below schema `mpk.policy.evidence.v0` only | schema-qualified-json-field | rename/regenerate as structured argv recipes | GO-VIR-02-T06/T08 | GO-VIR-02-T12 |
| `PolicyEvidenceReproductionCommand` | v0 free-form command type | exact-global-token | remove | GO-VIR-02-T06/T08 | GO-VIR-02-T12 |
| `minimal-v0` | explainer redaction profile | exact-global-token | remove; regenerate as `minimal-v1` | GO-VIR-02-T09/T11 | GO-VIR-02-T12 |

The package-manifest field
`policy.allowed_axiom_profiles` under schema `mpk.package.v0` remains active.
The search gate's package-manifest negative fixture makes a global grep for
that field a test failure.

## Exact GIR-only path inventory

| Exact path | Disposition | Preparation owner | Removal owner |
| --- | --- | --- | --- |
| `go-tools/go2gir` | remove after direct `go2vir` port | GO-VIR-02-T02 through T04 | GO-VIR-02-T12 |
| `crates/mpk-vc/src/gir.rs` | remove after VIR model consumers are complete | VIR-01-T03/T04 | GO-VIR-02-T12 |
| `examples/max64/gir.json` | rename/regenerate | GO-VIR-02-T11 | GO-VIR-02-T12 |
| `examples/order_policy/gir.json` | rename/regenerate | GO-VIR-02-T11 | GO-VIR-02-T12 |
| `examples/payment_policies/discount/gir.json` | rename/regenerate | GO-VIR-02-T11 | GO-VIR-02-T12 |
| `examples/payment_policies/fee/gir.json` | rename/regenerate | GO-VIR-02-T11 | GO-VIR-02-T12 |
| `examples/payment_policies/points/gir.json` | rename/regenerate | GO-VIR-02-T11 | GO-VIR-02-T12 |
| `examples/payment_policies/refund/gir.json` | rename/regenerate | GO-VIR-02-T11 | GO-VIR-02-T12 |
| `examples/payment_policies/reserve/gir.json` | rename/regenerate | GO-VIR-02-T11 | GO-VIR-02-T12 |

Other generated `vc.json`, `vc_skeleton.json`, policy/evidence JSON, and report
paths are not classified by filename alone because those filenames remain
valid in v1. Their retired schemas and fields are content matchers, and their
bytes are regenerated by `GO-VIR-02-T11` before `GO-VIR-02-T12` installs them.

## Current active-hit families

Every reported occurrence inherits the matcher's nonempty removal owner. This
table records the preparatory owner and prevents a path family from being lost
behind the common final cutover owner.

| Current family | Examples | Classification | Preparation owner | Final removal owner |
| --- | --- | --- | --- | --- |
| GIR importer/model/canonicalizer | `crates/mpk-vc/src`, `go-tools/go2gir` | remove or rename/regenerate | VIR-01-T03/T04, GO-VIR-02-T02/T04 | GO-VIR-02-T12 |
| Go-specific type/expression/WP/safety exports | `type_encode.rs`, `expr_encode.rs`, `wp*.rs`, `loops.rs`, `safety.rs` | rename/regenerate, then remove legacy exports | VIR-01-T06 through T10 | GO-VIR-02-T12 |
| Policy scan/verify/evidence/report | `crates/mpk-cli/src/policy_*.rs` and tests | rename/regenerate | GO-VIR-02-T06 through T08 | GO-VIR-02-T12 |
| AI explanation sanitizer/prompt/output | `ai_explain.rs`, its tests, Vertex documentation | rename/regenerate | GO-VIR-02-T09/T11 | GO-VIR-02-T12 |
| AI API import boundary | AI v0 specification and staged v1 router | historical-allowlist for v0 spec; remove active v0 route | GO-VIR-02-T10 | GO-VIR-02-T12 |
| Go/VC/payment fixtures and examples | `fixtures/go-*`, `fixtures/vc-alpha`, `examples/*` | rename/regenerate | GO-VIR-02-T11 | GO-VIR-02-T12 |
| Release evidence and templates | `release-report.json`, `develop/templates`, generated hashes | rename/regenerate | GO-VIR-02-T11 | GO-VIR-02-T12 |
| Build/check/release scripts | `check-all.sh`, `checker-agreement.sh`, release report generation | rename/regenerate | GO-VIR-02-T11 | GO-VIR-02-T12 |
| Root/developer/architecture/ProofOps docs and CI | `README.md`, `develop/README.md`, active docs, `.github` | remove active terminology or regenerate commands | GO-VIR-02-T11 | GO-VIR-02-T12 |
| Frozen v0 specifications and this migration record | exact historical table above | historical-allowlist | VIR-00-T10 and GO-VIR-02-T12 governance updates | GO-VIR-02-T12 gate activation |
| Two Rust migration design records | exact conditional records above | historical-allowlist only after atomic marker | GO-VIR-02-T12 | GO-VIR-02-T12 |

## Semantic baseline coverage

`develop/migrations/go-gir-semantic-baseline.json` records:

- all 100 named alpha functions in the exact `34 + 33 + 33` source groups and
  their source/manifests hashes;
- the 1,056 alpha postcondition obligations and theorem range;
- all five positive payment-policy functions, eight obligations each, theorem
  ranges, classification patterns, theory-goal counts, and GIR/VC/skeleton
  audit hashes;
- both current negative manifests plus focused Go frontend/contract rejection
  cases and their stable semantic reasons;
- contract defaults and rejection rules;
- partial- and total-correctness loop obligation kinds;
- the explicit fixed-width conversion term anchor;
- division, signed-shift, and array-index runtime-safety predicates;
- the current release certificate's agreeing Rust source-free/reference
  checker verdicts and hashes;
- the payment reserve's checked theory evidence, including the explicit fact
  that its current v0 evidence has no program-certificate checker verdict.

That final absence is data, not success. `GO-VIR-02-T11` must create and check
the regenerated program certificates with both source-free checkers.

## Section 19.4 completeness checklist

The following exact design terms are represented above and in scanner metadata:

```text
go2gir Go2Gir GO2GIR mpk.go2gir.cli.v0 mpk.go.source_manifest.v0
mpk.gir.v0 mpk.gir.emit.v0 MPK_GIR_V0 gir_emit gir-lowered
source_gir_hash mpk.policy.scan.v0 mpk.policy.evidence.v0
mpk.vc.cert_skeleton.v0 go_source GoSource PolicyScanTarget
PolicyEvidenceTarget PolicyHelperArtifactKind::Gir
SanitizedArtifactKind::GoSource SanitizedArtifactKind::Gir
allowed_axiom_profiles gir_hash gir_sha256 go2gir_sha256
mpk.ai.explain.request.v0 mpk.ai.explanation.v0
mpk.evidence-explainer.v0 minimal-v0 reproduction_commands
PolicyEvidenceReproductionCommand POST /gir/import GIR-only paths
```

The schema inventory additionally contains all exact retired schemas mandated
by `VIR-00-T01`; the type/export and `Std.Go.Base` sections cover the removal
surface from design sections 15.1, 20, and 21.
