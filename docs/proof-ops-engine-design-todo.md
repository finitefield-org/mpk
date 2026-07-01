# ProofOps Engine Design Todo

Source: `docs/proof-ops-engine-design.md`

Status: implementation-ready task breakdown

## Scope

This task list covers MPK-side work required by the ProofOps product. It turns
the existing `go2gir`, `mpk-vc`, `mpk-api`, `mpk-kernel`, and `mpk-cli` pieces
into a product-ready policy scan and verification workflow for payment-policy
functions.

In scope:

- stable scan and evidence JSON schemas;
- `mpk policy scan`;
- `mpk policy verify`;
- payment-policy strategy profile selection;
- proof/evidence labeling that preserves the MPK trust boundary;
- regression corpora for payment policies;
- helper lint for visible precondition enforcement;
- Markdown evidence reports and CI examples.

Out of scope:

- Gemini prompts, agent orchestration, web forms, billing, and customer storage;
- whole-service verification;
- database, network, payment-gateway, handler-effect, or full IAM verification;
- treating Go source, contract JSON, GIR, VC JSON, CI status, AI logs, or report
  prose as proof evidence;
- broad Go language support beyond the existing Go subset v0.

## Cross-Cutting Constraints

- Trusted proof evidence remains limited to canonical `.mpcert` bytes, checked
  theory certificates, checker verdicts, recomputed hashes, axiom reports, and
  hash-pinned imports as defined in `develop/specs/TRUST_BOUNDARY_V0.md`.
- `strategy_profile` is product workflow selection. It must not be confused with
  `checker_profile` or axiom policy profiles.
- Any command that reads Go source, contract JSON, GIR, VC JSON, or source maps
  must label those results as helper analysis unless checked certificate or
  checked theory evidence accepts the corresponding property.
- Unsupported input must fail closed with deterministic error codes.
- JSON is the stable product API. Markdown reports are derived views.
- All new public schemas must include a `schema` string and deny unknown fields
  in Rust-facing deserialization paths.

## Milestones

### POE-01 Define Policy Scan Schema

- Status: Pending
- Depends on: None
- Inputs:
  - `docs/proof-ops-engine-design.md`
  - `develop/specs/TRUST_BOUNDARY_V0.md`
  - `develop/specs/GO_SUBSET_V0.md`
  - `develop/specs/GIR_V0.md`
- Likely touched files:
  - `docs/proof-ops-engine-design.md`
  - `crates/mpk-cli/src/main.rs`
  - `crates/mpk-cli/tests/policy_scan.rs`
- Deliverables:
  - `mpk.policy.scan.v0` Rust structs.
  - Deterministic JSON serialization for scan success and rejection cases.
  - Status enum with exactly `ready`, `needs_refactor`, and `unsupported`.
  - Feature report entries with code, message, source path, function id when
    known, and helper-evidence label.
- Acceptance criteria:
  - Scan JSON includes `schema`, `target`, `source`, `contract`, `readiness`,
    `supported_features`, `rejected_features`, `preconditions`, and
    `proof_acceptance`.
  - `proof_acceptance` is always `false` or absent in scan output; scan cannot
    report a verified property.
  - Unknown fields reject in tests for any scan request/config JSON that is
    deserialized by MPK.
  - Snapshot tests cover a ready order-policy function and one unsupported
    function.
- Verification:
  - `cargo test -p mpk-cli --test policy_scan`
  - `cargo test -p mpk-cli policy_scan_schema`
  - `git diff --check`
- Notes:
  - This milestone defines the schema only. It may use hand-built fixture data
    and must not shell out to `go2gir` yet.

### POE-02 Add `mpk policy` CLI Routing

- Status: Pending
- Depends on: POE-01
- Inputs:
  - `crates/mpk-cli/src/main.rs`
  - Existing command patterns for `check`, `axiom-report`, `package check`, and
    `package verify-certs`.
- Likely touched files:
  - `crates/mpk-cli/src/main.rs`
  - `crates/mpk-cli/tests/policy_cli.rs`
- Deliverables:
  - `mpk policy scan` route.
  - `mpk policy verify` route stub.
  - Usage text documenting required flags:
    - `--function`
    - `--contract`
    - `--json-out` for scan
    - `--strategy-profile`
    - `--checker-profile`
    - `--evidence-json`
    - `--evidence-md`
  - Deterministic usage errors for missing, duplicated, unknown, and empty flag
    values.
- Acceptance criteria:
  - `mpk policy scan --help` and `mpk policy verify --help` return success.
  - Missing required flags return exit code 2 and stable messages.
  - `--strategy-profile` is accepted only on `policy verify`.
  - `--checker-profile` accepts only existing checker profile names:
    `core-bootstrap`, `mvp-structural`, and `mvp-strict`.
  - The verify route clearly reports `not implemented` until POE-08.
- Verification:
  - `cargo test -p mpk-cli --test policy_cli`
  - `cargo run --quiet -p mpk-cli -- policy scan --help`
  - `cargo run --quiet -p mpk-cli -- policy verify --help`
- Notes:
  - Keep CLI parsing non-interactive and dependency-free unless a parser crate is
    already adopted elsewhere.

### POE-03 Implement Scan Runner Around `go2gir`

- Status: Pending
- Depends on: POE-02
- Inputs:
  - `go-tools/go2gir/main.go`
  - `go-tools/go2gir/README.md`
  - `docs/web-system-integration.md`
  - `examples/order_policy`
- Likely touched files:
  - `crates/mpk-cli/src/main.rs`
  - `crates/mpk-cli/src/policy_scan.rs`
  - `crates/mpk-cli/tests/policy_scan.rs`
  - `examples/order_policy`
- Deliverables:
  - Scan runner that executes a configured `go2gir` binary or a default
    `target/debug/go2gir` path.
  - Parsing of `mpk.go2gir.cli.v0` output.
  - Mapping from `go2gir` statuses to `ready`, `needs_refactor`, or
    `unsupported`.
  - Inclusion of Go version, frontend binary hash, source hashes, GIR hash, and
    rejected-feature details when available.
- Acceptance criteria:
  - `examples/order_policy` scans as `ready`.
  - A fixture containing an unsupported Go feature scans as `unsupported`.
  - A function that is syntactically supported but lacks a usable contract scans
    as `needs_refactor`.
  - A non-zero `go2gir` exit with valid rejected-feature JSON is not treated as a
    process failure; it is represented as scan output.
  - A non-zero `go2gir` exit without valid JSON is a deterministic CLI input
    error.
- Verification:
  - `(cd go-tools/go2gir && go build -o ../../target/debug/go2gir .)`
  - `cargo test -p mpk-cli --test policy_scan`
  - `cargo run --quiet -p mpk-cli -- policy scan examples/order_policy --function example.com/orderpolicy.ApprovedReserveCents --contract examples/order_policy/policy_contract.json --json-out /tmp/mpk-policy-scan.json`
- Notes:
  - The scan runner may use process execution. It must label all `go2gir` output
    as helper analysis.

### POE-04 Define Evidence Schema

- Status: Pending
- Depends on: POE-01
- Inputs:
  - `docs/proof-ops-engine-design.md`
  - `develop/specs/TRUST_BOUNDARY_V0.md`
  - `package-manifest.md`
  - `scripts/generate-release-report.py`
- Likely touched files:
  - `crates/mpk-cli/src/policy_evidence.rs`
  - `crates/mpk-cli/tests/policy_evidence.rs`
  - `docs/proof-ops-engine-design.md`
- Deliverables:
  - `mpk.policy.evidence.v0` Rust structs.
  - Stable JSON serialization for accepted, helper-only, and rejected evidence.
  - Property evidence status enum with at least:
    - `mpk_verified`
    - `proof_pending`
    - `helper_only`
    - `unsupported`
  - Separate fields for `strategy_profile`, `checker_profile`, and
    `allowed_axiom_profiles`.
- Acceptance criteria:
  - Evidence JSON has separate `trusted_evidence` and `helper_artifacts`
    sections.
  - A property can be `mpk_verified` only when it references checked
    declaration evidence or checked theory-certificate evidence.
  - GIR hash and VC hash appear only under `helper_artifacts`.
  - Certificate hash, export hash, axiom report hash, Rust checker verdict, and
    reference-checker verdict appear only under `trusted_evidence`.
  - Tests reject unknown property status values and unknown top-level fields.
- Verification:
  - `cargo test -p mpk-cli --test policy_evidence`
  - `cargo test -p mpk-cli policy_evidence_schema`
  - `git diff --check`
- Notes:
  - Do not reuse `release-report.json` as the policy evidence schema; it is a
    release artifact, not a product-policy artifact.

### POE-05 Emit Markdown Evidence Reports From JSON

- Status: Pending
- Depends on: POE-04
- Inputs:
  - `docs/proof-ops-engine-design.md`
  - `docs/alpha-demo.md`
- Likely touched files:
  - `crates/mpk-cli/src/policy_report.rs`
  - `crates/mpk-cli/tests/policy_report.rs`
- Deliverables:
  - Deterministic Markdown renderer for `mpk.policy.evidence.v0`.
  - Sections for target function, verified properties, proof-pending
    properties, unsupported properties, required preconditions, hashes,
    checker verdicts, reproduction commands, and trust-boundary notes.
- Acceptance criteria:
  - Markdown output never becomes the source of truth; tests construct Markdown
    from evidence JSON fixtures.
  - The report explicitly says GIR, VC JSON, source text, contract JSON, and CI
    status are helper artifacts.
  - Line ordering is deterministic.
  - A rejected or helper-only evidence JSON still renders with a clear
    non-verified status.
- Verification:
  - `cargo test -p mpk-cli --test policy_report`
  - `cargo test -p mpk-cli policy_report`
- Notes:
  - Avoid customer-facing marketing language in MPK. ProofOps can add branding
    outside this repository.

### POE-06 Build Payment Policy Corpus Skeleton

- Status: Pending
- Depends on: POE-03, POE-04
- Inputs:
  - `examples/order_policy`
  - `fixtures/go-alpha`
  - `crates/mpk-vc/tests/max64_example.rs`
- Likely touched files:
  - `examples/payment_policies/reserve`
  - `examples/payment_policies/refund`
  - `examples/payment_policies/discount`
  - `examples/payment_policies/fee`
  - `examples/payment_policies/points`
  - `crates/mpk-vc/tests/payment_policy_examples.rs`
  - `crates/mpk-cli/tests/policy_scan.rs`
- Deliverables:
  - Five small pure Go policy examples:
    - reserve cap;
    - partial refund cap;
    - discount cap;
    - platform fee floor or cap;
    - points redemption cap.
  - Contract sidecars for each example.
  - Checked-in GIR, VC, and skeleton fixtures for examples supported by current
    VC generation.
  - Negative examples for unsupported floats, maps, pointers, and missing
    contract postconditions.
- Acceptance criteria:
  - Every positive example compiles with `go test ./...`.
  - `go2gir` lowers every positive example and rejects every negative example
    with deterministic rejected-feature JSON.
  - `cargo test -p mpk-vc --test payment_policy_examples` verifies stable VC and
    skeleton output for positive examples.
  - Each example README states which artifacts are helper artifacts and which
    are trusted proof evidence.
- Verification:
  - `(cd examples/payment_policies/reserve && go test ./...)`
  - `(cd examples/payment_policies/refund && go test ./...)`
  - `(cd examples/payment_policies/discount && go test ./...)`
  - `(cd examples/payment_policies/fee && go test ./...)`
  - `(cd examples/payment_policies/points && go test ./...)`
  - `(cd go-tools/go2gir && go test -count=1 ./...)`
  - `cargo test -p mpk-vc --test payment_policy_examples`
- Notes:
  - This milestone creates examples and helper fixtures. It does not require
    all properties to close to checked proof evidence.

### POE-07 Define Strategy Profile Metadata

- Status: Pending
- Depends on: POE-04, POE-06
- Inputs:
  - `docs/proof-ops-engine-design.md`
  - `crates/mpk-api/src/strategies.rs`
  - `crates/mpk-api/src/theory_strategy.rs`
  - `develop/specs/AXIOM_POLICY_V0.md`
- Likely touched files:
  - `crates/mpk-api/src/policy_strategy.rs`
  - `crates/mpk-api/tests/policy_strategy.rs`
  - `crates/mpk-cli/src/policy_evidence.rs`
- Deliverables:
  - Strategy profile enum with `payment-policy-alpha`.
  - Mapping from strategy profile to allowed obligation patterns and candidate
    theory strategies.
  - Rejection reason for an obligation outside the selected strategy profile.
  - Evidence output that records the strategy profile separately from checker
    profile and axiom policy profile.
- Acceptance criteria:
  - Tests prove `payment-policy-alpha` is accepted as a strategy profile.
  - Tests prove unknown strategy profiles reject deterministically.
  - Tests prove the same evidence can record `strategy_profile:
    payment-policy-alpha` and `checker_profile: mvp-strict` without conflating
    them.
  - Profile metadata does not weaken `ProofProfile::MvpStrict` requirements for
    theory proof nodes.
- Verification:
  - `cargo test -p mpk-api --test policy_strategy`
  - `cargo test -p mpk-cli --test policy_evidence`
- Notes:
  - The existing `ProofProfile` type is a checker/proof-node profile. Do not
    rename it to strategy profile.

### POE-08 Implement Basic Payment Strategy Classification

- Status: Pending
- Depends on: POE-07
- Inputs:
  - `crates/mpk-vc/src/vc.rs`
  - `crates/mpk-vc/src/expr_encode.rs`
  - `examples/payment_policies`
- Likely touched files:
  - `crates/mpk-vc/src/policy_obligation.rs`
  - `crates/mpk-vc/tests/payment_policy_examples.rs`
  - `crates/mpk-api/tests/policy_strategy.rs`
- Deliverables:
  - Classifier for simple payment obligations:
    - non-negative result;
    - result bounded by an input;
    - refund bounded by available paid amount;
    - fee or discount bounded by cap;
    - selected branch result equals an input.
  - Classification output used by evidence as helper analysis until proof
    evidence closes the property.
  - `unsupported_property` diagnostics for obligations outside the classifier.
- Acceptance criteria:
  - Reserve, refund, discount, fee, and points fixtures classify expected
    obligations.
  - Unsupported boolean structure, unsupported arithmetic, or unsupported type
    produces deterministic `unsupported_property` output.
  - Classifier output alone never sets property status to `mpk_verified`.
- Verification:
  - `cargo test -p mpk-vc --test payment_policy_examples`
  - `cargo test -p mpk-api --test policy_strategy`
- Notes:
  - Prefer a conservative recognizer over broad expression rewriting. Unknown
    patterns must remain proof-pending or unsupported.

### POE-09 Close First Reserve Policy With Checked Theory Evidence

- Status: Pending
- Depends on: POE-08
- Inputs:
  - `examples/order_policy`
  - `crates/mpk-api/src/theory_strategy.rs`
  - `crates/mpk-theory/src/linarith_cert.rs`
  - `crates/mpk-api/tests/strategies.rs`
- Likely touched files:
  - `crates/mpk-api/src/policy_strategy.rs`
  - `crates/mpk-api/tests/policy_strategy.rs`
  - `examples/payment_policies/reserve`
  - `crates/mpk-cli/tests/policy_evidence.rs`
- Deliverables:
  - End-to-end checked theory strategy for the simplest reserve-cap obligation
    shape supported by the current `linarith` checker.
  - Evidence JSON marking only the closed reserve property as `mpk_verified`.
  - Remaining reserve properties marked `proof_pending` or `unsupported` if they
    do not yet close.
- Acceptance criteria:
  - A test constructs the reserve obligation, runs the strategy under
    `mvp-strict`, and observes a `ProofNode::Theory`.
  - The test fails under a non-theory checker profile.
  - Evidence records checked theory-certificate format and hash for the verified
    property.
  - No solver yes/no output or AI output is read by the checker.
- Verification:
  - `cargo test -p mpk-api --test policy_strategy`
  - `cargo test -p mpk-api --test strategies theory_strategy_proves_max64_simple_vc_through_checked_certificate`
  - `cargo test -p mpk-theory linarith`
- Notes:
  - This milestone deliberately closes only one representative policy shape.
    Broader arithmetic coverage belongs to later milestones.

### POE-10 Implement `mpk policy verify` Orchestrator

- Status: Pending
- Depends on: POE-03, POE-04, POE-05, POE-07, POE-09
- Inputs:
  - `crates/mpk-cli/src/main.rs`
  - `go-tools/go2gir/main.go`
  - `crates/mpk-vc/src/lib.rs`
  - `crates/mpk-api/src/strategies.rs`
  - `crates/mpk-kernel/src/json_output.rs`
- Likely touched files:
  - `crates/mpk-cli/src/policy_verify.rs`
  - `crates/mpk-cli/src/policy_scan.rs`
  - `crates/mpk-cli/src/policy_evidence.rs`
  - `crates/mpk-cli/tests/policy_verify.rs`
- Deliverables:
  - `mpk policy verify` implementation that runs scan, GIR import, VC
    generation, selected strategy attempts, checker verification for trusted
    evidence, evidence JSON writing, and Markdown report writing.
  - Artifact layout under a caller-provided output directory or explicit output
    paths.
  - Reproduction commands embedded in evidence JSON.
- Acceptance criteria:
  - Verify on the reserve example writes evidence JSON and Markdown report.
  - Verified properties are marked `mpk_verified` only when backed by checked
    declaration or checked theory evidence.
  - Proof-pending properties are listed and do not make the whole run fail unless
    the caller passes a strict flag.
  - Unsupported scan or unsupported property exits non-zero with deterministic
    JSON and no partial trusted-evidence claim.
  - `mpk policy verify` does not mutate checked-in fixtures unless an explicit
    update flag is provided.
- Verification:
  - `(cd go-tools/go2gir && go build -o ../../target/debug/go2gir .)`
  - `cargo test -p mpk-cli --test policy_verify`
  - `cargo run --quiet -p mpk-cli -- policy verify examples/payment_policies/reserve --function example.com/payment/reserve.ApprovedReserveCents --contract examples/payment_policies/reserve/policy_contract.json --strategy-profile payment-policy-alpha --checker-profile mvp-strict --evidence-json /tmp/mpk-evidence.json --evidence-md /tmp/mpk-evidence.md`
- Notes:
  - If certificate emission from generated VC declarations is not yet complete,
    the orchestrator may report checked theory evidence for supported properties
    and proof-pending status for the rest.

### POE-11 Add Call-Site Precondition Helper

- Status: Pending
- Depends on: POE-03, POE-04
- Inputs:
  - `docs/web-system-integration.md`
  - `examples/order_policy/webapp/handler.go`
  - `examples/order_policy/policy_contract.json`
- Likely touched files:
  - `crates/mpk-cli/src/policy_callsite.rs`
  - `crates/mpk-cli/tests/policy_callsite.rs`
  - `examples/payment_policies/reserve/webapp`
- Deliverables:
  - Helper analysis that reports precondition status for visible call sites:
    `checked_by_local_guard`, `declared_upstream_invariant`, `not_observed`, or
    `unsupported_control_flow`.
  - Evidence JSON integration under `helper_artifacts.call_site_preconditions`.
- Acceptance criteria:
  - The order-policy webapp reports `requestedCents >= 0` as
    `checked_by_local_guard`.
  - The wallet balance precondition can be represented as
    `declared_upstream_invariant` only when the caller provides an explicit
    invariant annotation or config.
  - Missing checks report `not_observed`.
  - Loops, aliasing, or unsupported control flow report
    `unsupported_control_flow`.
  - Output is clearly labeled helper analysis and never proof evidence.
- Verification:
  - `cargo test -p mpk-cli --test policy_callsite`
  - `(cd examples/order_policy/webapp && go test -count=1 ./...)`
- Notes:
  - Keep this helper conservative. It is a lint for product reports, not a
    verified frontend.

### POE-12 Add Policy CI Examples

- Status: Pending
- Depends on: POE-10
- Inputs:
  - `docs/web-system-integration.md`
  - `scripts/check-all.sh`
  - `examples/payment_policies`
- Likely touched files:
  - `docs/proof-ops-policy-ci.md`
  - `examples/payment_policies/README.md`
  - `.github/workflows` if this repository already uses workflow files
- Deliverables:
  - CI documentation showing how to build `go2gir`, run `mpk policy scan`, run
    `mpk policy verify`, and fail on unexpected artifact drift.
  - Example command block suitable for customer repositories.
  - Trust-boundary warning that CI success does not replace checker evidence.
- Acceptance criteria:
  - CI example uses real command names and paths from POE-10.
  - CI example separates helper-artifact drift checks from trusted-evidence
    checks.
  - Documentation states which artifacts should be reviewed in PRs.
- Verification:
  - `rg -n "mpk policy scan|mpk policy verify|GIR|proof evidence" docs/proof-ops-policy-ci.md examples/payment_policies/README.md`
  - `git diff --check`
- Notes:
  - Do not add a GitHub workflow unless the repository already has a workflow
    convention or the implementation milestone explicitly chooses one.

### POE-13 Update Release And Alpha Checks

- Status: Pending
- Depends on: POE-06, POE-10
- Inputs:
  - `scripts/check-all.sh`
  - `docs/alpha-demo.md`
  - `alpha-release-report`
- Likely touched files:
  - `scripts/check-all.sh`
  - `docs/alpha-demo.md`
  - `alpha-release-report`
- Deliverables:
  - Lightweight policy scan and verify checks added to the local full-check path
    when runtime is acceptable.
  - Alpha demo section for the ProofOps engine path.
  - Release report wording that keeps policy evidence separate from package
    release evidence.
- Acceptance criteria:
  - `scripts/check-all.sh` runs policy example tests or explicitly documents why
    they remain outside the full suite.
  - `docs/alpha-demo.md` includes reproduction commands for policy scan and the
    first supported policy verify path.
  - The release report does not claim generated GIR/VC artifacts are proof
    evidence.
- Verification:
  - `./scripts/check-all.sh`
  - `cargo test -p mpk-cli --test policy_verify`
  - `rg -n "policy scan|policy verify|proof evidence|helper artifacts" docs/alpha-demo.md alpha-release-report`
- Notes:
  - If `./scripts/check-all.sh` becomes too slow, add targeted commands to
    `docs/alpha-demo.md` and document the reason in this milestone's final
    implementation note.

### POE-14 Harden Failure Modes And Determinism

- Status: Pending
- Depends on: POE-10, POE-11
- Inputs:
  - `develop/tasks/definition_of_done.md`
  - `develop/roadmap/RELEASE_GATES.md`
  - `fuzz/README.md`
- Likely touched files:
  - `crates/mpk-cli/tests/policy_scan.rs`
  - `crates/mpk-cli/tests/policy_verify.rs`
  - `crates/mpk-cli/tests/policy_callsite.rs`
  - `fuzz/fuzz_targets` if new parser fuzzing is justified
- Deliverables:
  - Negative tests for malformed scan/evidence JSON.
  - Negative tests for non-canonical or missing artifact paths.
  - Determinism tests for repeated scan and verify output.
  - Path traversal rejection for output and input paths where MPK normalizes
    product artifacts.
- Acceptance criteria:
  - Repeated policy scan on the same fixture produces byte-identical JSON.
  - Repeated policy verify on the same fixture produces byte-identical evidence
    JSON, except for explicitly excluded local absolute paths.
  - Malformed contracts, unknown strategy profiles, unknown checker profiles,
    invalid output paths, and unsupported Go features reject deterministically.
  - No panic occurs for malformed product-facing JSON fixtures.
- Verification:
  - `cargo test -p mpk-cli --test policy_scan`
  - `cargo test -p mpk-cli --test policy_verify`
  - `cargo test -p mpk-cli --test policy_callsite`
  - `cargo test --workspace`
- Notes:
  - Keep local absolute paths out of canonical hashes and stable report fields.

### POE-15 Final Documentation Review And Handoff

- Status: Pending
- Depends on: POE-12, POE-13, POE-14
- Inputs:
  - `docs/proof-ops-engine-design.md`
  - `docs/proof-ops-engine-design-todo.md`
  - `docs/web-system-integration.md`
  - `docs/alpha-demo.md`
  - `README.md`
- Likely touched files:
  - `README.md`
  - `docs/proof-ops-engine-design.md`
  - `docs/proof-ops-engine-design-todo.md`
  - `docs/web-system-integration.md`
  - `docs/alpha-demo.md`
- Deliverables:
  - Documentation updated to match implemented command names, schema fields,
    profile names, and evidence statuses.
  - Handoff summary for the ProofOps repository describing what it can consume
    from MPK.
- Acceptance criteria:
  - No implementation or user-facing docs use the obsolete policy-profile flag
    for the ProofOps engine path.
  - `strategy_profile`, `checker_profile`, and axiom policy profile terminology
    remains distinct.
  - Every customer-facing claim path identifies whether it is trusted evidence
    or helper analysis.
  - README links to both alpha demo and ProofOps engine docs.
- Verification:
  - `git diff --check`
  - Run this stale-term check:

```sh
python3 - <<'PY'
from pathlib import Path
needles = ["--" + "policy" + "-profile", "proof " + "profile", "Proof " + "Profile"]
paths = [Path("README.md"), Path("docs"), Path("develop/specs"), Path("develop/roadmap")]
hits = []
for root in paths:
    files = [root] if root.is_file() else sorted(root.rglob("*.md"))
    for path in files:
        text = path.read_text()
        for needle in needles:
            if needle in text:
                hits.append(f"{path}: {needle}")
if hits:
    raise SystemExit("\n".join(hits))
PY
```

  - `rg -n "strategy_profile|checker_profile|proof evidence|helper analysis" docs/proof-ops-engine-design.md docs/proof-ops-engine-design-todo.md docs/alpha-demo.md`
- Notes:
  - This milestone is documentation-only unless implementation changed public
    behavior that needs fixture updates.

## Review Ledger

Resolved findings:

- The source design used `--checker-profile mvp-structural` in the policy verify
  example, but checked theory strategies require `mvp-strict` in the current
  API and certificate profiles. The source design was updated to use
  `mvp-strict` for checked-theory policy verification.

Remaining findings after self-review:

- None.

Review checks applied:

- Every requirement in `docs/proof-ops-engine-design.md` maps to at least one
  milestone.
- Each milestone has dependencies, likely touched files, deliverables,
  acceptance criteria, and verification commands.
- Trust-boundary-sensitive terms distinguish strategy profile, checker profile,
  and axiom policy profile.
- No milestone asks ProofOps product code to be implemented in MPK.
