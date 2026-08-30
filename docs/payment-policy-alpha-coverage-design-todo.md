# Payment Policy Alpha Coverage Design Todo

Source design: `docs/payment-policy-alpha-coverage-design.md`

Status: historical completed predecessor-v1 task record. Its commands are not
active CLI guidance; see `proof-ops-policy-ci.md` for revision-2 usage.

This document breaks the payment-policy alpha coverage design into
implementation milestones that can be executed one at a time. Source design
phase IDs keep the form `PAYALPHA-COV-0x`; implementation task IDs in this file
use `PAYALPHA-COV-T0x` so a later implementation pass can pick exactly one
task ID without overloading the source phase names.

## Scope

Implement payload-bound checked theory closure for the existing
`payment-policy-alpha` workflow in MPK. The implementation must make reserve
policy verification progress from the current one checked obligation to all
eight checked reserve obligations, then apply the same support across the five
positive payment-policy examples.

Out of scope:

- product-side ProofOps code in `../proof-ops`;
- database, network, payment-gateway, handler-effect, access-control, or IAM
  verification;
- changes to the `mpk.policy.evidence.v1` JSON schema;
- trusting the classifier, VIR, VC JSON, Markdown, strategy candidates, or AI
  explanation text as proof evidence.

## Design Phase Mapping

The source design describes five phases. This TODO expands them into smaller
implementation task milestones:

| Design phase | Implementation milestones |
| --- | --- |
| PAYALPHA-COV-01 baseline and closure plumbing | PAYALPHA-COV-T01 |
| PAYALPHA-COV-02 payload-bound linarith closure | PAYALPHA-COV-T02, PAYALPHA-COV-T03, PAYALPHA-COV-T04 |
| PAYALPHA-COV-03 reflexive branch bool closure | PAYALPHA-COV-T05, PAYALPHA-COV-T06 |
| PAYALPHA-COV-04 corpus-wide evidence coverage | PAYALPHA-COV-T07 |
| PAYALPHA-COV-05 docs and CI gate | PAYALPHA-COV-T08 |

The source phase PAYALPHA-COV-01 included the neutral `policy_theory_goal` setup
as part of baseline plumbing. This task plan intentionally keeps
PAYALPHA-COV-T01 behavior-preserving and implements the neutral linear-goal
extraction in PAYALPHA-COV-T03 before any payload-bound linarith closure
consumes it.

## Cross-Cutting Constraints

- `mpk-vc` owns neutral extraction from `VcObligation` and must not depend on
  `mpk-theory`, `mpk-cert`, `mpk-kernel`, `mpk-api`, or `mpk-cli`.
- `mpk-theory` owns public encoders and checkers for concrete theory
  certificate payloads.
- `mpk-cli` owns policy orchestration, proof-profile gating, certificate
  checking, certificate hashing, deterministic evidence IDs, and evidence
  rendering.
- `mpk-api` should keep its strategy metadata and generic strategy tests; it
  should not gain a dependency on `mpk-vc`.
- `mpk_verified` may be emitted only when a concrete `VcObligation` is closed by
  checked MPK theory-certificate code and the property references that exact
  obligation under `trusted_evidence.theory_certificates[*].checked_obligations`.
- Supported obligations that are not closed remain `proof_pending` and must not
  receive `CheckedDeclaration` or `CheckedTheoryCertificate` refs.
- Unsupported scan or classifier results retain the existing unsupported failure
  behavior.
- Unknown checker profiles retain the existing deterministic CLI validation
  error. Known non-strict profiles must be parsed as `ProofProfile` values and
  must not attempt theory closure.
- Policy closure code must not use
  `theory_strategy_certificate_evidence(TheoryStrategyKind::Linarith)` or any
  other static strategy fixture for policy proof evidence.
- Deterministic certificate IDs use the exact prefixes
  `theory:policy-linarith-` and `theory:policy-bool-tautology-` with four-digit
  counters in stable obligation order.

## Milestones

### PAYALPHA-COV-T01 Baseline Closure Plumbing

Status: Completed

Depends on: current `policy verify` behavior from POE-10.

Inputs:

- `docs/payment-policy-alpha-coverage-design.md`
- `crates/mpk-cli/src/policy_verify.rs`
- `crates/mpk-cli/tests/policy_verify.rs`
- `crates/mpk-vc/src/vc.rs`
- `crates/mpk-vc/src/policy_obligation.rs`

Likely touched files:

- `crates/mpk-cli/src/policy_verify.rs`
- `crates/mpk-cli/tests/policy_verify.rs`

Tasks:

1. Add a CLI-local `PolicyClosedObligation` model with fields for
   `obligation_id`, `certificate_id`, `theory`, `format`,
   `theory_certificate_hash`, and `evidence_note`.
2. Replace the `Option<String>` verified-obligation plumbing with a
   `BTreeMap<String, PolicyClosedObligation>` keyed by obligation ID.
3. Introduce `try_close_policy_obligations(...)` as the single policy-closure
   entry point. In this milestone it may delegate to the current first-linarith
   implementation so behavior stays unchanged.
4. Pass the closure map into `property_from_classification` and make
   `property_from_classification` create checked-theory refs from the map rather
   than from hard-coded linarith IDs.
5. Build and validate a local obligation-by-ID map from `vc_module.obligations`
   before closure. Duplicate IDs or classifier references with no matching
   obligation must fail with `policy verify proof closure failed: <detail>`.
6. Parse `checker_profile` through the existing proof-profile parser inside the
   planner before comparing against `ProofProfile::MvpStrict`. Keep unknown
   checker-profile rejection deterministic.
7. Add or adjust tests that prove:
   - reserve still reports `verified=1 proof_pending=7 unsupported=0`;
   - strict reserve still fails after writing evidence with
     `proof-pending properties=7`;
   - the generated checked-theory ref and trusted certificate entry still agree;
   - supported but unclosed properties still have helper refs only.

Deliverables:

- Behavior-preserving closure map plumbing.
- Existing reserve evidence shape preserved except for any intentional internal
  ordering that remains byte-deterministic.

Acceptance criteria:

- `policy_verify_reserve_writes_evidence_and_markdown` still passes with
  `status=proof_pending`, `verified=1`, and `proof_pending=7`.
- `policy_verify_strict_fails_after_writing_proof_pending_evidence` still fails
  with `proof-pending properties=7`.
- No `mpk.policy.evidence.v1` schema fields are added, removed, or renamed.
- No unclosed property receives trusted checked evidence.

Verification:

```sh
cargo fmt --all -- --check
cargo test -p mpk-cli --test policy_verify
git diff --check
```

Notes:

- This milestone intentionally does not add `mpk-vc` policy theory extraction.
- Keep the old static strategy witness isolated so PAYALPHA-COV-T04 can remove
  it from policy closure cleanly.

### PAYALPHA-COV-T02 Add Theory Certificate Encoders

Status: Completed

Depends on: none, but PAYALPHA-COV-T04 consumes the linarith encoder and
PAYALPHA-COV-T06 consumes the bool encoder.

Inputs:

- `crates/mpk-theory/src/linarith_cert.rs`
- `crates/mpk-theory/src/bool_cert.rs`
- `crates/mpk-theory/src/lib.rs`
- `crates/mpk-kernel/src/proof_theory.rs`
- `crates/mpk-cert/src/encode.rs`
- `crates/mpk-api/src/theory_strategy.rs`

Likely touched files:

- `crates/mpk-theory/src/linarith_cert.rs`
- `crates/mpk-theory/src/bool_cert.rs`
- `crates/mpk-theory/src/lib.rs`
- `crates/mpk-kernel/tests/*` or another non-cyclic test location

Tasks:

1. Add `pub fn encode_linarith_certificate(certificate:
   &LinarithCertificate) -> Vec<u8>`.
2. Encode linarith payloads exactly as the kernel decoder reads them:
   `MPKLINR0`, `u8 premise_count`, each inequality as `u8 term_count` followed
   by `(u32 variable, i128 coefficient)` pairs and `i128 constant`, then the
   goal inequality, `u8 combination_count`, and `(u32 premise_index, u64
   multiplier)` rows.
3. Add `pub fn encode_bool_certificate(certificate: &BoolCertificate) ->
   Vec<u8>`.
4. Encode bool payloads exactly as `decode_bool_certificate` reads them:
   `MPKBOOL0`, `u8 variable_count`, expression tags, `u16 row_count`, then
   `(u8 assignment_mask, u8 normalized_value)` rows.
5. Implement private bool-expression encoding for `BoolExpr::{Const, Var, Not,
   And, Or, Implies, Iff}` using the existing decoder tags.
6. Export both encoders from `crates/mpk-theory/src/lib.rs`.
7. Add tests that:
   - check a small linarith certificate and prove its encoded payload is
     accepted when wrapped in `mpk_cert::encode::TheoryCertificate`;
   - check a small bool tautology certificate and prove its encoded payload is
     accepted when wrapped in `TheoryCertificate`;
   - compare encoder bytes with existing static payload expectations where that
     is practical.

Deliverables:

- Public `mpk-theory` encoders for linarith and bool certificate payloads.
- Kernel-acceptance tests for the encoded payloads.

Acceptance criteria:

- Encoded payloads round-trip through the same kernel checker path that accepts
  policy evidence.
- `mpk-theory` does not acquire a dependency cycle. Because `mpk-kernel` already
  depends on `mpk-theory`, place kernel-acceptance tests in a downstream crate if
  adding `mpk-kernel` as a dev-dependency to `mpk-theory` would create a cycle.
- No policy verify behavior changes in this milestone.

Verification:

```sh
cargo fmt --all -- --check
cargo test -p mpk-theory linarith
cargo test -p mpk-theory bool
cargo test -p mpk-kernel theory
cargo test -p mpk-cli --test policy_verify
git diff --check
```

Notes:

- The encoder signatures intentionally return `Vec<u8>` to match the source
  design. Bounds and shape failures should be caught by the existing checker
  tests before payloads are used as trusted evidence.

### PAYALPHA-COV-T03 Extract Policy Linear Theory Goals

Status: Completed

Depends on: PAYALPHA-COV-T01.

Inputs:

- `docs/payment-policy-alpha-coverage-design.md`
- `crates/mpk-vc/src/vc.rs`
- `crates/mpk-vc/src/expr_encode.rs`
- `crates/mpk-vc/src/policy_obligation.rs`
- `examples/payment_policies/*/vc.json`
- `crates/mpk-vc/tests/payment_policy_examples.rs`

Likely touched files:

- `crates/mpk-vc/src/policy_theory_goal.rs`
- `crates/mpk-vc/src/lib.rs`
- `crates/mpk-vc/src/policy_obligation.rs` if helper functions must be shared
- `crates/mpk-vc/tests/payment_policy_examples.rs` or a new focused test file

Tasks:

1. Add `crates/mpk-vc/src/policy_theory_goal.rs` and export it from
   `crates/mpk-vc/src/lib.rs`.
2. Define the neutral data model:
   - `PolicyTheoryGoal { obligation_id, function_id, pattern, kind }`
   - `PolicyTheoryGoalKind::{Linear(PolicyLinearGoal),
     BoolTautology(PolicyBoolGoal)}`
   - `PolicyLinearGoal { variables, premises, goal }`
   - `PolicyLinearInequality { terms, constant }`
   - `PolicyLinearTerm { variable, coefficient }`
   - placeholder bool structs/enums if needed by the shared enum.
3. Implement
   `policy_theory_goal_from_obligation(&VcObligation,
   &PaymentPolicyObligationClassification) -> Result<Option<PolicyTheoryGoal>,
   PolicyTheoryGoalError>`.
4. For linear patterns, parse only `Std.BitVec.BV64.sge`, `sle`, `sgt`, and
   `slt` over operands that are `Var`, `Result`, or signed BV64 literals.
5. Normalize comparisons to `sum(terms) + constant <= 0`:
   - `sge(lhs, rhs)` as `rhs - lhs <= 0`;
   - `sle(lhs, rhs)` as `lhs - rhs <= 0`;
   - `sgt(lhs, rhs)` as `rhs - lhs + 1 <= 0`;
   - `slt(lhs, rhs)` as `lhs - rhs + 1 <= 0`.
6. Use stable variable keys `var:<name>` and `result:<index>`, assign numeric
   IDs from a `BTreeMap`, reject duplicates, reject more than 64 variables, sort
   terms, and drop zero coefficients.
7. Return `Ok(None)` for unsupported arithmetic, non-BV64 predicates,
   functions, conversions, or shapes that are outside the alpha closure subset
   but do not contradict the classifier result.
8. Return `Err` when the classifier says a supported linear pattern should be
   present but the concrete conclusion is internally inconsistent, not a
   comparison, or otherwise contradicts that classification.
9. Add unit tests for:
   - exact premise closure candidates;
   - identity goals such as `x <= x`;
   - strict-branch premise candidates such as `requested > balance` proving
     `balance <= requested`;
   - signed BV64 literal handling;
   - unsupported arithmetic returning `Ok(None)`;
   - over-limit or duplicate variable failures.

Deliverables:

- Neutral, test-covered linear goal extraction in `mpk-vc`.
- No direct theory-certificate construction in `mpk-vc`.

Acceptance criteria:

- The five positive payment-policy examples still classify as eight supported
  obligations each.
- Linear extraction can produce goals for the non-negative and bound obligations
  needed by PAYALPHA-COV-T04.
- Branch equality obligations are either represented as bool goals only after
  PAYALPHA-COV-T05 or left non-applicable in this milestone.
- No CLI behavior changes are required by this milestone.

Verification:

```sh
cargo fmt --all -- --check
cargo test -p mpk-vc --test payment_policy_examples
cargo test -p mpk-vc policy_theory_goal
git diff --check
```

Notes:

- Keep the local max-variable constant in `mpk-vc` at 64, matching
  `mpk_theory::MAX_LINARITH_VARIABLES`, without importing `mpk-theory`.

### PAYALPHA-COV-T04 Enable Payload-Bound Linarith Closure

Status: Completed

Depends on: PAYALPHA-COV-T01, PAYALPHA-COV-T02, PAYALPHA-COV-T03.

Inputs:

- `crates/mpk-cli/src/policy_verify.rs`
- `crates/mpk-cli/Cargo.toml`
- `crates/mpk-cli/tests/policy_verify.rs`
- `crates/mpk-vc/src/policy_theory_goal.rs`
- `crates/mpk-theory/src/linarith_cert.rs`
- `crates/mpk-cert/src/hash.rs`
- `crates/mpk-cert/src/encode.rs`
- `crates/mpk-kernel/src/proof_theory.rs`

Likely touched files:

- `crates/mpk-cli/Cargo.toml`
- `crates/mpk-cli/src/policy_verify.rs`
- `crates/mpk-cli/tests/policy_verify.rs`

Tasks:

1. Add direct `mpk-cert` and `mpk-theory` dependencies to `mpk-cli`.
2. In `try_close_policy_obligations`, parse `checker_profile` once and return
   an empty closure map for known profiles other than `ProofProfile::MvpStrict`.
3. Validate every classifier entry against `PolicyStrategyMetadata` before
   attempting closure. Non-applicable validated entries stay `proof_pending`;
   validation inconsistency fails with `policy verify proof closure failed:
   <detail>`.
4. Iterate obligations in stable ID order. Skip unsupported classifications
   without adding trusted evidence.
5. For each linear `PolicyTheoryGoal`, build a concrete `LinarithCertificate`:
   - exact premise match uses the matching premise with multiplier 1;
   - strict branch premise with positive slack uses that premise with multiplier
     1;
   - identity goals with no remaining terms use an empty combination;
   - other shapes return no closure and remain `proof_pending`.
6. Check each generated certificate with `check_linarith_certificate` before it
   is used.
7. Encode each checked certificate with `encode_linarith_certificate`, wrap it
   in `TheoryCertificate { format: LINARITH_CERT_FORMAT, payload }`, and run the
   kernel theory checker path if needed to prove the encoded bytes are accepted.
8. Compute `theory_certificate_hash` from
   `encode_theory_certificate(&certificate)`,
   `hash_with_domain(HashDomain::TheoryCertificate, canonical)`, and
   `hash_hex`.
9. Emit one checked obligation per theory certificate with deterministic IDs
   `theory:policy-linarith-0001`, `theory:policy-linarith-0002`, and so on.
10. Remove the policy closure dependency on
    `theory_strategy_certificate_evidence(TheoryStrategyKind::Linarith)` and
    the old static witness path.
11. Update property notes to distinguish exact premise, identity, and
    branch-premise linarith closure reasons.
12. Update reserve CLI tests:
    - normal reserve verify expects `status=proof_pending`,
      `verified=6 proof_pending=2 unsupported=0`;
    - strict reserve verify still fails, now with `proof-pending properties=2`;
    - trusted evidence has six linarith certificates;
    - every verified property references exactly one checked theory certificate;
    - every pending branch equality property has helper refs only.
13. Add a non-strict checker-profile test proving `mvp-structural` or
    `core-bootstrap` produces no checked theory closures and remains
    proof-pending.
14. Add a tamper regression for the linear closure path. If the concrete VC
    conclusion or generated linarith certificate no longer matches the checked
    normalized goal, the policy closure path must not emit `mpk_verified`; an
    internal checked-certificate failure must surface as `policy verify proof
    closure failed: <detail>`.

Deliverables:

- Payload-bound linarith closure for reserve non-negative and bound
  obligations.
- Reserve moves from one verified property to six verified properties.

Acceptance criteria:

- Reserve non-strict CLI output is exactly
  `status=proof_pending verified=6 proof_pending=2 unsupported=0`.
- Reserve strict CLI failure is exactly
  `policy verify failed: proof-pending properties=2`.
- `! rg -n "theory_strategy_certificate_evidence|theoryWitness" crates/mpk-cli/src/policy_verify.rs`
  succeeds, proving no policy closure use remains.
- Each checked linarith certificate hash changes with its concrete VC payload
  when the obligation changes.
- Unknown checker profiles still fail before evidence is written through the
  existing CLI validation path.

Verification:

```sh
cargo fmt --all -- --check
cargo test -p mpk-theory linarith
cargo test -p mpk-vc --test payment_policy_examples
cargo test -p mpk-cli --test policy_verify
! rg -n "theory_strategy_certificate_evidence|theoryWitness" crates/mpk-cli/src/policy_verify.rs
git diff --check
```

Notes:

- The final `! rg` command is expected to produce no matches in
  `policy_verify.rs` and therefore exit successfully.

### PAYALPHA-COV-T05 Extract Reflexive Branch Bool Goals

Status: Completed

Depends on: PAYALPHA-COV-T03.

Inputs:

- `crates/mpk-vc/src/policy_theory_goal.rs`
- `crates/mpk-vc/src/expr_encode.rs`
- `crates/mpk-vc/src/policy_obligation.rs`
- `examples/payment_policies/*/vc.json`

Likely touched files:

- `crates/mpk-vc/src/policy_theory_goal.rs`
- `crates/mpk-vc/tests/payment_policy_examples.rs` or a focused
  `policy_theory_goal` test file

Tasks:

1. Add bool goal structs if they were not fully added in PAYALPHA-COV-T03:
   - `PolicyBoolGoal { reason, tautology }`
   - `PolicyBoolTautologyReason::ReflexiveSelectedBranchDisjunct`
   - `PolicyBoolTautology::{TrueOrOpaque, OpaqueOrTrue}`.
2. Extend `policy_theory_goal_from_obligation` for
   `SelectedBranchResultEqualsInput`.
3. Recognize only `Std.Bool.or` conclusions with exactly two disjuncts.
4. Recognize a disjunct as true only when it is syntactically
   `Std.Eq(term, term)` with identical term structure.
5. Preserve the reflexive side:
   - left reflexive disjunct becomes `TrueOrOpaque`;
   - right reflexive disjunct becomes `OpaqueOrTrue`.
6. Treat the other disjunct as opaque. Do not try to prove or inspect its
   equality.
7. Return `Ok(None)` for all other branch equality shapes so unsupported bool
   structures remain pending until a later design expands them.
8. Add tests for:
   - reserve branch equality producing one `TrueOrOpaque` and one
     `OpaqueOrTrue` where applicable;
   - non-`or` branch equality returning `Ok(None)`;
   - `or` with the wrong arity returning `Ok(None)` or the documented
     extraction error;
   - non-reflexive equality disjuncts returning `Ok(None)`.

Deliverables:

- Neutral bool tautology goal extraction in `mpk-vc`.
- No CLI bool certificate closure yet.

Acceptance criteria:

- Branch equality extraction never marks a property verified by itself.
- The extraction result contains no trusted evidence and no checker-facing
  payload bytes.
- Existing linarith CLI behavior from PAYALPHA-COV-T04 remains unchanged.

Verification:

```sh
cargo fmt --all -- --check
cargo test -p mpk-vc policy_theory_goal
cargo test -p mpk-vc --test payment_policy_examples
cargo test -p mpk-cli --test policy_verify
git diff --check
```

### PAYALPHA-COV-T06 Enable Payload-Bound Bool Closure

Status: Completed

Depends on: PAYALPHA-COV-T02, PAYALPHA-COV-T04, PAYALPHA-COV-T05.

Inputs:

- `crates/mpk-cli/src/policy_verify.rs`
- `crates/mpk-cli/tests/policy_verify.rs`
- `crates/mpk-theory/src/bool_cert.rs`
- `crates/mpk-vc/src/policy_theory_goal.rs`

Likely touched files:

- `crates/mpk-cli/src/policy_verify.rs`
- `crates/mpk-cli/tests/policy_verify.rs`

Tasks:

1. Extend the policy closure planner to handle
   `PolicyTheoryGoalKind::BoolTautology`.
2. Build `BoolCertificate` values for:
   - `TrueOrOpaque` as `BoolExpr::Or(Const(true), Var(0))`;
   - `OpaqueOrTrue` as `BoolExpr::Or(Var(0), Const(true))`.
3. Use `variable_count = 1` and exactly two rows:
   - assignment `[false]` normalizes to `true`;
   - assignment `[true]` normalizes to `true`.
4. Check each bool certificate with `check_bool_certificate` before evidence is
   emitted.
5. Encode each checked certificate with `encode_bool_certificate`, wrap it in a
   `TheoryCertificate`, and compute the canonical theory-certificate hash by
   the same `mpk-cert` path used for linarith.
6. Emit deterministic bool IDs with the
   `theory:policy-bool-tautology-0001` prefix and four-digit counters.
7. Attach checked theory evidence to the two reserve branch equality
   obligations.
8. Update reserve tests:
   - normal reserve verify expects `status=verified`,
     `verified=8 proof_pending=0 unsupported=0`;
   - strict reserve verify succeeds and writes evidence;
   - no reserve property remains `proof_pending`;
   - trusted evidence includes six linarith certificates and two bool tautology
     certificates;
   - every `mpk_verified` property has exactly one checked-theory ref.
9. Keep non-strict checker-profile tests proof-pending.
10. Add a bool-certificate tamper regression proving a malformed normalized
    bool payload is rejected before any branch equality property is marked
    `mpk_verified`.

Deliverables:

- Payload-bound bool tautology closure for reflexive selected-branch equality.
- Reserve strict verification succeeds without adding a new fixture.

Acceptance criteria:

- Reserve CLI output is exactly
  `status=verified verified=8 proof_pending=0 unsupported=0`.
- `mpk policy verify ... --strict` succeeds for the existing reserve example.
- Bool certificate hashes are computed from canonical encoded theory
  certificates, not from static strategy fixture helpers.
- No helper-only or unsupported property is promoted to `mpk_verified`.

Verification:

```sh
cargo fmt --all -- --check
cargo test -p mpk-theory bool
cargo test -p mpk-vc policy_theory_goal
cargo test -p mpk-cli --test policy_verify
git diff --check
```

### PAYALPHA-COV-T07 Verify the Positive Payment-Policy Corpus

Status: Completed

Depends on: PAYALPHA-COV-T06.

Inputs:

- `examples/payment_policies/manifest.json`
- `examples/payment_policies/{reserve,refund,discount,fee,points}`
- `crates/mpk-cli/tests/policy_verify.rs`
- `crates/mpk-vc/tests/payment_policy_examples.rs`
- `scripts/check-all.sh`

Likely touched files:

- `crates/mpk-cli/tests/policy_verify.rs`
- `scripts/check-all.sh` only if the existing checks do not cover the new gate
- payment-policy example files only if real fixture drift is discovered

Tasks:

1. Add a table-driven `policy_verify_positive_payment_corpus_has_expected_counts`
   test covering `reserve`, `refund`, `discount`, `fee`, and `points`.
2. Reuse the existing `go2vir` build helper in CLI tests so the corpus test
   does not rebuild the binary for every example.
3. For every positive example, run `mpk policy verify --strict` with
   `--strategy-profile payment-policy-alpha` and `--checker-profile mvp-strict`
   and assert the command succeeds with
   `verified=8 proof_pending=0 unsupported=0`.
4. For `refund`, `discount`, `fee`, and `points`, assert at least one property
   is `mpk_verified` and references checked theory evidence. This proves the
   implementation is not reserve-only.
5. Add pattern-specific assertions that:
   - refund bound obligations close under
     `RefundBoundedByAvailablePaidAmount`;
   - discount and fee obligations close under
     `FeeOrDiscountBoundedByCap`;
   - points obligations close under `ResultBoundedByInput`.
6. Parse every generated evidence JSON with `PolicyEvidenceReport::from_json`.
7. Add a determinism assertion for at least one non-reserve example, preferably
   `refund`, proving repeated evidence output is byte-identical and
   multi-certificate numbering is stable outside reserve.
8. Keep the existing `mpk-vc` fixture test that asserts each example still has
   two non-negative, four bound, and two branch-equality obligations.
9. If fixture regeneration is required, use the existing
   `MPK_UPDATE_PAYMENT_POLICY_EXAMPLES=1 cargo test -p mpk-vc --test
   payment_policy_examples` flow and commit only intentional fixture changes.
10. Run Go tests for all five example packages.
11. Update `scripts/check-all.sh` only if it does not already run a CLI test
    that would fail on loss of strict corpus coverage.

Deliverables:

- Corpus-level regression coverage proving all five positive examples pass
  strict verification under `mvp-strict`.

Acceptance criteria:

- All five positive examples pass `mpk policy verify --strict` and produce
  `verified=8 proof_pending=0 unsupported=0`.
- At least one `refund`, one `discount`, one `fee`, and one `points` property is
  backed by checked theory-certificate evidence.
- The `FeeOrDiscountBoundedByCap` implementation is not discount-cap-only; fee
  floor examples must pass.
- Repeated evidence generation is deterministic for at least one non-reserve
  example.
- Every generated corpus evidence JSON parses with
  `PolicyEvidenceReport::from_json`.
- Existing scan and VC classification tests still pass.

Verification:

```sh
for policy in reserve refund discount fee points; do
  (cd "examples/payment_policies/$policy" && go test ./...)
done
cargo fmt --all -- --check
cargo test -p mpk-vc --test payment_policy_examples
cargo test -p mpk-cli --test policy_verify
./scripts/check-all.sh
git diff --check
```

### PAYALPHA-COV-T08 Refresh Docs, Fixture, and Local Verification Guidance

Status: Completed

Depends on: PAYALPHA-COV-T07.

Inputs:

- `docs/payment-policy-alpha-coverage-design.md`
- `docs/alpha-demo.md`
- `docs/proof-ops-policy-ci.md`
- `examples/payment_policies/README.md`
- `examples/payment_policies/reserve/README.md`
- `examples/payment_policies/reserve/evidence_alpha.json`
- `scripts/check-fast.sh`
- `scripts/check-all.sh`

Likely touched files:

- `docs/alpha-demo.md`
- `docs/proof-ops-policy-ci.md`
- `examples/payment_policies/README.md`
- `examples/payment_policies/reserve/README.md`
- `examples/payment_policies/reserve/evidence_alpha.json`
- `scripts/check-fast.sh` or `scripts/check-all.sh` only if needed

Tasks:

1. Build `target/debug/go2vir` first if it is not already present:

   ```sh
   (cd go-tools/go2vir && go build -o ../../target/debug/go2vir .)
   ```

2. Refresh the tracked reserve evidence fixture with the explicit update flag:

   ```sh
   cargo run --quiet -p mpk-cli -- policy verify examples/payment_policies/reserve \
     --function example.com/payment/reserve.ApprovedReserveCents \
     --contract examples/payment_policies/reserve/policy_contract.json \
     --strategy-profile payment-policy-alpha \
     --checker-profile mvp-strict \
     --evidence-json examples/payment_policies/reserve/evidence_alpha.json \
     --evidence-md /tmp/mpk-reserve-evidence.md \
     --frontend-bundle target/debug/go2vir \
     --strict \
     --update-fixtures
   ```

3. Verify `examples/payment_policies/reserve/evidence_alpha.json` now has
   eight `mpk_verified` properties, no `proof_pending` properties, no
   unsupported properties, and checked theory-certificate refs for every
   property.
4. Update `examples/payment_policies/README.md` with the new expected strict
   verification result.
5. Update `examples/payment_policies/reserve/README.md` so it no longer says the
   fixture is only representative of one checked obligation.
6. Update `docs/alpha-demo.md` to say reserve `policy verify --strict` is the
   strict demo path after the coverage work lands.
7. Update `docs/proof-ops-policy-ci.md` to distinguish:
   - non-strict helper-artifact generation and drift review;
   - strict `mvp-strict` checked-theory gate for supported alpha payment
     policies.
8. Add or update a fast check only if the current scripts do not already fail
   when reserve strict verification or positive corpus coverage regresses.
9. Search docs for stale count text such as `verified=1`, `proof_pending=7`,
   `proof_pending=2`, and `may report status=proof_pending`, then update only
   statements made stale by the completed implementation.

Deliverables:

- Updated reserve evidence fixture.
- Docs that accurately describe the final strict payment-policy alpha path.
- local verification guidance that treats checked theory evidence as the gate
  for supported alpha policies.

Acceptance criteria:

- No docs claim the reserve happy path remains proof-pending after the final
  milestone.
- The fixture remains deterministic and is refreshed only through
  `--update-fixtures`.
- The distinction between helper artifacts and trusted checked evidence remains
  explicit in all updated docs.

Verification:

```sh
cargo fmt --all -- --check
cargo test -p mpk-cli --test policy_verify
cargo test -p mpk-vc --test payment_policy_examples
./scripts/check-fast.sh
rg -n 'verified=1|proof_pending=7|proof_pending=2|may report `status=proof_pending`|one `mpk_verified`' docs examples/payment_policies || printf 'no stale-text hits\n'
git diff --check
```

Notes:

- The final stale-text search is an audit. It may still find historical design
  references in `docs/payment-policy-alpha-coverage-design.md`; review each hit
  and update only user-facing instructions or final-state claims.

## Final Done Definition

The overall payment-policy alpha coverage work is complete when:

- reserve strict verification succeeds with
  `verified=8 proof_pending=0 unsupported=0`;
- all five positive payment-policy examples pass `mpk policy verify --strict`
  with
  `verified=8 proof_pending=0 unsupported=0`;
- refund, discount, fee, and points each contain at least one checked
  `mpk_verified` property;
- every `mpk_verified` property references checked theory-certificate evidence
  under `trusted_evidence`;
- unsupported and unclosed properties still cannot receive trusted evidence;
- non-`mvp-strict` checker profiles do not attempt theory closure;
- unknown profiles still reject deterministically;
- docs and the reserve fixture reflect the final strict path;
- `./scripts/check-all.sh` passes.

## Review Ledger

Resolved findings:

- Split the source design's large linarith phase into encoder, extraction, and
  CLI-closure milestones so each implementation pass has a narrow ownership
  boundary.
- Gave implementation tasks distinct `PAYALPHA-COV-T0x` IDs so they do not
  overload the source design's `PAYALPHA-COV-0x` phase IDs.
- Split bool extraction from bool closure so `mpk-vc` can remain proof-neutral.
- Added an explicit note that kernel-acceptance tests for encoders may need to
  live outside `mpk-theory` to avoid a dependency cycle.
- Replaced copy-paste-hostile search commands with executable forms, including
  a no-match `! rg` check and a single-quoted stale-text search pattern.
- Restored source-design coverage for strict corpus verification, non-reserve
  determinism, schema parsing, and tamper regressions.
- Preserved the trust-boundary distinction between `strategy_profile`,
  `checker_profile`, and `axiom_profile`.
- Made fixture refresh and stale-doc search explicit in the final milestone.

Remaining findings after self-review:

- None.

Review checks applied:

- Verified milestone dependencies against the source design phase order.
- Checked that every milestone has likely touched files, concrete tasks,
  acceptance criteria, and verification commands.
- Checked trust-boundary-sensitive terms against the existing
  `mpk.policy.evidence.v1` model.
