# ProofOps Engine Support Design

Status: draft product-enablement design

This document defines the MPK-side work needed to support the ProofOps product
and service repository in `../proof-ops`. MPK remains the verification engine:
it owns source-free proof acceptance, deterministic verification artifacts,
Go-policy lowering, VC generation, theory-backed proof construction, and
machine-readable evidence. ProofOps owns the customer-facing product,
Gemini-based analysis, web forms, pricing, reporting presentation, billing, and
XPRIZE submission operations.

## Product Context

ProofOps sells payment-policy verification workflows for small Go services.
The initial use cases are pure deterministic functions that decide reserve,
refund, discount, fee, points-redemption, and quota amounts. The product promise
must stay narrow:

- identify whether a function is MPK-ready;
- generate or validate safety contracts for payment policies;
- turn a supported policy function into reproducible verification artifacts;
- produce reviewable evidence for CI, customers, and XPRIZE judges;
- never treat AI output, source text, GIR, VC JSON, CI status, or report prose as
  proof evidence.

The MPK trust boundary remains unchanged. A payment policy is accepted only when
the relevant theorem is represented by canonical `.mpcert` bytes or checked
theory certificates and passes the configured checker policy.

## Ownership Boundary

MPK owns generic verification capabilities:

- Go subset feature detection and fail-closed diagnostics;
- contract sidecar parsing, validation, and function resolution;
- GIR emission, VC generation, and stable artifact hashing;
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

Add an engine-level readiness command or API that inspects a target Go package,
function, and optional contract sidecar without claiming proof acceptance.

Proposed CLI:

```sh
mpk policy scan ./internal/paymentpolicy \
  --function paymentpolicy.ApprovedReserveCents \
  --contract policy_contract.json \
  --json-out mpk-policy-scan.json
```

Required output:

- function identity and source file hashes;
- Go toolchain and `go2gir` binary hash;
- supported/unsupported feature report;
- contract parse and function-resolution status;
- required preconditions;
- MPK readiness status: `ready`, `needs_refactor`, or `unsupported`;
- deterministic error codes and source locations for unsupported features;
- no proof-acceptance field.

This output powers ProofOps free scans and $99 diagnosis reports. It is helper
evidence only.

### 2. Policy Verification Orchestrator

Add a single command that runs the existing pipeline in a stable, product-ready
order for one policy bundle.

Proposed CLI:

```sh
mpk policy verify ./internal/paymentpolicy \
  --function paymentpolicy.ApprovedReserveCents \
  --contract policy_contract.json \
  --strategy-profile payment-policy-alpha \
  --checker-profile mvp-strict \
  --evidence-json mpk-evidence.json \
  --evidence-md mpk-evidence.md
```

The command should orchestrate:

1. Go package loading and subset rejection;
2. contract sidecar validation;
3. GIR generation and hash computation;
4. VC generation and hash computation;
5. selected strategy profile execution;
6. certificate or checked-theory evidence export;
7. Rust fast-kernel verification;
8. optional Go reference-checker verification;
9. axiom report recomputation;
10. deterministic evidence output.

The orchestrator must not introduce a new acceptance path. It may report
`verified=true` only if checker-facing evidence passes the MPK trust-boundary
rules. Use `mvp-strict` when the selected strategy profile may emit checked
theory certificates; narrower checker profiles may still be useful for
helper-only scans or non-theory proof-node experiments.

### 3. Payment-Policy Strategy Profile

Define a narrow strategy profile for the first commercial policies. This is not
an MPK checker profile or axiom-policy profile. The strategy profile selects
generated obligations, payment templates, and proof-search strategies; the
checker profile still uses the MPK release-policy values such as
`core-bootstrap`, `mvp-structural`, or `mvp-strict`. The strategy profile should
cover linear integer and comparison obligations that occur in common payment
policy functions.

Initial obligations:

- non-negative result under non-negative inputs;
- result bounded by an input amount;
- refund bounded by paid minus already-refunded amount;
- discount or fee bounded by configured caps;
- branch result equals one of the branch-selected inputs;
- runtime-safety checks for supported integer operations.

The profile should initially prefer fixed-width integer cents and basis points.
Float, decimal packages, arbitrary precision arithmetic, maps, dynamic slices,
database state, network state, time, randomness, and concurrency remain out of
scope unless separately specified.

### 4. Checked Theory Certificates For Common Payment Obligations

The product is only credible if generated payment VCs can close to checked
evidence for a useful subset. MPK should add or harden strategy paths that emit
checked theory certificates for:

- signed integer linear inequalities;
- simple branch path-condition reasoning;
- safe subtraction under preconditions;
- multiplication by non-negative basis-point constants when bounds are known;
- simple min/max-shaped results.

Each strategy success must attach checked theory evidence or expand into
certificate-checkable proof nodes. Solver yes/no results and AI explanations are
never accepted directly.

### 5. Evidence Schema

Define a stable evidence schema for product integration.

Suggested schema id: `mpk.policy.evidence.v0`

Required fields:

- target function identity;
- source artifact hashes;
- contract hash;
- GIR hash;
- VC hash;
- certificate hash, when present;
- export hash, when present;
- axiom report hash and category counts;
- strategy profile;
- checker profile;
- Rust checker verdict;
- reference checker verdict, when required;
- checked theory-certificate hashes, when present;
- verified property list;
- helper-artifact warnings;
- unsupported or unverified property list;
- reproduction commands.

The JSON schema is the product API. Markdown output is useful for humans but is
not the source of truth. A property may be marked verified only when its evidence
is either a checked declaration in canonical `.mpcert` bytes or a checked theory
certificate accepted under the active checker profile. If a property is only
represented by source text, contract text, GIR, VC JSON, CI status, or Gemini
analysis, the schema must label it as helper analysis or proof-pending.

### 6. Call-Site Precondition Helper

Payment contracts often require preconditions such as `requestedCents >= 0`.
MPK should provide a helper lint that reports whether nearby call sites visibly
enforce required preconditions before calling the policy function.

This helper is not proof evidence. It should clearly report:

- `checked_by_local_guard`;
- `declared_upstream_invariant`;
- `not_observed`;
- `unsupported_control_flow`.

The output helps ProofOps explain integration risks without expanding the
checker trust boundary.

### 7. Payment Policy Corpus

Add a regression corpus for product-relevant examples:

- wallet reserve;
- partial refund;
- coupon discount;
- platform fee cap;
- loyalty points redemption;
- subscription quota;
- negative and overflow rejection fixtures.

Each corpus entry should include Go source, contract sidecar, expected GIR/VC
hashes, expected scan output, and expected verification/evidence output when
the strategy profile supports the case.

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
- `mpk.policy.scan.v0` and `mpk.policy.evidence.v0` draft schemas;
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
- stable GIR/VC/certificate/evidence artifact layout;
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
