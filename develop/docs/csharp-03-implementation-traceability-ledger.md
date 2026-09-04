# CSHARP-03 Implementation Traceability Ledger

Status: `CSHARP-03-T01-W01/W02/W03/W04/W05/W06/W07/W08/W09` complete (2026-09-04). The
entry audit, consumer inventory, private frontend/toolchain closure proof,
Roslyn shape probes, primitive/string/numeric/codec runtime measurements and
candidate foundation/specialization/binding/data semantics and the successor
contract/boundary/transition/identity/limit freeze have historical completion
records. The authorized W08 expansion amendment resolves
`CSHARP-03-T01-W09-F01` without changing core; W09 then measures checker
capacity and completes the private freeze. `CSHARP-03-T01-W10` is ready and all
implementation items remain serially blocked. No production acceptance path,
installed candidate, or active registry entry was introduced by W01-W09.

This ledger is subordinate to
[`08_csharp_practical_subset_design.md`](08_csharp_practical_subset_design.md)
and
[`08_csharp_practical_subset_design-todo.md`](08_csharp_practical_subset_design-todo.md).
The design and task document remain authoritative for semantics, scope,
dependencies, and exit gates. This file records execution state and evidence;
it does not freeze a new profile or alter an active release.

## 1. Ledger rules

- Work items execute in the serial order defined by the task document. At most
  one row may be `Ready`; a stop finding leaves none ready. No two rows may be
  `In progress`.
- `Primary test owner` is the exact planned path plus a full work-item prefix.
  A later work item must create or extend that owner without taking ownership
  from another row.
- `SELF` means the commit containing this ledger row. A document cannot contain
  its own Git commit hash without changing that hash; completed later rows must
  record their literal immutable commit hash.
- `—` means that a blocked or ready item has no implementation commit yet. It
  does not mean not applicable.
- A status may be only `Blocked`, `Ready`, `In progress`, or `Complete`.
  `Complete` requires the item-local handoff record, verification, zero-finding
  review, commit, and push.

## 2. Work-item status and primary ownership

<!-- work-item-ledger:start -->
| Work item | Status | Primary test owner | Commit |
| --- | --- | --- | --- |
| `CSHARP-03-T01-W01` | `Complete` | `crates/mpk-vc/tests/csharp_practical_inventory.rs#CSHARP-03-T01-W01` | `17275ffcba4f37d93a74fd188d9860b0a7d5f10d` |
| `CSHARP-03-T01-W02` | `Complete` | `crates/mpk-vc/tests/csharp_practical_inventory.rs#CSHARP-03-T01-W02` | `f84a5c6ff5122a3a5e64d9305fe999ed1f501f85` |
| `CSHARP-03-T01-W03` | `Complete` | `crates/mpk-cli/tests/csharp_practical_build_inputs.rs#CSHARP-03-T01-W03` | `4ad2cd480792d8e7cac71eb798e6b55b66bd97fb` |
| `CSHARP-03-T01-W04` | `Complete` | `crates/mpk-cli/tests/csharp_practical_probes.rs#CSHARP-03-T01-W04` | `b6680168c2666be503741575c009f0a26dd0da22` |
| `CSHARP-03-T01-W05` | `Complete` | `crates/mpk-cli/tests/csharp_practical_probes.rs#CSHARP-03-T01-W05` | `13415911853c0368c103bd9d5feeb8374596d724` |
| `CSHARP-03-T01-W06` | `Complete` | `crates/mpk-cli/tests/csharp_practical_probes.rs#CSHARP-03-T01-W06` | `22673dbc96d8ba4f0d9a4cb97c3f2490c00d1804` |
| `CSHARP-03-T01-W07` | `Complete` | `crates/mpk-cli/tests/csharp_practical_probes.rs#CSHARP-03-T01-W07` | `b0ff7daec663b95b1f88ecc1d98f0b7c1f6fdf00` |
| `CSHARP-03-T01-W08` | `Complete` | `crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W08` | `4ffd8b3a9918b6cae9e4d4704e4bc6b09a12cd5c` |
| `CSHARP-03-T01-W09` | `Complete` | `crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W09` | `SELF` |
| `CSHARP-03-T01-W10` | `Ready` | `crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W10` | `—` |
| `CSHARP-03-T02-W01` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_registry.rs#CSHARP-03-T02-W01` | `—` |
| `CSHARP-03-T02-W02` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_vir_model.rs#CSHARP-03-T02-W02` | `—` |
| `CSHARP-03-T02-W03` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_vir_model.rs#CSHARP-03-T02-W03` | `—` |
| `CSHARP-03-T02-W04` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_source_artifacts.rs#CSHARP-03-T02-W04` | `—` |
| `CSHARP-03-T02-W05` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_vir_validation.rs#CSHARP-03-T02-W05` | `—` |
| `CSHARP-03-T02-W06` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_vc_model.rs#CSHARP-03-T02-W06` | `—` |
| `CSHARP-03-T02-W07` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_frontend_protocol.rs#CSHARP-03-T02-W07` | `—` |
| `CSHARP-03-T02-W08` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_migration.rs#CSHARP-03-T02-W08` | `—` |
| `CSHARP-03-T02-W09` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_migration.rs#CSHARP-03-T02-W09` | `—` |
| `CSHARP-03-T03-W01` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_capture.rs#CSHARP-03-T03-W01` | `—` |
| `CSHARP-03-T03-W02` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_syntax.rs#CSHARP-03-T03-W02` | `—` |
| `CSHARP-03-T03-W03` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W03` | `—` |
| `CSHARP-03-T03-W04` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W04` | `—` |
| `CSHARP-03-T03-W05` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W05` | `—` |
| `CSHARP-03-T03-W06` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W06` | `—` |
| `CSHARP-03-T03-W07` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_collections.rs#CSHARP-03-T03-W07` | `—` |
| `CSHARP-03-T03-W08` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_collections.rs#CSHARP-03-T03-W08` | `—` |
| `CSHARP-03-T03-W09` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_collections.rs#CSHARP-03-T03-W09` | `—` |
| `CSHARP-03-T03-W10` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_codecs.rs#CSHARP-03-T03-W10` | `—` |
| `CSHARP-03-T03-W11` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_numbers.rs#CSHARP-03-T03-W11` | `—` |
| `CSHARP-03-T03-W12` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_domain.rs#CSHARP-03-T03-W12` | `—` |
| `CSHARP-03-T03-W13` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_domain.rs#CSHARP-03-T03-W13` | `—` |
| `CSHARP-03-T03-W14` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_domain.rs#CSHARP-03-T03-W14` | `—` |
| `CSHARP-03-T04-W01` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_control.rs#CSHARP-03-T04-W01` | `—` |
| `CSHARP-03-T04-W02` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_control.rs#CSHARP-03-T04-W02` | `—` |
| `CSHARP-03-T04-W03` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_control.rs#CSHARP-03-T04-W03` | `—` |
| `CSHARP-03-T04-W04` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_control.rs#CSHARP-03-T04-W04` | `—` |
| `CSHARP-03-T04-W05` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_control.rs#CSHARP-03-T04-W05` | `—` |
| `CSHARP-03-T04-W06` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_control.rs#CSHARP-03-T04-W06` | `—` |
| `CSHARP-03-T05-W01` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_boundary.rs#CSHARP-03-T05-W01` | `—` |
| `CSHARP-03-T05-W02` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_boundary.rs#CSHARP-03-T05-W02` | `—` |
| `CSHARP-03-T05-W03` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_boundary.rs#CSHARP-03-T05-W03` | `—` |
| `CSHARP-03-T05-W04` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_transition.rs#CSHARP-03-T05-W04` | `—` |
| `CSHARP-03-T05-W05` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_transition.rs#CSHARP-03-T05-W05` | `—` |
| `CSHARP-03-T05-W06` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_boundary_transition.rs#CSHARP-03-T05-W06` | `—` |
| `CSHARP-03-T06-W01` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_vc.rs#CSHARP-03-T06-W01` | `—` |
| `CSHARP-03-T06-W02` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_vc.rs#CSHARP-03-T06-W02` | `—` |
| `CSHARP-03-T06-W03` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_vc.rs#CSHARP-03-T06-W03` | `—` |
| `CSHARP-03-T06-W04` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_vc.rs#CSHARP-03-T06-W04` | `—` |
| `CSHARP-03-T06-W05` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_vc.rs#CSHARP-03-T06-W05` | `—` |
| `CSHARP-03-T06-W06` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_vc.rs#CSHARP-03-T06-W06` | `—` |
| `CSHARP-03-T06-W07` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_vc.rs#CSHARP-03-T06-W07` | `—` |
| `CSHARP-03-T06-W08` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_vc.rs#CSHARP-03-T06-W08` | `—` |
| `CSHARP-03-T06-W09` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_vc.rs#CSHARP-03-T06-W09` | `—` |
| `CSHARP-03-T06-W10` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_policy_verify.rs#CSHARP-03-T06-W10` | `—` |
| `CSHARP-03-T06-W11` | `Blocked` | `crates/mpk-api/tests/csharp_practical_api.rs#CSHARP-03-T06-W11` | `—` |
| `CSHARP-03-T06-W12` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_end_to_end.rs#CSHARP-03-T06-W12` | `—` |
| `CSHARP-03-T07-W01` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_build_inputs.rs#CSHARP-03-T07-W01` | `—` |
| `CSHARP-03-T07-W02` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_release_bundle.rs#CSHARP-03-T07-W02` | `—` |
| `CSHARP-03-T07-W03` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_frontend_runner.rs#CSHARP-03-T07-W03` | `—` |
| `CSHARP-03-T07-W04` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_frontend_runner.rs#CSHARP-03-T07-W04` | `—` |
| `CSHARP-03-T07-W05` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_release_gate.rs#CSHARP-03-T07-W05` | `—` |
| `CSHARP-03-T07-W06` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_release_gate.rs#CSHARP-03-T07-W06` | `—` |
| `CSHARP-03-T08-W01` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_fixture.rs#CSHARP-03-T08-W01` | `—` |
| `CSHARP-03-T08-W02` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_examples.rs#CSHARP-03-T08-W02` | `—` |
| `CSHARP-03-T08-W03` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_examples.rs#CSHARP-03-T08-W03` | `—` |
| `CSHARP-03-T08-W04` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_examples.rs#CSHARP-03-T08-W04` | `—` |
| `CSHARP-03-T08-W05` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_examples.rs#CSHARP-03-T08-W05` | `—` |
| `CSHARP-03-T08-W06` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_release_gate.rs#CSHARP-03-T08-W06` | `—` |
| `CSHARP-03-T08-W07` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_release_gate.rs#CSHARP-03-T08-W07` | `—` |
| `CSHARP-03-T08-W08` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_release_gate.rs#CSHARP-03-T08-W08` | `—` |
| `CSHARP-03-T08-W09` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_release_gate.rs#CSHARP-03-T08-W09` | `—` |
| `CSHARP-03-T08-W10` | `Blocked` | `crates/mpk-cli/tests/successor_atomic_cutover.rs#CSHARP-03-T08-W10` | `—` |
<!-- work-item-ledger:end -->

## 3. CSHARP-03-T01-W01 completion record

### 3.1 Inputs and retained predecessor evidence

- Entry snapshot: clean `main` commit
  `4d593f56f8750d151d9fe42627a84e9e4842d1cc`, tree
  `43164f9a70793b32743df90a39d99f289c481504`.
- `JAVA-03-T10`: commit
  `b7102c1acfcacdbf45b3d5a3ef21aac1ccf56f64`, tree
  `e139c6f9793929d68997fd40909f74f25e3ace53`, and accepted receipt
  `develop/migrations/archive/java-03-t10-native-receipt.json` with raw-byte
  SHA-256
  `712261fc353e1f84e70eecd8f0db690f5b9f7e364eef32b8f5afc5446fdd1de8`
  and Git blob SHA-1 `de38839dd599d57425f23234ba512660c6b160b9`.
- Entry design SHA-256:
  `b1b2b7fc4a89c0f95081f55b64ddd78c25cbc654c28bfa7627bfa597d6dd6e94`;
  entry task-plan SHA-256:
  `f3f192632b1a1dbcd8f37ab7c087f261f1eee8ceb007f6a470c24938a23f77f5`.
- Existing scalar C# profile/vector remain unchanged. Their SHA-256 values are
  `39b6f6b6da4fe071af42056bb8296b330566568e86c00dac3f8dc91bcc380ba4`
  and
  `8109f781ca1f2b90ba02f786da09ba97602f4cd484b8835b561d5ecf4e7781c8`.
  The semantic-registry specification and v1 vector remain unchanged at
  `e9770f4cfb955bb712791b89012b82ee065404f2fa5907cf75a07c573b59a4fe`
  and
  `f7007417279f5173d0102ec2833095f2d97f271e1cdf2622d381d31e6ab86ae7`.

The machine-readable baseline at
`develop/migrations/csharp-03/baseline.json` binds the active semantic and
bundle registries, all five tuples, four frontend and four toolchain bundle
identities, four candidate projections, both checker identities and agreement
hashes, all-zero axiom inventory, exact four-language corpus inputs, and the
composed release command. Its schema test recomputes 21 recorded file hashes
from the checked-in image and proves that the release report reproduces the
retained Java receipt.

### 3.2 Bounded outputs and exclusions

W01 adds exactly these files:

- `develop/migrations/csharp-03/baseline.json`;
- `develop/docs/csharp-03-implementation-traceability-ledger.md`; and
- `crates/mpk-vc/tests/csharp_practical_inventory.rs`.

It updates current status only in `README.md`,
`develop/docs/00_executive_summary.md`,
`develop/docs/06_multilanguage_frontend_design.md`,
`develop/docs/06_multilanguage_frontend_design-todo.md`,
`develop/docs/08_csharp_practical_subset_design.md`, and
`develop/docs/08_csharp_practical_subset_design-todo.md`.

Production source, normative specifications, vectors, canonical artifacts and
hashes, release descriptors, registries, candidates, fixtures, examples, and
public routes are unchanged. Source-design semantic rows, diagnostics, limits,
accepted/rejected source cases, MPK-free application inspection, generic
closure, foundation specialization, and application semantic bindings are
`not_applicable` at W01 under section 4 of the task document; their first
applicable owners are T01-W02 through T01-W10 and their routed downstream
production items.

At W01 completion, the active semantic and bundle registries contained neither
`CSHARP-03` nor `mpk.csharp.practical`. No identity therefore existed for an
active, public, staging, or ambient practical-profile route; W02 was the sole
ready item and all other rows remained blocked.

### 3.3 Verification and review

Target host: Darwin arm64. The following local, non-native checks passed from
the recorded active image:

- `cargo test -p mpk-vc --test csharp_practical_inventory`;
- `cargo test --workspace`;
- `cargo test -p mpk-cli --test successor_atomic_cutover`;
- `cargo test -p mpk-cli --test java_activation`;
- `cargo test -p mpk-cli --test csharp_policy_verify`;
- `cargo test -p mpk-vc --test go_vir_corpus`;
- `(cd go-tools/go2vir && go test -count=1 ./...)`;
- `./scripts/check-release-bundles.sh --fixture successor`;
- `python3 scripts/check-artifact-paths.py`;
- `python3 scripts/check-spec-vectors.py --check`;
- `python3 scripts/generate-release-report.py --check`; and
- `./scripts/check-fast.sh`.

Native x86-64 Linux gate: not rerun on the Darwin arm64 work host because
`sudo ./scripts/check-java-frontend.sh` and the delegating
`sudo ./scripts/check-all.sh` require native x86-64 Linux, root, strace, and a
writable cgroup-v2 hierarchy. For the same host reason,
`./scripts/check-reference.sh` passed its Go self-tests and Rust-CLI agreement
test but then rejected the final package-verification step before attempting
the embedded Linux x86-64 checker. W01 retains and independently rechecks the
accepted JAVA-03-T10 native receipt, embedded checker hash, checker agreement,
and zero-axiom report instead; no new native behavior or release bytes were
introduced.

Review/fix history:

- The first pass found a truncated export hash in `baseline.json`; it was
  replaced with the 64-character value independently present in the release
  report and both checker results, and SHA-256 shape validation was added.
- The second pass found that the draft had called `check-reference.sh` fully
  passed even though its final Linux-only executable step was not runnable on
  Darwin; the result and explicit unrun reason above now match the observed
  command, and checker identity checks were strengthened.
- The third pass strengthened entry-snapshot and duplicate-plan-item checks.

Final review findings: `0`. The final review checks scope, baseline
recomputation, ledger completeness/uniqueness, plan-to-owner routing, serial
statuses, absence of active practical-profile identity, and the explicit
native-gate disposition after all fixes.

## 4. CSHARP-03-T01-W02 completion record

### 4.1 Closed artifact and consumer inventory

The machine-readable inventory at
`develop/migrations/csharp-03/artifact-consumer-inventory.json` is bound to the
W01 commit `17275ffcba4f37d93a74fd188d9860b0a7d5f10d` and tree
`957b38264b0e149fa6050b0c5d692ee4b1761001`. It closes all 17 required identity
families: semantic registry, context, parameters, selection, profile
contracts, source-artifact graph, foundation, VIR, frontend protocol, source
map, source manifest, VC/skeleton, release, policy/evidence, program assembly,
AI, and API.

The inventory records 67 explicit producer/parser/validator/serializer/hash,
bundle, CLI/API, fixture, and test edges. Repository search fixtures: `136`;
their exact search roots, exclusions, counts, and sorted-path-set SHA-256 values
bind 4,922 family-to-path consumer hits. Every hit resolves through an ordered
path rule to either `active` or `private` and to a concrete edge role. The
closed ledger contains no unowned read/write/hash edge, unclassified route, or
unassigned cutover/rollback member.

Current names remain observations rather than a new specification freeze. In
particular, the working practical foundation and successor artifact names are
not installed or accepted by W02; final identity and hash-domain ownership
remains with `CSHARP-03-T01-W09`. No production source, normative vector,
active registry/release descriptor, candidate, fixture, CLI/API route, or
acceptance behavior changed.

### 4.2 Atomic migration and rollback boundary

`csharp-practical-successor-whole-release` is the only migration set. It
includes every family above, all four predecessor producers, all shared
consumers, all five retained release tuples, foundation bytes and descriptor,
registries, bundles, hashes, routes, fixtures, reports, gates, and
documentation. Producer migration remains owned by `CSHARP-03-T02-W08`,
consumer closure by `CSHARP-03-T02-W09`, and public activation by
`CSHARP-03-T08-W10`; no public old/new selector or partial tuple migration is
permitted.

`csharp-practical-pre-cutover-installed-image` is the only rollback set. It is
bound to the same W01 commit/tree and baseline receipt. Rollback replaces the
whole installed image, including binaries, checker bytes, frontend/toolchain
bundles, both registry roots, candidates/receipts, profile contracts,
fixtures, reports, and documentation. Per-file, per-registry, per-route, and
per-language rollback are explicitly forbidden.

### 4.3 Verification and review

Target host: Darwin arm64. The following local checks passed:

- `cargo test -p mpk-vc --test csharp_practical_inventory`;
- `cargo fmt --all -- --check`;
- `cargo clippy -p mpk-vc --test csharp_practical_inventory -- -D warnings`;
- `python3 scripts/check-artifact-paths.py`;
- `python3 scripts/check-spec-vectors.py --check`; and
- `./scripts/check-fast.sh`.

The W02 owner test recomputes all 136 repository searches and proves the stored
path-set fingerprints reject both deletion of an observed consumer and
addition of a synthetic consumer. It also checks exact family/edge/route
schemas, every live bundle member through per-bundle counts and path-set
SHA-256 fingerprints, five tuple keys, all eight public and two internal CLI
routes, all 33 successor API routes, planned downstream owners, the single
atomic set, and the whole-image rollback set.

Review/fix history:

- The first implementation pass found an unowned active consumer in
  `crates/mpk-cli/build.rs`; the route rules now include crate build inputs,
  and the full search was recomputed.
- The second pass removed `bin` from ignored directory names because
  `rust-tools/rust2vir/src/bin` is production source rather than generated
  output, and added private classifications for tool `tests`, `testdata`, and
  fuzz paths.
- The third pass audited every mandatory starting point and found that the
  shared v0 VIR/map/manifest/VC substrate, C# capture-through-lowering stages,
  runtime/program encoders, and the C# libc-compat member were represented too
  coarsely. It added 23 explicit edges, 19 repository searches, and exact
  per-bundle inventory-file counts and path-set fingerprints without activating
  any successor identity.
- The fourth pass found that broad identity/hash families, all per-bundle paths,
  and surplus API routes were not independently mutation-sensitive. It added 18
  identity/domain searches, fingerprints every bundle member path, compares the
  complete API route table, and runs add/delete mutations for every search.
- The fifth pass compared every C# frontend source with the search union and
  found source transport, emission-model, private CLI-parser, and executable
  entrypoint ownership under-specified. Four explicit edges and four searches
  now bind those read/write and CLI-route consumers.
- The sixth pass found the top-level machine/human release reports and shipped
  reference-checker binary were only indirectly reachable through the W01
  baseline. It now inventories both report edges and search roots, binds the
  checker bytes, and requires every exact identity/hash token to have a
  family-owned search.
- The seventh pass found the tracked C# policy fixture incorrectly routed to
  the private shared-consumer stage. Its replacement owner is now T08-W10, as
  required by the plan; T08-W01 remains the owner of a separate private staged
  replacement and does not mutate the tracked fixture path.
- The eighth pass found that Certificate v0 hash ownership was described as one
  umbrella instead of enumerated. All nine unchanged domains and the shared
  `mpk-cert` dispatch edge now have exact search ownership.
- The ninth pass found that active checked-in examples and the top-level private
  fuzz harness were absent from the search roots, and that predecessor registry
  identities and build/driver hash domains were represented only indirectly.
  Both roots are now route-classified, every installed profile/parameter/
  selection/contract identity is enumerated from the active registry, and all
  affected current and retained-predecessor domains have exact searches.
- The tenth pass found a W01 completion paragraph whose present-tense wording
  appeared to leave W02 ready after W02 completion. It now explicitly records
  that status as the historical W01 state.
- The eleventh pass found that every search fixture independently rebuilt the
  same repository file list. The owner tests now take one immutable search
  snapshot per test and reuse it for all fingerprints and add/delete mutations,
  preserving coverage while avoiding fixture-count-scaled filesystem walks.
- The twelfth pass found that the separate CLI route table named only the three
  successor policy/explanation commands. It now closes the five existing
  certificate/package routes and both internal frontend routes as well, and
  checks the production dispatchers for surplus or missing arms.

Final review findings: `0`. The final review checks W02-only scope, complete
family and role coverage, live-path anchors, search-set mutation sensitivity,
active/private separation, downstream ownership, no practical activation,
and atomic cutover/whole-image rollback closure after all fixes.

## 5. CSHARP-03-T01-W03 completion record

### 5.1 Frozen frontend and toolchain closure

The private descriptor at
`develop/migrations/csharp-03/build-inputs/build-inputs.json` is bound to the
completed W02 commit `f84a5c6ff5122a3a5e64d9305fe999ed1f501f85`, tree
`c14885505d0eeb6901aa077dd6f497b2fc0a4d5d`, and W02 inventory raw SHA-256
`6b5b7f601f6174d61496424084d264604a5a3325a460a5c0640bfcd71a564c49`.
Its own raw SHA-256 is
`83bf64dcbedce89f79613fe7aab3d95a92179122df54f9b5407273a245738015`.

The descriptor closes these measured inputs:

- .NET SDK `10.0.400`, .NET runtime/reference pack `10.0.11`, Roslyn source
  commit `c0573ed0a7dc3e3b4d2e70da47f97cc51a35524f`, Roslyn runtime packages
  `5.6.0`, and analyzer package `5.3.0` as build metadata only;
- six exact upstream archives with size and SHA-256, including SHA-512 for the
  SDK and runtime tarballs; all six cached archive files are regular mode
  `0444` files;
- 167 exact `net10.0` reference assemblies totaling 6,046,008 bytes under
  `MPK-CSHARP-REFERENCE-INVENTORY-0.1`, with inventory SHA-256
  `30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad`;
- the complete toolchain preimage under
  `MPK-CSHARP-TOOLCHAIN-INPUTS-0.1`, with SHA-256
  `d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f`;
- 34 exact source/build-metadata files, 13 notice projections, direct
  `csc.dll` invocation with all 16 ordered flags, and 20 exact build-process
  environment variables; and
- bounded offline tar/ZIP extraction, canonical modes, path/case/symlink
  rejection, an empty temporary home/package cache, and no network after
  cached-archive validation.

Project evaluation, restore, package-cache discovery, source generators,
compile-time analyzers, ambient references, and unlisted source/reference
selection are forbidden. The wrapper starts from an empty environment and
passes only its three launcher variables; the compiler process starts from an
empty environment and receives only the descriptor's declared closure.

### 5.2 Deterministic private candidate and non-activation

The private inventory at
`develop/migrations/csharp-03/build-inputs/candidate-inventory.json` has raw
SHA-256
`ff4b48790c67135144419c816149f8edfbd7b40ade231d6ab44c8433efef0cce`.
Two clean builds from the frozen bytes produced the same 18-file candidate:
five frontend/runtime files and 13 notice files, all mode `0644`. Its candidate
file-inventory SHA-256 is
`e02a1d95f8c7f9fe576de16575b6c1247bebca0f678f8ddfc26ead3ad64a395f`.
Candidate deterministic USTAR SHA-256:
`a26bc0ad42ed424812caf25b5b8d73df95e2ccefaa0442282ecb8399c440a302`;
its size is 10,516,480 bytes and its canonical header layout fixes USTAR,
mode, UID/GID, owner/group, and timestamp values.

Private mutation checks: `12`. They reject archive/reference/candidate byte,
project/candidate/notice count, cache/candidate mode, compiler flag, declared
environment, and restore-policy changes before publication. The isolated
two-build run additionally injects hostile home, SDK, package-cache, proxy,
and unlisted variables; the empty-environment wrapper ignores them and still
reproduces the recorded candidate bytes.

The private descriptor, inventory, harness, wrapper, and primary tests are the
only new W03 surfaces. They do not register a practical candidate or change
production acceptance. The active scalar descriptor, active scalar candidate
inventory, and active scalar vector remain byte-identical at raw SHA-256
`0345044d16d4efb3568c32a3d7bc67fec508fe9359eff423a7f09c7f69b348dc`,
`4ff3ba6fdc2eb2857c32563b959f11194075a4264164cd7aebc808858e500e9b`,
and `8109f781ca1f2b90ba02f786da09ba97602f4cd484b8835b561d5ecf4e7781c8`.
No active registry, release descriptor, production source, normative profile,
or public route changed.

W03's primary test is
`crates/mpk-cli/tests/csharp_practical_build_inputs.rs#CSHARP-03-T01-W03`.
The W02 consumer inventory remains live: its 136 searches now bind 4,922 hits,
including the new primary test's two private hash-domain uses.

### 5.3 Verification and review

Target host: Darwin arm64. The following local checks passed:

- `cargo test -p mpk-cli --test csharp_practical_build_inputs`;
- `cargo test -p mpk-vc --test csharp_practical_inventory`;
- `cargo fmt --all -- --check`;
- `cargo clippy -p mpk-cli --test csharp_practical_build_inputs -- -D warnings`;
- `./scripts/build-csharp-practical-frontend.sh --self-test`;
- `./scripts/build-csharp-practical-frontend.sh --check-build-inputs`;
- `./scripts/build-csharp-frontend.sh --check-build-inputs`;
- `python3 scripts/check-artifact-paths.py`;
- `python3 scripts/check-spec-vectors.py --check`; and
- `./scripts/check-fast.sh`.

The exact two-clean-build recipe passed under host emulation in the existing
Linux x86-64 local gate image
`sha256:ea3189955dd9c0e5deda7a30ef48a0c7ef5af3b128f74fcd6e368384b8e1420a`,
with `--network none`, a read-only repository mount, a fresh executable tmpfs,
and hostile ambient variables:

```text
docker run --rm --platform linux/amd64 --privileged --network none --read-only --tmpfs /tmp:rw,nosuid,nodev,exec,size=4g -e MPK_CSHARP_PRACTICAL_UNLISTED_AMBIENT=hostile -e HOME=/ambient-home-must-not-be-used -e DOTNET_ROOT=/ambient-dotnet-must-not-be-used -e NUGET_PACKAGES=/ambient-nuget-must-not-be-used -e HTTP_PROXY=http://127.0.0.1:1 -e HTTPS_PROXY=http://127.0.0.1:1 --mount type=bind,source=<repository>,target=/workspace,readonly -w /workspace mpk-java-t10-gate:local ./scripts/build-csharp-practical-frontend.sh --check
```

This W03 evidence does not redefine the future practical release gate. Its
exact release command and relation to the current Java-owned aggregate gate
remain owned by T01-W10, while native practical execution remains T07/T08
scope. The installed Java native receipt remains predecessor evidence only.

Review/fix history:

- The first implementation pass found that a well-formed but different
  candidate archive hash survived inventory-shape validation. Exact descriptor,
  recipe, project, candidate, archive hashes and archive size are now checked,
  and all 12 mutation cases pass only when every altered value rejects.
- The second pass found that validating the historical W02 source manifest
  directly against the mutable future worktree would make W03 evidence fail
  after later implementation tasks. Historical manifest identity is now
  validated against its frozen aggregate hash, while both build-check actions
  separately validate the live files before compiling them.
- The third pass detected the two new hash-domain consumers through the W02
  search gate. Both path fingerprints and the aggregate hit count were updated
  in this same work item; all 136 add/delete-sensitive searches pass.
- The fourth pass found that the complete build and inventory-update path did
  not independently check live project bytes before compiling and could write
  a generated inventory before checking all frozen aggregate values. Live
  files are now checked before compilation and generated inventory is fully
  validated before either comparison or atomic replacement.
- The fifth pass found one remaining historical-manifest dependency on the
  active scalar script's mutable project-file list. The historical validator
  now relies only on its exact 34-record aggregate, while the build paths retain
  their separate live-manifest validation.

Final review findings: `0`. The final review checks W03-only scope, frozen
byte/count/mode/flag/reference/environment closure, offline two-build
reproducibility, ambient-state stripping, private registration state, active
scalar byte preservation, W02 consumer closure, serial ledger state, and the
T01-W10/T07/T08 native-gate ownership boundary after all fixes.

## 6. CSHARP-03-T01-W04 completion record

### 6.1 Pinned compiler observations and upgrade mutations

W04 consumes only the completed W03 commit
`4ad2cd480792d8e7cac71eb798e6b55b66bd97fb`, tree
`3ab99588482bfb3666088fa88dede679c748c17c`, private build-input descriptor
raw SHA-256
`83bf64dcbedce89f79613fe7aab3d95a92179122df54f9b5407273a245738015`,
private candidate-inventory raw SHA-256
`ff4b48790c67135144419c816149f8edfbd7b40ade231d6ab44c8433efef0cce`,
toolchain SHA-256
`d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f`,
and reference-projection SHA-256
`30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad`.
The disposable probe source raw SHA-256 is
`e49a96c63ef1dc8548d54b5ad5cb6dd81ebb90b56fa7a27d54adfcb99c1d4657`.

The canonical result at
`develop/migrations/csharp-03/probes/roslyn-data-construction.json` is
5,925,271 bytes with raw SHA-256
`c5de8bc209331c2295497210a570ba0be32e0871b3dd2576980d6c109222142e`.
Its normalized raw-observation section is 5,870,363 bytes with SHA-256
`f264897e932272135510f8294eaccb3e42a9f93445392cbded8d18917821db93`;
the deterministic probe binary SHA-256 is
`0fd5d16ebfbebed44377301b64fad8366e6fe9d04f3ababbd4406a6a0a100ca5`.
Both clean reruns agree on all three values.

Fourteen isolated compilation units record 181 distinct target shapes: 129
proposed admitted shapes and 52 rejected near misses. The sorted whole,
admitted, and rejected shape-ID sets have SHA-256 values
`727b7203815631d83cdb8475a2ce8360061205318763ed36a09fce76628a57b2`,
`fe3a7b166ac51e184249debc491532b71fa30a9d1a5723cc830da67a8792ff6e`,
and
`506ba206622d81aa61b5ee8973958fc2c68a4155cf64d047e0daec4bcc9fd346`.
Every admitted target has a separately named upgrade mutation and a SHA-256
over its exact observation. All eight admitted compilation units are
warning- and error-free. Rejected near misses deliberately include both
compiler-successful profile exclusions and compiler-error shapes.

Each compilation records exact source bytes/hash, diagnostics, syntax nodes,
tokens and directive trivia with UTF-16 spans, declared and selected symbols,
conversion classification, complete root `IOperation` trees, target operation
shapes, and CFG blocks, regions, branches, implicit flags, and spans where a
method or constructor body supplies a graph. Source-type inventories include
all explicit and implicit members. Deterministically emitted, never-executed
probe assemblies are re-imported through public APIs; their symbol views and
raw ECMA-335 type/member/custom-attribute/signature rows retain the compiler-
owned `IsExternalInit`, `RequiredMemberAttribute`, and
`CompilerFeatureRequiredAttribute` observations that source symbols summarize
or omit. Selected intrinsic symbols include complete containing-type and
parameter signatures, while string, array, and date observations retain their
incidental generic metadata without admitting that metadata as source surface.

### 6.2 Bounded outputs and non-activation

W04 adds only:

- `develop/probes/csharp-03/DataConstructionProbe.cs` and its README;
- `develop/probes/csharp-03/run-data-construction-probe.py` and sanitized shell
  wrapper;
- the canonical private measurement above; and
- `crates/mpk-cli/tests/csharp_practical_probes.rs` as the exact
  `CSHARP-03-T01-W04` primary owner.

At W04 completion, the runner source and shell-wrapper raw SHA-256 values were
`2ac7d9491a618a29d44cc695f3dbc831e71710ef7003d938085f58a9e01c7731`
and
`f39aa6a2dec1db6f90128b981a2cd77058048e178c4cf8d95fdd6f379c8b488b`.
The then-current probe README and primary test raw SHA-256 values were
`24db0e39cbbe607738956dc1c63a6d23000985b9ec060a4c96465e0d2c44a8eb`
and
`59cfd5b48ab58a67035bc39baf8769d2d7e57f6ad06a3cb46f11e6e5e8a33811`.

The probe sources are harness-owned; no selected application snippet contains
an MPK source or binary dependency, and no snippet or emitted probe assembly is
executed as application code. W04 does not freeze the final operation set,
semantic templates/bindings, schemas, vectors, diagnostic identities, limits,
or practical profile identity. Those remain T01-W08-W10 scope. Control,
exception, and pattern probes remain W05-owned; dependency/generic/iterator/
async probes remain W06-owned; run-time semantic probes remain W07-owned.

No production source, normative specification/vector, fixture, candidate,
build input, public route, registry, release descriptor, or installed artifact
changed. The active scalar descriptor, inventory, and vector remain at the W03
raw SHA-256 values
`0345044d16d4efb3568c32a3d7bc67fec508fe9359eff423a7f09c7f69b348dc`,
`4ff3ba6fdc2eb2857c32563b959f11194075a4264164cd7aebc808858e500e9b`,
and
`8109f781ca1f2b90ba02f786da09ba97602f4cd484b8835b561d5ecf4e7781c8`.
At W04 completion, W05 was the sole ready item; section 7 records its later
completion and W06 readiness.

### 6.3 Verification and review

Target host: Darwin arm64. The following local checks passed:

- `cargo test -p mpk-cli --test csharp_practical_probes`;
- `cargo fmt --all -- --check`;
- `cargo clippy -p mpk-cli --test csharp_practical_probes -- -D warnings`;
- `./develop/probes/csharp-03/run-data-construction-probe.sh --check-record`;
- `./develop/probes/csharp-03/run-data-construction-probe.sh --self-test`;
- `cargo test -p mpk-vc --test csharp_practical_inventory`;
- `python3 scripts/check-artifact-paths.py`;
- `python3 scripts/check-spec-vectors.py --check`; and
- `./scripts/check-fast.sh`.

The exact two-build/two-run probe check passed under x86-64 emulation in the
existing Linux local-gate image
`sha256:ea3189955dd9c0e5deda7a30ef48a0c7ef5af3b128f74fcd6e368384b8e1420a`
with no network, a read-only container root, a fresh executable tmpfs, the
fixed W03 archive cache, empty compiler/run environments, and fresh output
directories. The command is recorded in
`develop/probes/csharp-03/README.md`; its `--check` action reproduced the
checked-in result byte-for-byte after all fixes.

Review/fix history:

- The first pass found that repeated full symbol/operation objects inflated
  the observation to 22.6 MB. Full operation trees now live once at their
  roots, CFGs retain summaries and edges, and detailed symbols remain at target
  and source-type ownership points; no required observation was removed.
- The second pass added direct near misses for global/static/alias imports,
  nullable directive variants, expression-bodied construction, multi-`var`,
  required non-init state, and constructor/object-initializer rewrites.
- The third pass found one admitted nullable `Value` probe produced `CS8629`.
  Its source now establishes presence first, and every admitted compilation is
  diagnostic-free.
- The fourth pass found a compilation-unit marker selected the root rather
  than its first `using`, and incidental string/array metadata markers selected
  parameters instead of values. Target selection now excludes the root and the
  metadata cases bind exact value expressions.
- The fifth pass found selected method symbols were recorded without
  containing type or parameter signature. The public symbol display now fixes
  complete overload identities.
- The sixth pass found Roslyn source symbols expose requiredness as symbol
  properties while omitting the compiler-emitted required attributes from
  `GetAttributes()`. The probe now emits deterministic temporary metadata,
  re-imports it, and records raw ECMA-335 custom attributes and signature blobs
  through public APIs.
- The seventh pass removed an unused runner import and added a direct assertion
  for `CompilerFeatureRequiredAttribute`, so the required-member marker promise
  no longer relies only on the whole-document fingerprint.

Final review findings: `0`. The final pass checks W04-only scope, exact W03
input binding, complete task-term routing, syntax/symbol/operation/CFG and
emitted-marker evidence, source/span/hash integrity, diagnostic-free admitted
cases, near-miss coverage, one unique upgrade mutation per admitted shape,
changed-observation rejection, deterministic rerun bytes, active scalar byte
preservation, non-activation, and serial ledger state after all fixes.

## 7. CSHARP-03-T01-W05 completion record

### 7.1 Pinned control, exception, and pattern observations

W05 consumes only the completed W04 commit
`b6680168c2666be503741575c009f0a26dd0da22`, tree
`0f1e86bbdf986870b60fe335da58290baac26b0f`, and canonical W04 result raw
SHA-256
`c5de8bc209331c2295497210a570ba0be32e0871b3dd2576980d6c109222142e`.
It retains the W03 descriptor, candidate inventory, toolchain, and reference-
projection SHA-256 values
`83bf64dcbedce89f79613fe7aab3d95a92179122df54f9b5407273a245738015`,
`ff4b48790c67135144419c816149f8edfbd7b40ade231d6ab44c8433efef0cce`,
`d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f`,
and
`30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad`.

The disposable public-Roslyn-API probe source at
`develop/probes/csharp-03/ControlExceptionPatternProbe.cs` is 70,299 bytes
with raw SHA-256
`f62ff3deb7c0fff2799f99426ab9dbd7e6fd373a5fd9d8ed91bbb118a9808f1f`.
Its compiler uses `MetadataImportOptions.Public`; the owner test rejects
reflection/private-binding escape names and direct compiler-internal
namespaces. No selected source unit contains an MPK package, namespace,
assembly reference, attribute, interface, base type, generated source, or
runtime component.

The canonical result at
`develop/migrations/csharp-03/probes/roslyn-control-exception-pattern.json` is
2,331,920 bytes with raw SHA-256
`b1215ad7f4a0e08dc269834229d7158158d31c0e9475218fa0791feea5a1629a`.
Its normalized raw-observation section is 2,277,192 bytes with SHA-256
`14e14bd219678f49e53efa660d127ebded7b74af4dd72a790bdda5fe2d7b80d3`;
the deterministic probe binary SHA-256 is
`f4730abbac2055aa795208b30a1926bd4ff4e4606343dc92818d3e718f714b01`.
Both clean reruns agree on all three values.

Eighteen isolated compilation units record 103 distinct source shapes: 62
proposed admitted shapes and 41 rejected near misses. The sorted whole,
admitted, and rejected shape-ID sets have SHA-256 values
`431e5891260b9e3284f6b3646ae25d4643d9d53c8fdede0db69a1d2fd5d2d501`,
`524e05d67fa72c5520176711f06a42739f44d881ad30e2c0b31cfbc83f76864c`,
and
`b510c715a9d915a1217bccfbcc80611877c06a5bd8cf036bd1959db7012e0870`.
All eight admitted units are warning- and error-free. The rejected units
retain compiler-successful profile exclusions, warning-only forms, and
compiler-error forms as three separately required outcome classes. All 83
method/constructor operation roots in the 15 compiler-successful units have
exactly one CFG; compiler-error units retain every graph Roslyn can construct.

The source corpus covers all four loop statements, explicit-type and `var`
array `foreach`, exact string `foreach`, structured break/continue, switch
statements and expressions, guards, every closed admitted pattern category,
standalone and propagated throws, exact source/built-in exception
construction, ordered typed catches, immutable payload access, rethrow,
filters and filter failure, nested lexical search/unwind, `finally`, and each
normal or abrupt completion interaction. Rejected near misses cover jumps,
unsupported enumeration, open/effectful/deconstruction/dynamic/slice pattern
forms, non-exhaustive/fall-through/goto switches, exception construction and
runtime-state escapes, invalid handlers/filters/rethrow, and illegal exits
from `finally`.

Each unit records complete syntax and operation roots, exact target and source
spans, diagnostics, CFG blocks/branches/region trees, exception regions,
abrupt completions, and one combined lexical source-order sequence. Guard and
filter expressions—including the throwing filter—are explicit decision
nodes. Explicit and omitted parameterless `System.Exception()` base calls are
both fixed as invocation operations. All 40 decision graphs and 25 exception regions have distinct upgrade mutations bound to their exact observation
hashes. The 65 mutations change a decision-node operation kind, handler search
order, or exception-region nesting depth rather than only envelope metadata;
each mutation must fail full-document validation.

### 7.2 Bounded outputs and non-activation

W05 adds only the disposable probe and sanitized runner, its canonical private
measurement, the W05 section of the probe README, owner-test extensions in
`crates/mpk-cli/tests/csharp_practical_probes.rs`, and status/ledger updates.
The runner source and shell-wrapper raw SHA-256 values are
`cf452035539a3d5f48eb74d5fafe04d917cf6c486922b07c5f6cb87b3470e45e`
and
`35e92c495431eb502db2662209a1261d0d2635a1e93781c83a9a72403218fa36`.
At W05 completion, the then-current probe README and primary test raw SHA-256
values were
`d90f90a8b517bcfc2ec30464365907e02c14b3b9cb4a0894d4c46fff32cb3355`
and
`6f86651b930068feb81a904703c0921df62fe23c8e972c9ba3912dab09deaa5c`.

No production source, normative specification/vector, fixture, candidate,
build input, public route, registry, release descriptor, or installed artifact
changed. The active scalar descriptor, inventory, and vector remain at raw
SHA-256 values
`0345044d16d4efb3568c32a3d7bc67fec508fe9359eff423a7f09c7f69b348dc`,
`4ff3ba6fdc2eb2857c32563b959f11194075a4264164cd7aebc808858e500e9b`,
and
`8109f781ca1f2b90ba02f786da09ba97602f4cd484b8835b561d5ecf4e7781c8`.
W05 does not freeze decision semantics, exception lowering, diagnostic or
profile/schema identities, or a production route. At W05 completion, W06 was
the sole ready item; section 8 records its later completion and W07 readiness.
Runtime behavior remains W07-owned and normative freeze remains W08-W10-owned.

### 7.3 Verification and review

Target host: Darwin arm64. The following local checks passed:

- `cargo test -p mpk-cli --test csharp_practical_probes`;
- `cargo test -p mpk-vc --test csharp_practical_inventory`;
- `cargo fmt --all -- --check`;
- `cargo clippy -p mpk-cli --test csharp_practical_probes -- -D warnings`;
- `./develop/probes/csharp-03/run-control-exception-pattern-probe.sh --check-record`;
- `./develop/probes/csharp-03/run-control-exception-pattern-probe.sh --self-test`;
- `python3 scripts/check-artifact-paths.py`;
- `python3 scripts/check-spec-vectors.py --check`; and
- `./scripts/check-fast.sh`.

The exact two-clean-build/two-clean-run check also passed under x86-64
emulation in the immutable local Linux gate image
`sha256:ea3189955dd9c0e5deda7a30ef48a0c7ef5af3b128f74fcd6e368384b8e1420a`
with no network, a read-only container and repository, a fresh executable
tmpfs, the fixed W03 archive cache, empty compiler/run environments, and fresh
output directories. The `--check` command recorded in
`develop/probes/csharp-03/README.md` reproduced the checked-in result
byte-for-byte after all fixes.

Review/fix history:

- Initial passes separated warning-only non-exhaustive switches from compiler-
  error fall-through cases, made reference-`foreach` a compiler-successful
  profile exclusion, corrected explicit base-constructor targeting and lexical
  try-nesting depth, and closed the abrupt-`finally` preservation matrix.
- The next pass made parenthesized/relational/logical patterns select their
  exact syntax/operation forms and added an independent `and` source-shape ID,
  so no admitted pattern category relies only on incidental graph presence.
- A CFG-completeness pass changed successful method/constructor graph failures
  from silent omission to a probe failure and independently checks all 83
  eligible roots. It also freezes diagnostic/outcome coherence and all three
  rejected outcome classes.
- The upgrade pass replaced envelope-ordinal mutations with substantive
  decision/region mutations, closed their exact fields and 22/18/11/14 family-
  disposition counts, and verifies every mutation both by hash and full-schema
  rejection.
- The final semantic pass added top-level guard/filter expressions to decision
  graphs, proved that filter failure remains present, fixed propagation to an
  exact invocation target, and exposed the omitted parameterless base call as
  an implicit `System.Exception()` invocation.
- The rejection-isolation pass separated positional deconstruction from open-
  hierarchy rejection and removed the slice from the string list-pattern
  case, leaving each near miss with one intended unsupported boundary.
- The target-integrity pass found that a preferred-kind search could skip to a
  later node, and that adjacent `and`/relational markers exposed the ambiguity.
  Every marker now binds only among nodes starting at the first actual token
  after its own comment, so missing or changed local syntax fails closed.

Final review findings: `0`. The final pass checks W05-only scope, exact W04/W03
input binding, complete admitted/rejected source-shape coverage, public API
use, compiler-success CFG closure, source/span/hash/order integrity,
diagnostic-free admitted units, warning/error/clean rejection coverage,
decision and exception-region mutation sensitivity, deterministic rerun
bytes, active scalar byte preservation, non-activation, and serial ledger
state after all fixes.

## 8. CSHARP-03-T01-W06 completion record

### 8.1 Frozen dependency, generic, iterator, and async observations

W06 consumes only the completed W05 commit
`13415911853c0368c103bd9d5feeb8374596d724`, tree
`5d9000f11b2c3cab35ad08dc61a66fb14894d249`, and canonical W05 result raw
SHA-256
`b1215ad7f4a0e08dc269834229d7158158d31c0e9475218fa0791feea5a1629a`.
It retains the W03 descriptor, candidate inventory, toolchain, and reference-
projection SHA-256 values
`83bf64dcbedce89f79613fe7aab3d95a92179122df54f9b5407273a245738015`,
`ff4b48790c67135144419c816149f8edfbd7b40ade231d6ab44c8433efef0cce`,
`d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f`,
and
`30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad`.

The disposable public-API probe source at
`develop/probes/csharp-03/DependencyGenericSuspensionProbe.cs` is 89,065 bytes
with raw SHA-256
`7e2114bdb75ef5b78e330c24e04c551c7766740ba037a12419547212026c6db6`.
It uses `MetadataImportOptions.Public`, public Roslyn symbols/operations and
public `System.Reflection.Metadata` readers. The owner test rejects reflection
binding/private access and direct compiler-internal namespaces. Probe snippets
and emitted assemblies are observations only and are never executed as
application code.

The canonical result at
`develop/migrations/csharp-03/probes/roslyn-dependency-generic-suspension.json`
is 4,511,101 bytes with raw SHA-256
`5dadf10613f95be9b35c108008a33474c55d222bef1be987c2614c6dcc48fe96`.
Its normalized raw-observation section is 4,400,807 bytes with SHA-256
`dd6a5b4bea8909e8f2680f050df2d167055c214e503d701b548a07639b11ac07`;
the deterministic probe binary SHA-256 is
`81edd76cd4fba206f005447dcf4cf46e85dd41421a45af09ccfef6dcb2755c3e`.
Both clean reruns agree on all three values.

Sixteen isolated compilation units record 144 distinct source shapes: 12
narrow admitted-exception observations and 132 rejected profile forms. The
sorted whole, admitted-exception, and rejected shape-ID sets have SHA-256
values
`6f7cb87aa1efae91b220244b5b85cac5d13e9995b8b93539bc04cc1925060446`,
`3529ba40edc421a2a19fe74eceaf825426063c4336da2b72c75cc4c06633d35c`,
and
`4a72c24a0b06bb25e4e8b69dcd17695a253d754ee85b6599c636d9d944415ef4`.
The 41 sorted rejection/exception family IDs have SHA-256
`407f67fc75f02b61d555834ade2f192e0db3e249f74f16b505291235bb7e93be`.
Rejected compilations retain compiler-clean exclusions, one warning-only
exclusion, and compiler-error near misses.

The corpus binds synthetic package, project, and ambient assemblies to exact
virtual origins and covers every task-owned dependency family, 23 source-
written attribute targets, and exact compiler-emitted `IsExternalInit`,
`RequiredMemberAttribute`, and `CompilerFeatureRequiredAttribute` evidence.
It distinguishes all closed generic families named by the task. The exact
value-type `T?` exception is immediately specialized to an `option` payload
with no residual type parameter; explicit `System.Nullable<T>`, its alias,
construction/cast shortcuts, invalid payloads, every user-generic form, and
arbitrary constructed framework types reject. Exact string/array/decimal/date
symbols remain observable despite incidental generic metadata, while all eight
source-visible transitive-metadata cases reject.

Iterator and async observations are rejection-only. Their 50 shapes cover
iterator/yield/enumeration protocols and async iterator/await/task/value-task/
awaiter/cancellation/state-machine forms, including task factories, races, and
parallel execution; emitted `d__` types and exact state-machine attributes are
retained as compiler evidence, never as admitted suspension semantics.
All 144 source shapes have distinct upgrade mutations bound to their exact
target hashes. The Python self-test changes every target and requires full-
document rejection; the independent Rust owner validates
every mutation hash and performs one full rejection mutation per each of the
41 families.

### 8.2 Bounded outputs and non-activation

W06 adds only the disposable probe and sanitized runner, its canonical private
measurement, the W06 probe README section, owner-test extensions in
`crates/mpk-cli/tests/csharp_practical_probes.rs`, and status/ledger updates.
The runner source and shell-wrapper raw SHA-256 values are
`b2bdeed1dc821a667359bb559bd16225b1e8d25b3f79709fe68a098e39bc1950`
and
`a332507e3b4113115844a4fcb82fd41d8c0408702d7036102f18ad091263870c`.
At W06 completion, the probe README and primary owner-test raw SHA-256 values
are
`eb69d8509beee56dd550bc7693cc4f2baf61b6ff0cd9cde79126fc0e76c6763e`
and
`8dd1ec95caf21bab7d0bd44ed19d7511a157b05f47a65ef01e7836a613ac755b`.

No production source, normative specification/vector, fixture, candidate,
build input, public route, registry, release descriptor, or installed artifact
changed. The active scalar descriptor, inventory, and vector remain at raw
SHA-256 values
`0345044d16d4efb3568c32a3d7bc67fec508fe9359eff423a7f09c7f69b348dc`,
`4ff3ba6fdc2eb2857c32563b959f11194075a4264164cd7aebc808858e500e9b`,
and
`8109f781ca1f2b90ba02f786da09ba97602f4cd484b8835b561d5ecf4e7781c8`.
W06 does not admit an MPK source dependency, user generic, iterator, async
form, or a new registered or normative profile/schema identity. W07 is the
sole ready item; normative freeze and activation remain W08-W10-owned.

### 8.3 Verification and review

Target host: Darwin arm64. The following local checks passed:

- `cargo test -p mpk-cli --test csharp_practical_probes`;
- `cargo test -p mpk-vc --test csharp_practical_inventory`;
- `cargo fmt --all -- --check`;
- `cargo clippy -p mpk-cli --test csharp_practical_probes -- -D warnings`;
- `./develop/probes/csharp-03/run-dependency-generic-suspension-probe.sh --check-record`;
- `./develop/probes/csharp-03/run-dependency-generic-suspension-probe.sh --self-test`;
- `python3 scripts/check-artifact-paths.py`;
- `python3 scripts/check-spec-vectors.py --check`; and
- `./scripts/check-fast.sh`.

The exact two-clean-build/two-clean-run check also passed under x86-64
emulation in the immutable local Linux gate image
`sha256:ea3189955dd9c0e5deda7a30ef48a0c7ef5af3b128f74fcd6e368384b8e1420a`
with no network, a read-only container and repository, a fresh executable
tmpfs, the fixed W03 archive cache, empty compiler/run environments, and fresh
output directories. The `--check` command recorded in
`develop/probes/csharp-03/README.md` reproduced the checked-in result
byte-for-byte after all fixes.

Review/fix history:

- The compiler/API pass corrected one nested observation tuple, removed a
  nonexistent yield-operation interface assumption, and made unexpected probe
  failures diagnosable without changing successful output.
- The emitted-metadata pass handles a nil base-type metadata handle before its
  default handle kind can be mistaken for a type definition, then verified
  required/init and all iterator/async state-machine evidence through public
  metadata APIs.
- The outcome pass added an isolated warning-only namespace dependency case so
  clean, warning, and error rejection classes are independently frozen.
- The owner-test pass found that nullable local-type and conversion records did
  not both carry immediate-specialization evidence. Target selection now binds
  the exact local `NullableType`, converted nullable types participate in the
  generic facts, and all four value-type `T?` observations prove a concrete
  payload with no residual type parameter.
- The coverage pass separated generic and non-generic `IEnumerable` protocol
  observations and added explicit `IAsyncEnumerable<T>` parameter evidence.
  It also gave the source-written `AttributeUsage` syntax its own rejected
  target, so every attribute syntax in the attribute fixture is owned. A final
  design-to-corpus comparison added `Task.Run`, `Task.WhenAny`, and
  `Parallel.For` observations for the explicit task-race and parallel-
  execution exclusions.
- The documentation pass corrected the probe README's admitted-source statement
  so it does not contradict W06's deliberately compiled negative MPK
  dependency cases.

Final review findings: `0`. The final pass checks W06-only scope, exact W05/W03
input binding, all dependency/generic/attribute/incidental-metadata categories,
the sole exact-`T?` constructed-generic exception and immediate specialization,
complete iterator/async rejection and state-machine observations, source/span/
hash/order integrity, clean/warning/error rejected outcomes, all shape/family
mutation links, deterministic rerun bytes, W05 and active scalar byte
preservation, non-activation, and serial ledger state after all fixes.

## 9. CSHARP-03-T01-W07 completion record

### 9.1 Frozen primitive, string, numeric, and codec observations

W07 consumes only the completed W06 commit
`22673dbc96d8ba4f0d9a4cb97c3f2490c00d1804`, tree
`687631b3799ba385ccde29de9d72286c48d3f8fc`, canonical W06 result raw SHA-256
`5dadf10613f95be9b35c108008a33474c55d222bef1be987c2614c6dcc48fe96`,
and W06 probe-source raw SHA-256
`7e2114bdb75ef5b78e330c24e04c551c7766740ba037a12419547212026c6db6`.
It retains the W03 descriptor, candidate inventory, toolchain, and reference-
projection SHA-256 values
`83bf64dcbedce89f79613fe7aab3d95a92179122df54f9b5407273a245738015`,
`ff4b48790c67135144419c816149f8edfbd7b40ade231d6ab44c8433efef0cce`,
`d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f`,
and
`30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad`.

The disposable runtime source at
`develop/probes/csharp-03/PrimitiveStringNumericCodecProbe.cs` is 126,717
bytes with raw SHA-256
`d587acd6b1baab5602c8da8c54a803a9baa797400b70a6328bfd059e6a9f5f42`.
It is compiled directly as C# 14 with the fixed 167-reference projection and
executed only on .NET 10.0.11 Linux-x64. It emits UTF-16, binary32/binary64,
and decimal sign/scale/coefficient encodings rather than culture-formatted
values. Raw exception prose, stack text, host paths, source snippets, and
ambient culture names are excluded.

The canonical result at
`develop/migrations/csharp-03/probes/runtime-primitive-string-numeric-codec.json`
is 9,318,258 bytes with raw SHA-256
`0055835ce456fb9c438336332bc0e2a214d900c137eca34f90c3fcddd2688769`.
Its normalized three-culture observation section is 6,641,752 bytes with
SHA-256
`872e6150d17476c52ee01db3530f9e710afc8c6252592daa5393f3c705e46967`;
the deterministic probe binary SHA-256 is
`7b61263a2847340902b5692dd397c458a72cdd24a7b9158a8f4b3ea2279d85ed`.
Both clean builds agree on the binary and all raw and normalized bytes.

The result contains 3,468 distinct runtime vectors grouped into
154 exact operations and 26 evidence families. Their sorted vector-,
operation-, and family-ID set
SHA-256 values are
`e4e2f9c55154bec304a66e80c5d574c071307ff91e4bd93b3a0073153905073c`,
`96db56971b3cc908ac618880bf4d1993567d0217ea3a325c14deb4691277b3a5`,
and
`802e897a25d358fce385ea9390da70a5c2cd5bb9a3d6f4dc5a419e5ee6e9da37`.
Each operation fixes its accepted domain, possible failures in precedence
order, observed failure IDs, result encodings, culture-invariant candidate
projection, runtime differential hashes, and one named input mutation.

The string corpus preserves UTF-16 code units, surrogate pairs, and lone
surrogates and closes ordinal operations, null/range exceptions, the exact
concatenation matrix, restricted interpolation normalization, and rejected
conversion/alignment/format holes. The codec corpus implements the exact
ASCII grammars independently of general framework parsing/formatting and
covers every syntax, noncanonical, range, scale/precision, and input-bound
parser failure. Formatter output obligations and sidecar codec/rounding
precedence are separate. Every integer, decimal, date/time, duration/instant,
floating-bit, and GUID codec has an explicit lossless round trip; fixed-scale
decimal round trips to the selected-mode rounded value.
Fixed-scale parsing removes only exact zero padding when a coefficient needs
more than 96 bits; it never rounds an unrepresentable input. Scale-2/28
maximum and minimum values, integer padding, negative zero, and least-fraction
rounding are checked by value equivalence rather than representation-bit
equality.

Binary32 and binary64 each use the exhaustive cross product of nine values for
every admitted binary arithmetic, Min/Max, and ordered-comparison operation,
plus all unary/intrinsic cases and critical conversions. The set includes both
zero signs, infinities, the least subnormal, quiet NaN, and signaling NaN;
every result remains an exact bit string. Decimal uses an exhaustive pairwise
small domain with distinct scaled/signed zero representations, explicit
sign/96-bit-coefficient/scale results, all five selected rounding modes,
integral conversions, equality normalization, division-by-zero precedence,
and operation-linked overflow endpoints.

Each build runs twice under each of three explicitly constructed hostile
current cultures. A second unlisted runtime environment value is also changed
for every culture, giving twelve isolated executions across the two builds.
All profile-side values, exact bits, error IDs, and precedence remain equal.
83 culture-varying BCL differential vectors are retained; their
sorted ID-set SHA-256 is
`d17191a68f4d0e2e0596e309e4e945765f294f7e0a2a2a397e558fc66ae0c965`.
Profile-side codec results come only from the probe's closed ASCII grammars;
BCL Parse/Format/ToString/interpolation results remain differential evidence.

### 9.2 Bounded outputs and non-activation

W07 adds only the disposable runtime probe, its sanitized runner and shell
wrapper, the canonical private measurement, its probe README section, owner-
test extensions in `crates/mpk-cli/tests/csharp_practical_probes.rs`, and
status/ledger updates. The runner and wrapper raw SHA-256 values are
`85c3c99b1e84af28f7c9b734c1e333e88b994e90bfff02ad3819e9ddccf7089e`
and
`db5cf9230665ffb3e52b1ea95b31bf828dfcd48178bd7f317125206befe7cc94`.

No production source, normative specification/vector, fixture, foundation
descriptor, candidate bundle, public route, registered identity, build input,
or active registry/release artifact changed. The active scalar descriptor,
inventory, and vector remain at raw SHA-256 values
`0345044d16d4efb3568c32a3d7bc67fec508fe9359eff423a7f09c7f69b348dc`,
`4ff3ba6fdc2eb2857c32563b959f11194075a4264164cd7aebc808858e500e9b`,
and
`8109f781ca1f2b90ba02f786da09ba97602f4cd484b8835b561d5ecf4e7781c8`.
W08 is the sole ready item. The normative freeze package remains T01-W08-W10-
owned; production implementation and atomic activation remain in the later
implementation/release milestones.

### 9.3 Verification and review

Target host: Darwin arm64. The following local checks passed:

- `cargo test -p mpk-cli --test csharp_practical_probes`;
- `cargo test -p mpk-vc --test csharp_practical_inventory`;
- `cargo fmt --all -- --check`;
- `cargo clippy -p mpk-cli --test csharp_practical_probes -- -D warnings`;
- `./develop/probes/csharp-03/run-primitive-string-numeric-codec-probe.sh --check-record`;
- `./develop/probes/csharp-03/run-primitive-string-numeric-codec-probe.sh --self-test`;
- `python3 scripts/check-artifact-paths.py`;
- `python3 scripts/check-spec-vectors.py --check`; and
- `./scripts/check-fast.sh`.

The exact two-clean-build/twelve-run check also passed under x86-64 emulation
in immutable local Linux gate image
`sha256:ea3189955dd9c0e5deda7a30ef48a0c7ef5af3b128f74fcd6e368384b8e1420a`
with no network, a read-only container/repository, a fresh executable tmpfs,
the fixed W03 archive cache, closed compiler/runtime environments, and fresh
output directories. The `--check` command recorded in the probe README
reproduced the checked-in result byte-for-byte after all fixes.

Review/fix history:

- The compile/runtime pass removed compiler-folded decimal overflow cases,
  routed them through runtime operands, and made 96-bit word conversions
  explicit under the globally checked build.
- The schema pass replaced a false one-family-per-operation assumption with an
  exact family set, allowing normal, null, and range observations to remain
  attached to one operation contract.
- The error-closure pass added missing null-receiver and input-bound vectors,
  made every operation's failure precedence exact, and requires every declared
  parse/exception/source-rejection failure to have a differential vector.
- The decimal pass moved endpoint failures from synthetic edge operation names
  onto `decimal.add`, subtract, multiply, divide, and remainder and added both
  division-by-zero and overflow outcomes under their declared precedence.
- The round-trip pass added separately indexed lossless relations for every
  codec and fixed-scale decimal rounded-value relations for all five measured
  rounding modes.
- The codec-independence pass separated candidate decimal round trips from
  BCL parsing and uses explicit ASCII digit generation for numeric/date/time
  output. BCL Parse/Format calls supply differential evidence only.
- The fixed-scale endpoint pass preserves exact values when zero padding
  exceeds 96 coefficient bits, rejects nonzero precision loss, and checks all
  five modes against the maximum, minimum, zero, integer, and least-fraction
  round-trip cases without requiring identical decimal representation bits.
- The precedence pass executes the real probe parsers and closed sidecar
  dispatch on multi-failure inputs, checks syntax before canonicality and
  scale before range, and records over-bound repeated text losslessly.
- The run-count pass rejects missing and extra culture runs before pairing
  them with the closed culture list and adds both rejection self-tests.
- The independent owner-test pass recomputes all vector, operation, family,
  culture-variant, observation, mutation, predecessor, and active-release
  links and rejects changed bits, parse/rejection IDs, precedence, and
  aggregate-index payloads. Recorded-input hash mutations are distinct from
  the real numeric/string/parser input cases executed by the runtime probe.

Final review findings: `0`. The final pass checks W07-only scope, exact W06/W03
input binding, complete §10/§11 operation and error ownership, UTF-16/null/
concat/interpolation behavior, every exact codec grammar and round trip,
float/double NaN and signed-zero bits, decimal representation/rounding/
overflow behavior, all three hostile cultures, unlisted-input independence,
every operation mutation, deterministic rerun bytes, W06 and active scalar
byte preservation, non-activation, and serial ledger state after all fixes.

## 10. CSHARP-03-T01-W08 completion record

### 10.1 Candidate foundation and specification handoff

The input is W07 commit `b0ff7daec663b95b1f88ecc1d98f0b7c1f6fdf00`, tree
`b74233683af4c85ba7576d65e627b0a7efa51598`. W03 build inputs, reference
projection, W04–W07 observations and active scalar release bytes are retained.
This subsection preserves the original W08 completion bytes; section 11 records
the authorized current-candidate amendment and superseding hashes. The original
data/foundation contract is
`develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md`, owned by
`crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W08`.

The one candidate descriptor at
`develop/migrations/csharp-03/foundation/foundation-descriptor.json` is 2,087
bytes, raw SHA-256
`1aceff726735f38a4b7c57e2ca1688c61428a49bb5997bb6963369d5beaf2118`.
Its domain-separated semantic content hash is
`d3c3422d509ee00fff3e98e6bc0d8b27b6f30bd2848a0d38d3873927813482e0`.
It binds the normative specification and the 18,051-byte definition inventory
`develop/migrations/csharp-03/foundation/foundation-definitions.json`, raw
SHA-256 `01f05d59c0a3642baeb4f3c5b55a6a946251bab71a2750f0115b9920b1ffd9bc`.
Exactly twelve templates and four non-template definitions are enumerated;
there is no source-callable member or caller extension point.

The freeze fixes stored declarations, constructors/init/required transactions,
receiver-first pure calls, default eligibility, array ownership and publication,
count/fill construction, ordered maps/sets and equality/order; all template
IDs/arities/dependencies/equations and ordinary-core representation recipes;
concrete root/argument/member closure, canonical instance IDs, deduplication,
provenance unions and counters; exact semantic bindings, typed operation maps,
field-complete projection obligations; nullable/outcome operations; and
date/time/duration/instant/GUID/money values, operations, codecs and errors.
Enum carrier width/signedness is explicit and carrier/tag values use canonical
decimal strings rather than unsafe JSON integer tokens.

There are 2,051 executable specification vectors in
`develop/specs/vectors/csharp-practical-foundation-v1.json` (795,975 bytes),
raw SHA-256 `af9695867a92a212fe2abd68e5c3b38ff9cc47d044b6860aece96fb2b1cf13f0`.
Every row names its primary downstream implementation task and exact test
owner; source count/fill execution is T04-W02, not T03's representation handoff.
The local comparison budgets are exercised at cap-1/cap/cap+1, including actual
255/256/257-instance closures and 15/16/17 argument depth. The all-template
specimen derives 13 distinct instances, 83 operations and 863 recipe nodes.
These are specification-recipe counts, not claims of measured kernel capacity;
the complete emitted-term and checker-capacity freeze remains T01-W09-owned.

At original completion, ordinary-core recipes used concrete finite-depth
Boolean function trees and existing Bool/Nat/Eq eliminators. Section 11
supersedes only that infeasible expansion recipe with the binary Bool cube and
static-transformer form; semantic operation behavior remains unchanged.
Field/sequence equality is an explicit relation, not function extensionality.
Actual definition construction and certificate proof discharge remain the named
T02/T06 work items; W08 does not claim they are already built.

### 10.2 Independent runtime evidence and non-activation

`develop/probes/csharp-03/FoundationDataProbe.cs` is 16,249 bytes, raw SHA-256
`940ea5faf7a5a35863f479aed619a4226dd960b050f173ff25b0387d00be9b1f`.
`foundation_runtime_model.py` independently computes expectations using
Gregorian/integer/Boolean/IEEE-bit/decimal-value algorithms. Two clean builds
under the fixed W03 compiler/reference/runtime closure each execute twice
under each of two hostile cultures (eight executions total).

The 501,898-byte result
`develop/migrations/csharp-03/probes/runtime-foundation-data.json`, raw SHA-256
`6ef1194e1398d5822c676248ea6ccbbb31381b95cfd32c8b8a65e68376118064`, contains
1,629 independent runtime vectors covering 82 operation groups. Its normalized
observation hash is
`fee66d2fe11251d0dbbcc2d0408bc32dbb5352d3a8a853f3a6ea6c640e9f5369`,
and deterministic binary hash is
`e796041d209811c5a3b968cd3caba95a70940cbc3f73509d9d01395c0686d02d`.
The record binds source/oracle/runner/W03 input hashes. The full read-only local
Linux rerun reproduces the record byte-for-byte, with network disabled and a
fresh executable tmpfs in the same immutable image recorded for W07.

Coverage includes Gregorian clamping/endpoints, wrapping time subtraction and
duration addition, signed duration components/overflow, unsigned GUID field
order, every admitted lifted nullable operator, NaN non-reflexivity, signed-zero
bits, bool? truth/equality tables, construction/init/receiver/argument order,
null-call argument evaluation, array defaults/errors, two-pass output order,
instant precision-before-range and difference overflow, and all money
arithmetic/rounding/error-precedence operations. Instrumented helper/setup APIs
remain observation-only; they add no application source admission. W07's sealed
primitive/codecs record is unchanged at
`0055835ce456fb9c438336332bc0e2a214d900c137eca34f90c3fcddd2688769`.

Only private probes/models, generated candidate specification artifacts,
normative vectors/manifest registration, owner tests and documentation changed.
No production source, installed descriptor, public route, active registry,
application dependency, release candidate bundle, core/checker behavior or
GitHub workflow changed. The active scalar descriptor/inventory/vector remain
at the raw digests recorded in section 9.2. At W08 completion T01-W09 was the
sole ready item; section 11 records the later finding and authorized amendment
that restores that readiness. W10 and all production implementation/activation
work remain serially blocked.

### 10.3 Verification and review

Local verification:

- `python3 develop/probes/csharp-03/foundation_package.py --check`;
- `python3 develop/probes/csharp-03/run-foundation-data-probe.py --check-record`;
- the README's fixed offline Linux, read-only `--check` command;
- `cargo test -p mpk-vc --test csharp_practical_spec --test canonical_json`;
- `cargo test -p mpk-vc --test csharp_practical_inventory`;
- `python3 scripts/check-spec-vectors.py --check` (25 vector sets);
- `python3 scripts/check-artifact-paths.py`;
- `cargo fmt --all -- --check`; and
- `./scripts/check-fast.sh`.

Review/fix iterations:

- Separate binding admission from actual-source-default eligibility; none/missing
  internal defaults do not replace CLR null/zero storage or ignore invariants.
- Include every stored source member in projection obligations, distinguish
  representation round trips from IEEE operator equality, reject inactive
  payload loss, arm collapse, signature mismatch and operation non-commutation.
- Recursively collect instances through source member graphs as well as direct
  template arguments/dependencies; preserve all root provenance after dedup.
- Reject old-owner reads after transfer and inconsistent branch owners/lifetimes;
  intersect initialization facts and retain explicit SSA phi values at joins.
- Preserve negative zero for floating remainder/negation, exercise nullable NaN
  comparisons and bool? equality, and verify arguments precede null-call failure.
- Use shared-safe JSON integer tokens, explicit decimal-string wide carriers
  and enum underlying types; update the closed vector-manifest count from 24
  to 25 without changing parser acceptance rules.
- Correct loop and instance-call downstream ownership, bind normative document
  bytes into the descriptor, retain predecessor/active hashes, and verify the
  generated descriptor, definitions, operation closure, counters and evidence
  independently in Rust as well as through the Python model.
- Disable probe-side Python bytecode writes; move only the two W08-created
  Python 3.11 cache files to a disposable temporary directory, restoring the
  frozen W02 consumer search without changing its inventory or fingerprints.

Final review findings: `0`. The final pass covers W08-only scope, descriptor
member/hash closure, exact twelve/four registry, generic erasure, source-member
closure, bindings and default/projection distinctions, core-recipe trust
boundary, field/collection semantics, all runtime comparisons and precedence,
manifest/schema/owner consistency, serial ledger state and non-activation.

## 11. CSHARP-03-T01-W09 feasibility finding and resolution record

Status: finding `CSHARP-03-T01-W09-F01` (P1) is `Resolved` (2026-09-04).
This section preserves the authorized W08 amendment and counterexample;
section 12 records the subsequent W09 completion and capacity freeze.

### 11.1 Retained counterexample

Entry commit is `4ffd8b3a9918b6cae9e4d4704e4bc6b09a12cd5c` on `main`, with a
clean worktree at entry. The original W08 section 4 assumed arbitrary concrete-
result Bool/Nat elimination. The checked interfaces instead have fixed results:

```text
Std.Bool.rec : Bool -> Bool -> Bool -> Bool
Std.Nat.rec  : Nat -> (Nat -> Nat -> Nat) -> Nat -> Nat
```

The retained direct `Bool.rec`-to-tree and `Nat.rec`-to-Bool certificates still
fail declaration typechecking in both checkers, including Nat major zero. This
rules out the old recipe and a smaller numerical cap as its repair; it does not
change or weaken either checker.

### 11.2 Authorized replacement and dual-checker evidence

The replacement carrier is `C(0)=Bool; C(d+1)=Bool -> C(d)`. Selector binders
are binary address bits, least-significant first. Products, sums, padding and
sequence indexing expose every coordinate before selecting through
`Std.Bool.rec`, so its cases, major and result are always Bool. Bounded scans
statically compose closed, concrete `S -> S` state transformers with ordinary
`Lam`/`App`/`Let`; they never eliminate Bool or Nat into `S` and never apply
`Std.Nat.rec`. Fixed-width numeric/index/codec work remains finite Boolean
circuit generation, with no unary wide scalar or theory shortcut.

The complete rules are in the amended
`develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md` section 4. The canonical
record is `develop/migrations/csharp-03/probes/recursor-feasibility.json`, raw
SHA-256
`c1a9024df81555ab3af21926885c62a1da88ded918842ca9f657794a079a8785`.
It binds 73 checker/build/probe source files and the unchanged checked standard
inputs:

- Bool raw SHA-256:
  `88a37f9df68a18bc19d51c0832279ff97a2944fc1676326022269766933ee806`;
- Nat raw SHA-256:
  `65bc5b701561d07b0f675668d942917da23c43606c0cfe0f9edfa113be99f583`.

Each of 15 self-contained Certificate v0 cases has 13 declarations and zero
axioms, proof nodes and theory certificates. Each checker receives identical
bytes twice: 15 cases x 2 checkers x 2 runs = 60 invocations. Per run, ten
controls/replacement cases accept, including the two-address Bool cube and a
function-valued state advanced through cross-coordinate static transformer
composition; the five old cross-result cases reject with the original type
mismatch. The probe independently counts direct `Std.Nat.rec` applications and
requires zero in every replacement case. Unexpected builds, timeouts, signals
or checker errors abort the runner and cannot become a semantic verdict.

### 11.3 Candidate amendment and remaining boundary

The candidate artifacts were regenerated as one deterministic set from the
amended model:

| Artifact | Raw SHA-256 / identity |
| --- | --- |
| foundation specification | `29c5986e3c7ce2ab018e36eea61caaf9d9e53d6b8e47f0229ef4681db8c3fc8b` |
| foundation definitions | `25738447bf793e37dc2125e7a07da55a03fb15f2fa4dfb87b25646a16cc9d1b4` |
| foundation descriptor content hash | `d8c2a023f1c445470123519f5024a17aaca1766553331a2fed4733fecf7deec1` |
| foundation conformance vectors | `5889d91e2365dfb8bce4260a4eae0fb3dd63b2e5fa430f7ed5a6dc8a0220bdc1` |

The candidate remains private and inactive. No core/checker behavior,
production frontend, public route, installed profile/registry, release input,
application source, source-visible generic, or value bound changed. The probe's
`capacity_measurement` now binds the separate checker-capacity record while
`release_gate` remains false. Section 12 owns that measurement and the final
W09 boundaries; this feasibility result alone makes no capacity claim.

### 11.4 Verification and review

Completed local checks:

| Command | Result |
| --- | --- |
| `python3 develop/probes/csharp-03/run-recursor-probe.py --check` | pass: expected verdicts in all 60 invocations |
| `python3 develop/probes/csharp-03/foundation_package.py --check` | pass: 2,051 vectors, 12 templates, 4 non-template definitions |
| `cargo test -p mpk-vc --test csharp_practical_spec --test csharp_practical_inventory` | pass |
| `python3 scripts/check-spec-vectors.py --check` | pass: 25 vector sets |
| `python3 scripts/check-artifact-paths.py` | pass: 256 registered canonical JSON artifacts |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

The default fast gate's provisioned-Linux tests remain ignored as documented;
this amendment is not a native Linux release receipt.
Feasibility-amendment review findings: `0`. The complete W09 review and
publication boundary are recorded separately in section 12.

## 12. CSHARP-03-T01-W09 completion record

### 12.1 Frozen private handoff

W09 consumes W08 commit
`4ffd8b3a9918b6cae9e4d4704e4bc6b09a12cd5c`, the complete W02 17-family
consumer inventory, the amended W08 foundation descriptor/content hash, and
the section 11 recursor evidence. It produces no public schema, manifest entry,
installed bundle, registry value, or production route. W10 remains the sole
publication owner.

The generated private freeze at
`develop/migrations/csharp-03/freeze/profile-freeze.json` is 97,316 bytes with
raw SHA-256
`83954067c156e58cb349dbf07da44edf60a3ec550e628e6d2f1a890889d574e3`.
Its domain-separated content SHA-256 is
`f292de00a79048ecd1ff2cbe52d90fad36654f1b3e74ad580b5ec3077afa28cb`
under `MPK-CSHARP-PRACTICAL-FREEZE-1.0`. It binds the W02 inventory raw SHA-256
`14b861354c54a59b06e625810b106d53fa830a39d49e68c3aef9ec82b93fef55`,
the W08 foundation descriptor content SHA-256
`d8c2a023f1c445470123519f5024a17aaca1766553331a2fed4733fecf7deec1`,
the recursor-evidence raw SHA-256
`c1a9024df81555ab3af21926885c62a1da88ded918842ca9f657794a079a8785`,
and the checker-capacity evidence below.

The freeze contains all 17 successor identity families and their unchanged
W02 implementation owners; globally unique successor IDs and hash domains;
the exact five-profile by nine-category compiled-contract matrix; 15 strict
root schemas; 20 strict nested records; three closed tagged unions; a closed,
field-complete explicitly typed contract-expression union; canonical
JSON and raw/canonical/reparsed boundary linkage; transition, response, event,
version and error precedence; optional complete-snapshot idempotency; 29
diagnostic families; total-termination rules; 35 practical limits; and all 32
unchanged scalar-v0 limits. Unknown fields/tags, duplicate JSON keys, later
versions, mixed artifact families, and ambient profile selection all reject.

The private vectors at
`develop/migrations/csharp-03/freeze/profile-freeze-vectors.json` contain 700 sorted rows and
are 230,986 bytes with raw SHA-256
`7d1de4f4d087fe0de7b32ec44ee2b17f08cbfb052e5993699137c47736c94ef3`.
They cover every strict root's valid value, later version, unknown field,
missing required field, wrong field type, and duplicate-key mutation; every
strict nested record, expression arm, and tagged-union arm's valid/unknown/
missing/wrong-type/duplicate cases plus unknown tags; expression-tag closure;
identity/domain collision and old/new mixing; canonical JSON; missing/null/
value and raw/canonical/reparsed linkage; transition/idempotency precedence;
every diagnostic adjacency; every retained and new counter at limit minus one,
limit and limit plus one; dispatch; and total termination. Each row names one
downstream implementation task and exact primary production-test owner.

### 12.2 Boundary, transition, identity, and executable decisions

The boundary document is an MPK verification-overlay transport. It is not an
application API, stored model, production serializer, or external-company
deployment dependency. Input evidence retains both raw provenance/bytes and
the independently parsed canonical typed value; output evidence retains the
source value, canonical bytes, and reparsed value. Typed equality is checked
field-completely; digest equality is never a substitute.

Transition version is unsigned 64-bit and advances by checked addition only on
a new success. Replay and all errors preserve the complete state. Retained-key
snapshot replay/conflict precedes expected-version conflict, which precedes
history capacity, version exhaustion, declared business errors, and new
success. Event order is source append order. Idempotency is unavailable unless
the retained record stores the key, complete application-owned `Command` and
`Context` snapshots, and complete response, and a source equality helper is
proved field-complete and equivalent to their canonical field encodings.
`float`, `double`, recursively containing types, and any other non-reflexive
snapshot are ineligible.

One successor frontend bundle,
`frontend.csharp.csharp2vir.candidate.v2`, with `csharp2vir.dll` serves both
`mpk.csharp.scalar.v0` and `mpk.csharp.practical.v1`. Dispatch uses only the
validated `semantic_context.semantic_profile`; an environment flag, fallback,
or mixed old/new artifact family rejects. The scalar route must pass byte-level
predecessor source-verdict, obligation, and Certificate v0 equivalence before
atomic activation. The neutral successor assembly profile is
`mpk.program_certificate.ordinary_context.v2`; existing Certificate v0 and
checker hash domains retain their exact meanings.

### 12.3 Checker-capacity evidence

The record at
`develop/migrations/csharp-03/probes/checker-capacity.json` is 38,701 bytes
with raw SHA-256
`de040d4342e90a23e4bbe6464aeaccbfa9f2630c1423b77b716b40c805ac8a99`.
Its 73-file checker/build/probe source inventory SHA-256 is
`e855ce008b87b4509a8af7d3b07ce5f907f9a98383b942710d362a146a2d0e38`.
The probe generates self-contained ordinary Certificate v0 networks at:

| Counter | Below / at / above |
| --- | --- |
| binder depth | 255 / 256 / 257 |
| successor-generated declarations, excluding the pinned prelude | 8,191 / 8,192 / 8,193 |
| total ordinary term nodes | 262,143 / 262,144 / 262,145 |
| statically composed concrete transformers | 16,383 / 16,384 / 16,385 |

All twelve cases have zero axioms, proof nodes, and theory certificates. Each
checker receives identical bytes twice: 12 cases x 2 checkers x 2 runs = 48
accepted invocations. The largest certificate is 2,081,286 bytes. Recorded Rust
observations range from 35 to 1,814 ms and reference-Go observations from 26 to
397 ms under a 60-second per-invocation failure bound. Timing is observational
and excluded from stable rerun equality; certificate bytes/hashes, verdicts,
exits, stdout/stderr and their hashes must match. The profile accepts below/at
and rejects above before checker invocation, retaining measured one-step
headroom without modifying either checker.

### 12.4 Non-activation and verification

Only private migration evidence, disposable generators/probes, owner tests,
and design/task/ledger/probe documentation changed. The accompanying W08
amendment regenerates its candidate foundation artifacts and preserves their
semantics. No production source, core/checker rule, source-visible package or
generic, external-company application file, installed registry/release input,
candidate executable bytes, public route, or GitHub workflow changed.

Completed local verification:

| Command | Result |
| --- | --- |
| `python3 develop/probes/csharp-03/run-recursor-probe.py --check` | pass: 60 expected dual-checker verdicts |
| `python3 develop/probes/csharp-03/run-checker-capacity-probe.py --check` | pass: 48 checker acceptances and stable outputs |
| `python3 develop/probes/csharp-03/profile_freeze.py --check` | pass: 17 families, 15 schemas, 35 practical limits, 700 vectors |
| `python3 develop/probes/csharp-03/foundation_package.py --check` | pass: amended W08 artifacts reproduce |
| `cargo test -p mpk-vc --test csharp_practical_spec --test csharp_practical_inventory` | pass |
| `python3 scripts/check-spec-vectors.py --check` | pass |
| `python3 scripts/check-artifact-paths.py` | pass |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

Review/fix iterations:

- The schema-owner pass found that the validated semantic request's declared
  hash field was absent from its ordered required fields. The field was added,
  and both the generator and independent Rust owner now require every hashed
  root to place its hash last.
- The evidence-state pass replaced the feasibility record's obsolete
  capacity-pending marker with an exact path/raw-hash binding to the separate
  capacity record; the small recursor cases remain explicitly non-capacity
  evidence.
- The diagnostic pass corrected the frozen sanitized public message's UTF-8
  byte count, changed the entry field from a literal placeholder token to the
  exact frozen-message type, restricted phases to 0 through 8, closed the
  phase/code and source-location rules, and added generator-side recomputation.
- The schema-closure pass found untyped expression fields, implicit nested
  requiredness, an underspecified boundary-default arm, and missing exact
  practical-parameter/selection roots. It closed all field-type expressions,
  restricted construction and public invariants to Bool expressions, made all
  tagged unions strict, made the shared validated request dispatch its strict
  selection shape from the registry entry, and added the missing roots without
  changing an active schema.
- The parameter pass cross-checked the W04-W06 pinned Roslyn compilation
  options and corrected the practical checked-overflow default to the measured
  enabled value rather than copying the scalar-v0 predecessor value.
- The source-artifact pass found that the success envelope's reduced three-
  artifact record omitted semantic bindings, closed instances, the foundation,
  and boundary/transition contracts. It now requires the complete strict
  `mpk.frontend.source_artifacts.v2` root.
- The frontend-linkage pass found that success did not repeat the validated
  request hash and that diagnostics could not distinguish pre-validation from
  validated failures. Success now binds the request hash, semantic context,
  complete artifact root, matching artifact context, and request selection;
  diagnostics always bind the raw request and use a closed
  `unvalidated`/`validated` linkage union without partial artifacts.
- The context-binding pass found that the shared validated request hard-coded
  the practical C# selection shape and did not enumerate its registry-entry or
  repeated-context equalities. It now dispatches the strict selection envelope
  from the resolved entry and freezes every field-complete mismatch rejection.
- The vector-ownership pass replaced the single placeholder schema/limit owner
  with each frozen producer or counter owner and its task-plan primary test;
  it also added wrong-type coverage for nested records and full valid/unknown/
  missing/wrong-type/duplicate coverage for every expression and union arm.
- The status pass corrected the stale design sentence that still called W09
  ready after its handoff had completed.

Final review findings: `0`. The final pass checks W09-only scope plus the
authorized W08 amendment, all W02 families/owners, global name/domain
uniqueness, strict schema and expression closure, boundary non-protocol status,
complete-snapshot idempotency, transition/diagnostic precedence, every counter
boundary, dual-checker capacity/reproducibility, total termination, W10-only
publication, predecessor preservation, serial ledger state, and non-activation.
