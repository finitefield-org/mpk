# Payment Policy Alpha Coverage Design

Status: historical predecessor-v1 implementation record. Its flags and schema
are not active interfaces; see `proof-ops-engine-design.md` for the revision-2
successor contract.

This document defines the first practical coverage expansion for the
`payment-policy-alpha` strategy profile. The goal is to make the MPK-side
ProofOps payment-policy path more useful without widening the proof trust
boundary.

## Objective

This document records the first coverage step for common payment policies. The
implemented result is:

- the reserve example verifies all eight emitted properties with checked
  theory-certificate evidence;
- refund, discount, fee, and points also pass strict verification with all
  eight emitted properties `mpk_verified`;
- all five positive payment-policy examples pass `mpk policy verify --strict`
  with `verified=8 proof_pending=0 unsupported=0`;
- keep every `mpk_verified` claim backed by checked theory-certificate evidence
  under `trusted_evidence`.

This work is MPK-side engine work. ProofOps can consume the resulting
`mpk.policy.evidence.v1` output, but ProofOps product code is out of scope.

## Implementation State

The current `mpk policy verify` path is implemented in
`crates/mpk-cli/src/policy_verify.rs`:

- runs `policy scan`;
- imports VIR and generates VCs;
- classifies each obligation with
  `classify_payment_policy_obligations`;
- extracts payload-bound policy theory goals from each supported obligation;
- closes supported linear obligations with checked `mpk.linarith.v0`
  certificates;
- closes reflexive selected-branch equality obligations with checked
  `mpk.bool-normalize.v0` certificates;
- marks a property `mpk_verified` only when the accepted checked theory
  certificate names that obligation;
- fails `--strict` when any property remains `proof_pending` or when any
  unsupported property is emitted.

The live CLI behavior is covered by `crates/mpk-cli/tests/policy_verify.rs`.
The checked-in `examples/payment_policies/reserve/evidence_alpha.json` fixture
is refreshed with `--update-fixtures` and records
`verified=8 proof_pending=0 unsupported=0` for reserve.

The positive payment-policy corpus currently has five examples:

| Example | Function shape | Obligation mix |
| --- | --- | --- |
| `reserve` | `min(requestedCents, balanceCents)` | 2 non-negative, 4 bound, 2 branch equality |
| `refund` | `min(requestedCents, paidCents)` | 2 non-negative, 4 refund bound, 2 branch equality |
| `discount` | `min(requestedDiscountCents, subtotalCents)` | 2 non-negative, 4 discount-cap bound, 2 branch equality |
| `fee` | `max(calculatedFeeCents, minimumFeeCents)` | 2 non-negative, 4 fee-floor bound, 2 branch equality |
| `points` | `min(requestedPoints, pointsBalance)` | 2 non-negative, 4 bound, 2 branch equality |

The classifier in `crates/mpk-vc/src/policy_obligation.rs` is helper analysis
only. Its output must never directly create trusted evidence.

The current Rust enum name `FeeOrDiscountBoundedByCap` is broader than the
current examples. In this design, it covers both discount-cap obligations and
fee-floor obligations emitted by the min/max-shaped corpus. Do not implement
the closure as a cap-only rule.

## Trust Boundary Requirements

The expansion must preserve these invariants:

- A classifier pattern is only a closure candidate, not proof evidence.
- `helper_artifacts`, VIR JSON, VC JSON, Markdown, command success, and AI output
  never make a property verified.
- `mpk_verified` may be emitted only by a closure result that was produced from
  the actual `VcObligation` and checked by MPK code.
- Each verified property must reference a
  `PolicyPropertyEvidenceRef::CheckedTheoryCertificate` whose
  `obligation_id` appears in the matching
  `trusted_evidence.theory_certificates[*].checked_obligations`.
- Unsupported or not-yet-closed obligations remain `proof_pending` or
  `unsupported`; do not hide them to make `--strict` pass.

Do not expand the current static witness pattern as-is. Before increasing
coverage, policy closure code must bind a theory certificate to the actual VC
obligation by recomputing the normalized goal from that `VcObligation` and
checking the certificate against that normalized goal.

## Design

### 1. Add A Payload-Bound Policy Closure Layer

Add a policy-specific closure layer that sits between VC classification and
evidence rendering.

Suggested files:

- `crates/mpk-vc/src/policy_theory_goal.rs`;
- `crates/mpk-vc/src/lib.rs`;
- `crates/mpk-theory/src/linarith_cert.rs`;
- `crates/mpk-theory/src/bool_cert.rs`;
- `crates/mpk-cli/src/policy_verify.rs`;
- `crates/mpk-cli/Cargo.toml`.

Crate ownership:

- `mpk-vc` owns extraction of neutral theory goals from `VcObligation` values.
  It must not depend on `mpk-theory`, `mpk-cert`, `mpk-kernel`, or `mpk-api`.
- `mpk-theory` owns public encode/check helpers for concrete theory
  certificates. It already owns the checker structs and constants; this step
  must add matching public encoders instead of duplicating byte layouts in
  `mpk-cli`.
- `mpk-cli` owns policy orchestration, checker-profile gating, certificate
  hashing, evidence ids, and evidence rendering. Add direct `mpk-cert` and
  `mpk-theory` dependencies to `crates/mpk-cli/Cargo.toml` for this purpose.
- `mpk-api` keeps the strategy-profile metadata and the existing generic
  strategy tests. Do not add a dependency from `mpk-api` to `mpk-vc`.

Core data model:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyTheoryGoal {
    pub obligation_id: String,
    pub function_id: String,
    pub pattern: PaymentPolicyObligationPattern,
    pub kind: PolicyTheoryGoalKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyTheoryGoalKind {
    Linear(PolicyLinearGoal),
    BoolTautology(PolicyBoolGoal),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyLinearGoal {
    // Stable variable index to display name. The index is used by
    // mpk_theory::LinearTerm; the display name is for tests and diagnostics.
    pub variables: Vec<String>,
    pub premises: Vec<PolicyLinearInequality>,
    pub goal: PolicyLinearInequality,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyLinearInequality {
    // sum(terms) + constant <= 0
    pub terms: Vec<PolicyLinearTerm>,
    pub constant: i128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyLinearTerm {
    pub variable: u32,
    pub coefficient: i128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyBoolGoal {
    pub reason: PolicyBoolTautologyReason,
    pub tautology: PolicyBoolTautology,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyBoolTautologyReason {
    ReflexiveSelectedBranchDisjunct,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyBoolTautology {
    TrueOrOpaque,
    OpaqueOrTrue,
}
```

Public extraction API:

```rust
pub fn policy_theory_goal_from_obligation(
    obligation: &VcObligation,
    classification: &PaymentPolicyObligationClassification,
) -> Result<Option<PolicyTheoryGoal>, PolicyTheoryGoalError>
```

Return values:

- `Ok(Some(goal))`: the obligation can be attempted by a checked theory
  closure.
- `Ok(None)`: the obligation is supported helper analysis, but this coverage
  step has no checked closure for its concrete shape.
- `Err(error)`: generated VC data is internally inconsistent, for example a
  classifier says a postcondition is linear but the referenced obligation cannot
  be parsed as a comparison. `policy verify` must fail with an internal error
  for this case rather than silently changing trust semantics.

Variable indexing rules for `PolicyLinearGoal`:

- Use a `BTreeMap<String, u32>` to assign stable indexes.
- Variables use keys `var:<name>`.
- Result terms use keys `result:<index>`.
- Define `POLICY_THEORY_MAX_LINEAR_VARIABLES: usize = 64` in `mpk-vc`, matching
  `mpk_theory::MAX_LINARITH_VARIABLES` without adding a crate dependency.
- Reject duplicate indexes and more than `POLICY_THEORY_MAX_LINEAR_VARIABLES`
  variables.
- Sort `terms` by `variable` and remove zero coefficients.

The closure API returns a trusted result only after checking the concrete
certificate:

```rust
pub struct PolicyClosedObligation {
    pub obligation_id: String,
    pub certificate_id: String,
    pub theory: &'static str,
    pub format: String,
    pub theory_certificate_hash: String,
    pub evidence_note: String,
}
```

Only `PolicyClosedObligation` values can be used to create
`mpk_verified` properties.

Certificate encoding and checking contract:

```rust
pub fn encode_linarith_certificate(certificate: &LinarithCertificate) -> Vec<u8>;
pub fn encode_bool_certificate(certificate: &BoolCertificate) -> Vec<u8>;
```

Add these public helpers in `mpk-theory`. The bytes must match the payload
format currently decoded by `mpk-kernel` for `mpk.linarith.v0` and
`mpk.bool-normalize.v0`. Tests must round-trip by encoding a certificate,
wrapping it in `mpk_cert::encode::TheoryCertificate`, and verifying that the
kernel accepts the wrapped payload.

`mpk-cli` must compute `theory_certificate_hash` with the same domain hash used
by existing strategy evidence:

```rust
let theory_certificate = TheoryCertificate { format, payload };
let canonical = encode_theory_certificate(&theory_certificate);
let hash = hash_hex(&hash_with_domain(HashDomain::TheoryCertificate, &canonical));
```

Do not use `theory_strategy_certificate_evidence(TheoryStrategyKind::Linarith)`
for policy closures after this change. That helper returns the old static
strategy fixture payload and is not bound to a policy VC obligation.

### 2. Linearize Supported BV64 Comparisons

Add deterministic linearization for signed BV64 comparisons whose operands are
only variables, result terms, or signed BV64 literals.

Supported predicates:

| Predicate | Linear inequality form |
| --- | --- |
| `sge(lhs, rhs)` | `rhs - lhs <= 0` |
| `sle(lhs, rhs)` | `lhs - rhs <= 0` |
| `sgt(lhs, rhs)` | `rhs - lhs + 1 <= 0` |
| `slt(lhs, rhs)` | `lhs - rhs + 1 <= 0` |

Operand rules:

- accept `MpkExprTerm::Var`, `MpkExprTerm::Result`, and signed BV64
  `MpkExprTerm::BitVecLiteral` only;
- reject unsigned literals, non-BV64 literals, function applications, boolean
  literals, and unsupported arithmetic with `Ok(None)` from the theory-goal
  extractor;
- encode literals by changing the inequality constant;
- encode variables and result terms through the stable variable index map;
- normalize by sorting terms and combining coefficients.

The first implementation supports the corpus shapes only:

- exact premise proves goal;
- a strict branch premise proves a weaker non-strict goal with positive slack;
- identity goal such as `x <= x` or `x >= x` closes with an empty combination.

Do not add a general-purpose solver yet. A small deterministic certificate
builder is enough:

1. Normalize all premises and the goal into canonical variable order.
2. If the goal normalizes to zero, build a linarith certificate with an empty
   combination.
3. If one premise has the same variable coefficients as the goal and a constant
   that is at least as strong as the goal, build a one-row certificate with
   multiplier `1`.
4. Otherwise return no closure and leave the property `proof_pending`.

Concrete examples from `examples/payment_policies/reserve/vc.json`:

| Obligation | Reason | Certificate shape |
| --- | --- | --- |
| `then.post0` proves `balanceCents >= 0` | exact precondition premise | one-row linarith |
| `then.post1` proves `balanceCents <= balanceCents` | identity goal | empty-combination linarith |
| `then.post2` proves `balanceCents <= requestedCents` | branch premise `requestedCents > balanceCents` is stronger | one-row linarith with positive slack |
| `else.post0` proves `requestedCents >= 0` | exact precondition premise | one-row linarith |
| `else.post1` proves `requestedCents <= balanceCents` | else branch premise is stronger | one-row linarith with positive slack |
| `else.post2` proves `requestedCents <= requestedCents` | identity goal | empty-combination linarith |

Expected phase-A result for the current positive corpus:

| Example | Expected verified after linear closure | Remaining pending |
| --- | ---: | ---: |
| `reserve` | 6 | 2 branch equality |
| `refund` | 6 | 2 branch equality |
| `discount` | 6 | 2 branch equality |
| `fee` | 6 | 2 branch equality |
| `points` | 6 | 2 branch equality |

This phase satisfies the product requirement that refund, discount, fee, and
points each have at least one `mpk_verified` property.

### 3. Close Reflexive Branch Equality Obligations

The current min/max examples emit selected-branch obligations shaped as:

```text
Std.Bool.or(Std.Eq(branch_result, input_a), Std.Eq(branch_result, input_b))
```

For each concrete branch VC, one disjunct is reflexive after branch lowering,
for example `Std.Eq(balanceCents, balanceCents)`. Add a bool-tautology closure
for exactly this shape:

- accept only `Std.Bool.or` with exactly two disjuncts;
- close only when at least one disjunct is syntactically reflexive
  `Std.Eq(term, term)`;
- normalize the reflexive disjunct to `true`;
- treat the other disjunct as an opaque bool variable;
- build and check a `mpk.bool-normalize.v0` certificate for `true OR p` or
  `p OR true`;
- leave all other branch equality shapes `proof_pending`.

Concrete bool certificates:

- `TrueOrOpaque` uses `BoolExpr::Or(Box::new(BoolExpr::Const(true)),
  Box::new(BoolExpr::Var(0)))` with `variable_count = 1`.
- `OpaqueOrTrue` uses `BoolExpr::Or(Box::new(BoolExpr::Var(0)),
  Box::new(BoolExpr::Const(true)))` with `variable_count = 1`.
- The certificate rows must cover both assignments and claim `true` for both:
  `[false] -> true` and `[true] -> true`.
- Check the certificate with `check_bool_certificate` before creating
  `PolicyClosedObligation`.

Expected phase-B result:

| Example | Expected verified after bool closure | Remaining pending |
| --- | ---: | ---: |
| `reserve` | 8 | 0 |
| `refund` | 8 | 0 |
| `discount` | 8 | 0 |
| `fee` | 8 | 0 |
| `points` | 8 | 0 |

The initial `--strict` success acceptance uses the smallest stable path:
`examples/payment_policies/reserve` with the existing contract. Do not add a
separate strict-success fixture for this coverage step; the product-facing proof
path must demonstrate strict success on an existing corpus example.

### 4. Checker Profile Behavior

Checked theory closure is available only under `checker_profile = "mvp-strict"`.
This preserves the current behavior of `try_close_first_linarith_obligation`,
where non-theory profiles do not produce trusted theory evidence.

Rules:

- `mvp-strict`: attempt linarith and bool closures and emit trusted evidence for
  successfully checked certificates.
- `mvp-structural` and `core-bootstrap`: do not attempt theory closure; keep
  otherwise-supported properties `proof_pending`.
- unknown checker profiles: keep the existing deterministic CLI rejection.
- `--strict`: fail after writing evidence if any property remains
  `proof_pending` or `unsupported`; do not special-case checker profiles.

This means `mpk policy verify --checker-profile mvp-structural --strict` must
still fail for the payment corpus, because theory-backed properties remain
pending.

Implementation must parse `checker_profile` through the existing proof-profile
parser before comparing it with `ProofProfile::MvpStrict`. A string inequality
such as `checker_profile != "mvp-strict"` is not sufficient, because it would
risk treating an unknown checker profile as a known non-theory profile.

### 5. Replace Single-Obligation Closure Planning

Replace `try_close_first_linarith_obligation` with a deterministic multi-close
planner.

Suggested shape:

```rust
fn try_close_policy_obligations(
    request: &PolicyVerifyRequest,
    vc_obligations: &[VcObligation],
    classifications: &[PaymentPolicyObligationClassification],
) -> Result<BTreeMap<String, PolicyClosedObligation>, PolicyVerifyRunError>
```

Planner rules:

- parse `PolicyStrategyMetadata` once before iterating;
- build a `BTreeMap<&str, &VcObligation>` by obligation id and fail if
  duplicate ids are generated;
- fail if a classification references an obligation id that is missing from the
  VC module for the selected function;
- process obligations in stable `obligation_id` order;
- skip unsupported classifications;
- require `PolicyStrategyMetadata::validate_obligation` before closure;
- select closure by classified pattern and available theory goal;
- never fail the whole verify run because a supported closure attempt did not
  apply; leave that property `proof_pending`;
- fail the run only for internal errors, malformed generated artifacts, unknown
  profiles, or checked-certificate validation errors.

Change `property_from_classification` to accept the closure map instead of the
current single `Option<String>` obligation id. Each verified property must
reference its own certificate id.

Certificate ids must be deterministic:

```text
theory:policy-linarith-0001
theory:policy-linarith-0002
theory:policy-bool-tautology-0001
```

Use stable numbering by sorted obligation id, not discovery order from hash maps.
It is acceptable for two certificates to have identical certificate hashes if
their canonical payloads are identical; their evidence ids and checked
obligation ids must still be distinct.

### 6. Evidence Handling

No `mpk.policy.evidence.v1` shape change is required for this step.

For each closure:

- append a `PolicyTheoryCertificateEvidence` entry under `trusted_evidence`;
- set `theory` to `linarith` or `bool_tautology`;
- set `format` to the checked certificate format;
- set `checker_profile` to the active checker profile;
- set `checked_obligations` to exactly the obligation ids closed by that
  certificate;
- mark the matching property `mpk_verified`;
- add a short note naming the checked theory and the reason, for example
  `Closed by checked linarith evidence for a branch-premise linear bound.`

For every unclosed supported property:

- keep `proof_pending`;
- keep only helper evidence refs;
- include a note with the classifier pattern.

### 7. CLI Output And Failure Handling

The human-facing CLI contract changes only when verified/pending counts change.

Expected stdout by phase for the reserve example:

| Phase | Non-strict stdout status | Counts |
| --- | --- | --- |
| current and PAYALPHA-COV-01 | `status=proof_pending` | `verified=1 proof_pending=7 unsupported=0` |
| after PAYALPHA-COV-02 | `status=proof_pending` | `verified=6 proof_pending=2 unsupported=0` |
| after PAYALPHA-COV-03 | `status=verified` | `verified=8 proof_pending=0 unsupported=0` |

Failure rules:

- unknown strategy profile: preserve the existing CLI validation error;
- unknown checker profile: preserve the existing CLI validation error;
- scan not ready: preserve `policy verify failed: scan status=<status>`;
- unsupported properties: preserve `policy verify failed: unsupported properties=<n>`;
- strict pending failure: preserve
  `policy verify failed: proof-pending properties=<n>`;
- policy theory-goal extraction inconsistency: fail with
  `policy verify proof closure failed: <detail>`;
- checked-certificate construction or checker failure after a closure is
  expected to apply: fail with `policy verify proof closure failed: <detail>`;
- closure not applicable for a supported-but-unhandled concrete shape: do not
  fail; emit `proof_pending`.

## Implementation Milestones

### PAYALPHA-COV-01: Baseline And Closure Plumbing

Tasks:

- add `crates/mpk-vc/src/policy_theory_goal.rs` with the neutral data model,
  extraction API, and focused unit tests;
- export the new module from `crates/mpk-vc/src/lib.rs`;
- add `PolicyClosedObligation` and a closure map in
  `crates/mpk-cli/src/policy_verify.rs`;
- replace the single `verified_obligation: Option<String>` plumbing with
  `BTreeMap<String, PolicyClosedObligation>`;
- keep `try_close_first_linarith_obligation` as the only producer temporarily,
  but make it return a one-entry closure map;
- preserve the current reserve CLI output
  `verified=1 proof_pending=7 unsupported=0`;
- keep behavior unchanged at the end of this milestone.

Required tests:

- `policy_verify_reserve_writes_evidence_and_markdown` still expects
  `verified=1 proof_pending=7 unsupported=0`;
- `policy_verify_strict_fails_after_writing_proof_pending_evidence` still fails
  with `proof-pending properties=7`;
- a new `mpk-vc` unit test proves `policy_theory_goal_from_obligation` returns
  `Ok(None)` for selected-branch bool obligations until PAYALPHA-COV-03.

Validation:

```sh
cargo test -p mpk-cli --test policy_verify
cargo test -p mpk-vc --test payment_policy_examples
```

### PAYALPHA-COV-02: Payload-Bound Linarith Closure

Tasks:

- add `mpk-cert` and `mpk-theory` dependencies to `crates/mpk-cli/Cargo.toml`;
- add `encode_linarith_certificate` to `mpk-theory`;
- add round-trip tests for `encode_linarith_certificate` using the existing
  kernel theory-certificate decoder path;
- implement BV64 comparison linearization;
- implement the small exact-premise and identity certificate builder;
- check the generated linarith certificate before returning
  `PolicyClosedObligation`;
- close all non-branch positive-corpus obligations that match the supported
  linear rules;
- update reserve CLI tests from `verified=1 proof_pending=7` to
  `verified=6 proof_pending=2`;
- add tests proving each positive corpus example has at least one
  `mpk_verified` property.

Implementation notes:

- remove the `ProofOps.PolicyVerify.theoryWitness` registration from the
  policy-closure path once payload-bound linarith closure is active;
- do not remove the generic `TheoryStrategyKind::Linarith` tests, because they
  still cover API strategy dispatch;
- parse `checker_profile`; for known profiles other than `ProofProfile::MvpStrict`,
  return an empty closure map rather than trying to check linarith evidence.
  Unknown checker profiles must still reject before evidence is written.

Required tests:

- a `mpk-vc` unit test for each predicate conversion: `sge`, `sle`, `sgt`,
  `slt`;
- a `mpk-vc` unit test proving arithmetic applications such as
  `Std.BitVec.BV64.add` return `Ok(None)`;
- a `mpk-cli` integration test table covering `reserve`, `refund`, `discount`,
  `fee`, and `points`, with each example asserting
  `verified_count >= 1`;
- a `mpk-cli` test proving non-`mvp-strict` checker profiles do not produce
  `trusted_evidence.theory_certificates`.
- a `mpk-cli` test proving an unknown checker profile still rejects even after
  the non-`mvp-strict` empty-closure path exists.

Validation:

```sh
cargo test -p mpk-theory linarith
cargo test -p mpk-vc --test payment_policy_examples
cargo test -p mpk-cli --test policy_verify
```

### PAYALPHA-COV-03: Reflexive Branch Bool Closure

Tasks:

- add `encode_bool_certificate` to `mpk-theory`;
- add round-trip tests for `encode_bool_certificate` using the existing kernel
  theory-certificate decoder path;
- implement branch tautology detection for `Std.Bool.or` with one reflexive
  equality disjunct;
- build and check a `mpk.bool-normalize.v0` certificate for the normalized bool
  formula;
- close the two selected-branch obligations in the reserve example;
- change the strict reserve test to expect success;
- add a non-strict test proving no `proof_pending` remains for reserve.

Required tests:

- a `mpk-vc` unit test for `TrueOrOpaque` and `OpaqueOrTrue`;
- a negative `mpk-vc` unit test where neither branch disjunct is reflexive and
  the result is `Ok(None)`;
- rename or replace
  `policy_verify_strict_fails_after_writing_proof_pending_evidence` so the
  reserve `--strict` path expects success, stdout status `verified`, and
  `verified=8 proof_pending=0 unsupported=0`;
- keep a separate strict-failure test using an unsupported-property temporary
  fixture so the fail-after-writing behavior remains covered.

Validation:

```sh
cargo test -p mpk-theory bool
cargo test -p mpk-cli --test policy_verify policy_verify_reserve_writes_evidence_and_markdown
cargo test -p mpk-cli --test policy_verify policy_verify_strict
```

### PAYALPHA-COV-04: Corpus-Wide Evidence Coverage

Tasks:

- add CLI tests for refund, discount, fee, and points using temporary evidence
  paths;
- assert each example has at least one `mpk_verified` property after
  PAYALPHA-COV-02;
- after PAYALPHA-COV-03, assert all five positive examples can pass `--strict`
  with `verified=8 proof_pending=0 unsupported=0`;
- keep tracked `evidence_alpha.json` limited to the existing reserve fixture in
  this coverage step. Do not add tracked evidence fixtures for refund, discount,
  fee, or points unless a later product requirement asks for golden reports for
  every example.

Required tests:

- a table-driven `policy_verify_positive_payment_corpus_has_expected_counts`
  test with exact expected counts per example;
- a determinism assertion for at least one non-reserve example, preferably
  `refund`, to ensure multi-certificate numbering is stable outside reserve;
- a schema parse assertion using `PolicyEvidenceReport::from_json` for every
  generated evidence JSON.

Validation:

```sh
cargo test -p mpk-cli --test policy_verify
cargo test -p mpk-vc --test payment_policy_examples
```

### PAYALPHA-COV-05: Docs And Local Gate

Tasks:

- update `examples/payment_policies/README.md` with the new expected verified
  coverage;
- update `docs/proof-ops-policy-ci.md` if the local example status changes from
  `proof_pending` to `verified`;
- update `docs/alpha-demo.md` so customer-facing demo language does not claim
  old pending counts;
- add or update a fast check script only if the existing test suite does not
  already cover the product path.

Required doc updates after PAYALPHA-COV-03:

- `docs/alpha-demo.md` must say reserve `policy verify --strict` is the strict
  success demo and must no longer mention one verified reserve
  obligation as the expected final result.
- `docs/proof-ops-policy-ci.md` must distinguish two gates: non-strict helper
  drift generation and strict product verification for examples with zero
  pending obligations.
- `examples/payment_policies/reserve/README.md` must describe all reserve
  properties as checked when PAYALPHA-COV-03 lands.
- `examples/payment_policies/reserve/evidence_alpha.json` must be refreshed
  with `--update-fixtures` only after the CLI output is deterministic.

Validation:

```sh
./scripts/check-fast.sh
```

## Test Matrix

Required tests:

- unknown strategy and checker profiles still reject deterministically;
- unsupported scan and unsupported property paths still write untrusted evidence
  and fail;
- `mpk_verified` properties still reject if they lack checked theory evidence;
- unclosed supported properties still have no checked evidence refs;
- repeated evidence output remains byte-identical;
- tracked fixture mutation still requires `--update-fixtures`;
- tampering with a VC conclusion after classification prevents closure;
- tampering with a certificate payload prevents closure or fails with an
  internal checked-certificate error.

## Risks

The main risk is accidentally treating the classifier as proof. Mitigation:
make the only constructor for `PolicyClosedObligation` private to the module
that recomputes and checks the certificate from the actual `VcObligation`.

The second risk is confusing mathematical integers with signed BV64 semantics.
The first implementation accepts only simple variables, result terms, and
signed BV64 literals. Arithmetic expressions, overflow-sensitive operations,
and runtime-safety obligations remain pending until a dedicated bit-vector or
range proof path is designed.

The third risk is schema churn. This design intentionally keeps
`mpk.policy.evidence.v1` unchanged and strengthens the producing code instead.
If external consumers later need independently auditable normalized-goal hashes,
handle that as a separate evidence-schema versioning task.

## Done Criteria

This coverage step is complete when:

- `reserve` passes `mpk policy verify --strict` with
  `verified=8 proof_pending=0 unsupported=0`;
- refund, discount, fee, and points each pass `mpk policy verify --strict` with
  `verified=8 proof_pending=0 unsupported=0`;
- no `mpk_verified` property is created without checked theory-certificate
  evidence under `trusted_evidence`;
- the full product-path verification commands are deterministic and covered by
  tests;
- related docs no longer describe the old one-verified-seven-pending state as
  the final expected behavior.
