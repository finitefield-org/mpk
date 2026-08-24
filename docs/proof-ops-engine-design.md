# ProofOps Engine Support Design

Status: implemented product-enablement handoff

This document defines the MPK-side work needed to support the ProofOps product
and service repository in `../proof-ops`. MPK remains the verification engine:
it owns source-free proof acceptance, deterministic verification artifacts,
Go-policy lowering, VC generation, theory-backed proof construction, and
machine-readable evidence. ProofOps owns the customer-facing product,
Gemini-based analysis, web forms, pricing, reporting presentation, billing, and
XPRIZE submission operations.

## Product Context

ProofOps sells payment-policy verification workflows for small services. The
initial customer corpus remains Go; MPK also ships a frozen Rust product-gate
example that exercises the same VIR/policy/checker boundary. The initial use
cases are pure deterministic functions that decide reserve,
refund, discount, fee, points-redemption, and quota amounts. The product promise
must stay narrow:

- identify whether a function is MPK-ready;
- generate or validate safety contracts for payment policies;
- turn a supported policy function into reproducible verification artifacts;
- produce reviewable evidence for CI, customers, and XPRIZE judges;
- never treat AI output, source text, VIR, VC JSON, CI status, or report prose as
  proof evidence.

The MPK trust boundary remains unchanged. A payment policy is accepted only when
the relevant theorem is represented by canonical `.mpcert` bytes or checked
theory certificates and passes the configured checker policy.

The Rust example uses `payment-policy-rust-alpha`,
`mpk.rust.checked.v0`, `mvp-strict`, and `mvp-theory`. Its rustc, frontend, VIR,
VC, manifests, and release provenance remain helper traceability. Only its
canonical program certificate and recomputed zero-axiom, dual-checker result
support acceptance. Frozen compiler/build-input identity is recorded only in
`release-report.json`, never in policy evidence or checker input.

## Ownership Boundary

MPK owns generic verification capabilities:

- Go subset feature detection and fail-closed diagnostics;
- contract sidecar parsing, validation, and function resolution;
- VIR emission, VC generation, and stable artifact hashing;
- theory-certificate-backed proof strategies for supported payment obligations;
- source-free Rust checker and independent Go reference-checker invocation;
- package/evidence schemas and deterministic machine-readable output;
- golden corpora and regression tests for supported policy patterns.

ProofOps owns customer-facing capabilities:

- landing pages, forms, dashboards, and user authentication;
- Gemini prompts, tool orchestration, and analysis logs;
- customer code intake, redaction, privacy policy, and retention rules;
- report branding and business-language explanations;
- pricing, checkout, invoicing, customer support, and testimonials;
- XPRIZE business metrics and submission artifacts.

MPK must expose enough structured output for ProofOps to build the product
without scraping human logs or trusting generated prose.

## Required MPK Capabilities

### 1. Policy Readiness Scan

MPK exposes an engine-level readiness command that inspects a target Go
package, function, and contract sidecar without claiming proof acceptance.

Current CLI:

```sh
mpk policy scan ./internal/paymentpolicy \
  --language go \
  --semantic-profile mpk.go.fixed.v0 \
  --require-release-registry-id mpk.release.registry.v0 \
  --require-release-registry-sha256 <registry-sha256> \
  --frontend-bundle frontend.go.go2vir.v0 \
  --toolchain-bundle toolchain.go.go1.25.0.linux-amd64.v0 \
  --target linux/amd64 \
  --package example.com/customer/internal/paymentpolicy \
  --function example.com/customer/internal/paymentpolicy.ApprovedReserveCents \
  --contract policy_contract.json \
  --json-out mpk-policy-scan.json
```

Required output:

- function identity and source file hashes;
- Go toolchain and `go2vir` binary hash;
- supported/unsupported feature report;
- contract parse and function-resolution status;
- required preconditions;
- MPK readiness status: `ready`, `needs_refactor`, or `unsupported`;
- deterministic error codes and source locations for unsupported features;
- no proof-acceptance field.

This output powers ProofOps free scans and $99 diagnosis reports. It is helper
evidence only.

The stable scan schema is `mpk.policy.scan.v1`. It records the source language,
semantic profile and parameters, registered release identities, source/IR/map
hashes, selected function, helper artifacts, readiness, issues, and
unsupported codes. The closed shape and ordering rules are owned by
`develop/specs/POLICY_V1.md`.

For product-facing artifact paths, `policy scan` rejects path traversal.
Frontend and toolchain options are registered bundle IDs, never local paths.

### 2. Policy Verification Orchestrator

MPK exposes a single command that runs the existing pipeline in a stable,
product-ready order for one policy bundle.

Current CLI:

```sh
mpk policy verify ./internal/paymentpolicy \
  --language go \
  --semantic-profile mpk.go.fixed.v0 \
  --require-release-registry-id mpk.release.registry.v0 \
  --require-release-registry-sha256 <registry-sha256> \
  --frontend-bundle frontend.go.go2vir.v0 \
  --toolchain-bundle toolchain.go.go1.25.0.linux-amd64.v0 \
  --target linux/amd64 \
  --package example.com/customer/internal/paymentpolicy \
  --function example.com/customer/internal/paymentpolicy.ApprovedReserveCents \
  --contract policy_contract.json \
  --strategy-profile payment-policy-alpha \
  --checker-profile mvp-strict \
  --axiom-profile zero-axiom \
  --evidence-json mpk-evidence.json \
  --evidence-md mpk-evidence.md
```

The command orchestrates:

1. Go package loading and subset rejection;
2. contract sidecar validation;
3. VIR generation and hash computation;
4. VC generation and hash computation;
5. selected strategy profile execution;
6. checked-theory evidence export for the supported reserve path;
7. deterministic evidence JSON and Markdown output.

The orchestrator must not introduce a new acceptance path. A property may use
status `mpk_verified` only if checker-facing evidence passes the MPK
trust-boundary rules. Use `mvp-strict` when the selected strategy profile may
emit checked theory certificates; narrower checker profiles may still be useful
for helper-only scans or non-theory proof-node experiments. `--strict` turns
remaining `proof_pending` properties into a failing verify run.

For product-facing artifact paths, `policy verify` rejects path traversal
components in the target, `--contract`, `--evidence-json`, and `--evidence-md`
values. The generated evidence uses placeholder reproduction paths such as
`<go2vir>`, `<evidence.json>`, and `<evidence.md>` so local absolute paths do
not enter stable product reports.

### 3. Payment-Policy Strategy Profile

MPK defines a narrow strategy profile for the first commercial policies. This
is not an MPK checker profile or axiom-policy profile. The strategy profile
selects generated obligations, payment templates, and proof-search strategies;
the checker profile still uses the MPK release-policy values such as
`core-bootstrap`, `mvp-structural`, or `mvp-strict`. The current
`payment-policy-alpha` profile covers the initial linear integer and comparison
obligations that occur in common payment policy functions.

Initial obligations:

- non-negative result under non-negative inputs;
- result bounded by an input amount;
- refund bounded by paid minus already-refunded amount;
- discount caps and fee floors bounded by configured amounts;
- branch result equals one of the branch-selected inputs;
- runtime-safety checks for supported integer operations.

The profile prefers fixed-width integer cents and basis points. Float, decimal
packages, arbitrary precision arithmetic, maps, dynamic slices, database state,
network state, time, randomness, and concurrency remain out of scope unless
separately specified.

### 4. Checked Theory Certificates For Common Payment Obligations

The product is only credible if generated payment VCs can close to checked
evidence for a useful subset. The current payment-policy alpha path emits
checked `linarith` certificates for supported linear obligations and checked
`bool_tautology` certificates for reflexive selected-branch equality
obligations. The reserve, refund, discount, fee, and points positive corpus
passes strict verification with eight checked properties per example. Additional
strategy coverage should harden checked theory certificates for:

- signed integer linear inequalities;
- simple branch path-condition reasoning;
- safe subtraction under preconditions;
- multiplication by non-negative basis-point constants when bounds are known;
- simple min/max-shaped results.

Each strategy success must attach checked theory evidence or expand into
certificate-checkable proof nodes. Solver yes/no results and AI explanations are
never accepted directly; unsupported or not-yet-closed obligations remain
`proof_pending` or `unsupported`.

The implementation design for the first coverage expansion lives in
[`payment-policy-alpha-coverage-design.md`](payment-policy-alpha-coverage-design.md).
It covers the reserve checked-theory fixture, corpus-wide strict verification
for refund, discount, fee, and points, and the `--strict` success path for the
supported positive corpus.

### 5. Evidence Schema

The stable evidence schema for product integration is
`mpk.policy.evidence.v1`.

Required fields:

- target function identity;
- source artifact hashes;
- contract hash;
- VIR hash;
- VC hash;
- certificate hash, when present;
- export hash, when present;
- axiom report hash and category counts;
- strategy profile;
- checker profile;
- Rust checker verdict;
- reference checker verdict, when required;
- checked theory-certificate formats and hashes, when present;
- property list with per-property status;
- helper-artifact warnings;
- reproduction commands.

The JSON schema is the product API. Markdown output is useful for humans but is
not the source of truth. A property may be marked verified only when its evidence
is either a checked declaration in canonical `.mpcert` bytes or a checked theory
certificate accepted under the active checker profile. If a property is only
represented by source text, contract text, VIR, VC JSON, CI status, or Gemini
analysis, the schema must label it as helper analysis or proof-pending.

POE-04 pins the Rust-facing JSON shape as `mpk.policy.evidence.v1`:

- top-level workflow policy fields:
  - `strategy_profile`;
  - `checker_profile`;
  - `axiom_profile`;
- `trusted_evidence`, limited to:
  - checked certificate identities, `certificate_hash`, `export_hash`, and
    `axiom_report_hash`;
  - checked theory-certificate formats, hashes, and checked obligation ids;
  - recomputed axiom report category counts;
  - Rust fast-kernel verdicts;
  - independent reference-checker verdicts when required;
- `helper_artifacts`, limited to:
  - source hashes and source-file hashes;
  - contract hash and contract schema;
  - VIR hash;
  - VC hash;
  - helper warnings from source, contract, VIR, VC, AI analysis, or CI status;
- `properties`, where each property has one of `mpk_verified`,
  `proof_pending`, `helper_only`, or `unsupported`.

The schema validator rejects unknown top-level fields and unknown property
statuses. A property with `mpk_verified` is valid only when it references a
checked declaration id from `trusted_evidence.certificates` or a checked
obligation id from `trusted_evidence.theory_certificates`. Helper artifacts can
explain a property, but they cannot make it verified.

## ProofOps Repository Handoff

The ProofOps repository can consume MPK as an external engine pinned by git SHA
or binary hash. It should treat the following MPK outputs as the stable product
contract:

- `mpk policy scan` output with schema `mpk.policy.scan.v1`;
- `mpk policy verify` output with schema `mpk.policy.evidence.v1`;
- Markdown evidence reports generated from the evidence JSON for human review;
- command stdout only as an operator status line, not as product data.

ProofOps can display these fields directly in reports and dashboards:

- scan `readiness.status`: `ready`, `needs_refactor`, or `unsupported`;
- scan `supported_features`, `rejected_features`, and `preconditions`, all as
  helper analysis from `helper_evidence` entries;
- evidence `strategy_profile`, currently `payment-policy-alpha`;
- evidence `checker_profile`, for example `mvp-strict`;
- evidence `axiom_profile`, which is the axiom policy allowlist and is
  not the strategy profile or checker profile;
- evidence `trusted_evidence`, which is the only machine-readable source for
  checked certificate, checked theory-certificate, checker verdict, and axiom
  report claims;
- evidence `helper_artifacts`, which can explain source, contract, VIR, VC,
  AI, CI, or call-site context but cannot verify a property;
- evidence `properties[*].status`: `mpk_verified`, `proof_pending`,
  `helper_only`, or `unsupported`;
- evidence `reproduction_recipes`, whose product-facing paths are placeholders
  rather than local absolute paths.

Customer-facing ProofOps claims must map MPK fields this way:

| Claim label | MPK source | Customer meaning |
| --- | --- | --- |
| `mpk_verified` | `properties[*].status == "mpk_verified"` with checked declaration or checked theory-certificate evidence under `trusted_evidence` | MPK accepted this property under the active checker profile. |
| `mpk_helper` | scan output, `helper_artifacts`, call-site preconditions, VIR hash, VC hash, or Markdown text | Useful analysis or traceability, not proof evidence. |
| `proof_pending` | `properties[*].status == "proof_pending"` | MPK generated and classified the obligation, but no checked proof evidence closed it. |
| `unsupported` | scan `unsupported`, rejected features, unsupported helper warnings, or `properties[*].status == "unsupported"` | The current MPK subset or strategy cannot verify this path. |

ProofOps must not turn Go source, contract JSON, VIR, VC JSON, helper hashes,
Markdown prose, CI status, Gemini logs, or operator notes into proof evidence.
Those artifacts can support explanations, triage, and sales reporting only when
they are labeled as helper analysis or AI/manual context.

### MPK-Owned Optional Explanation Boundary

MPK also owns one narrow, opt-in helper path: `mpk explain`, built with the
optional `vertex-ai` feature, can send a minimal allowlisted projection of a
validated `mpk.policy.evidence.v1` report to Gemini on Vertex AI through local
ADC. It writes separate `mpk.ai.explanation.v1` JSON and Markdown outputs.
Those outputs are always untrusted helper analysis; they carry no authority to
create proof evidence, change a property status, or alter a checker result.
The command does not modify evidence, run automatically from policy commands,
accept static credentials, upload source or certificates, or own customer
consent and retention policy. The dry run is offline, and normal remote
processing must be disclosed and configured by the operator. ProofOps remains
the owner of customer workflows, presentation, consent, and retention
decisions around these reports.

### 6. Call-Site Precondition Helper

Payment contracts often require preconditions such as `requestedCents >= 0`.
MPK provides a helper lint that reports whether nearby call sites visibly
enforce required preconditions before calling the policy function.

This helper is not proof evidence. It reports:

- `checked_by_local_guard`;
- `declared_upstream_invariant`;
- `not_observed`;
- `unsupported_control_flow`.

The output helps ProofOps explain integration risks without expanding the
checker trust boundary.

### 7. Payment Policy Corpus

MPK includes a regression corpus for product-relevant examples:

- wallet reserve;
- partial refund;
- coupon discount;
- platform fee floor;
- loyalty points redemption;
- negative fixtures for floats, maps, pointers, and missing postconditions.

Each positive corpus entry includes Go source, contract sidecar, expected VIR
and VC artifacts, scan coverage, and verification/evidence output when the
strategy profile supports the case.

## Non-Goals

The MPK-side work must not implement:

- Gemini prompts or agent workflows;
- web forms, dashboards, customer accounts, or billing;
- sales copy, pricing logic, or marketing analytics;
- arbitrary Go service verification;
- full IAM, Rego, Cedar, or authorization-graph replacement;
- database, network, payment-gateway, or handler effect verification;
- proof acceptance based on AI logs, generated prose, source text, or CI status.

## Milestones

### M0: Product Boundary Spec

Deliverables:

- this design reviewed against `develop/specs/TRUST_BOUNDARY_V0.md`;
- `mpk.policy.scan.v1` and `mpk.policy.evidence.v1` draft schemas;
- list of supported payment patterns and explicit exclusions.

Exit criteria:

- ProofOps can build UI and reports from documented JSON outputs;
- no product requirement expands the MPK trusted boundary.

### M1: Readiness Scan

Deliverables:

- `mpk policy scan`;
- JSON output fixtures;
- unsupported-feature diagnostics for common non-MPK-ready payment code.

Exit criteria:

- free-scan and $99 report flows can run without manual log parsing.

### M2: Verify Orchestrator

Deliverables:

- `mpk policy verify`;
- stable VIR/VC/certificate/evidence artifact layout;
- Markdown and JSON evidence output.

Exit criteria:

- a supported single-function payment policy can be checked in CI through one
  command.

### M3: Payment Strategy Profile

Deliverables:

- checked theory strategy coverage for core payment inequalities;
- positive and negative examples for reserve, refund, discount, fee, and points;
- clear `unsupported_property` diagnostics.

Exit criteria:

- ProofOps can sell one-function CI packs without manually proving each basic
  inequality.

### M4: Integration Helpers

Deliverables:

- GitHub Actions examples;
- call-site precondition helper;
- evidence report examples suitable for customer review.

Exit criteria:

- ProofOps can deliver a $499 CI pack with predictable artifacts and a clear
  support boundary.

## Design Constraints

- All new checker-facing behavior must reference and obey
  `develop/specs/TRUST_BOUNDARY_V0.md`.
- Any schema used by ProofOps must be versioned and deterministic.
- Any claim in a product report must be traceable to either trusted proof
  evidence or explicitly labeled helper analysis.
- Unsupported features must fail closed with deterministic error codes.
- Product-specific language may live in ProofOps; MPK output should remain
  neutral, structured, and auditable.
