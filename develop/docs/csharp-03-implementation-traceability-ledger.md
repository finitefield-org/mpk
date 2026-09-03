# CSHARP-03 Implementation Traceability Ledger

Status: `CSHARP-03-T01-W01/W02/W03/W04/W05/W06` complete (2026-09-04). The entry
audit, consumer inventory, private frontend/toolchain closure proof, and
Roslyn data/construction/control/exception/pattern/dependency/generic/iterator/
async-rejection measurements are closed, `CSHARP-03-T01-W07` is ready, and
every later work item remains blocked by its serial predecessor. No practical-
profile identity, production
acceptance path, registered candidate, or active registry entry was introduced
by W01-W06.

This ledger is subordinate to
[`08_csharp_practical_subset_design.md`](08_csharp_practical_subset_design.md)
and
[`08_csharp_practical_subset_design-todo.md`](08_csharp_practical_subset_design-todo.md).
The design and task document remain authoritative for semantics, scope,
dependencies, and exit gates. This file records execution state and evidence;
it does not freeze a new profile or alter an active release.

## 1. Ledger rules

- Work items execute in the serial order defined by the task document. Exactly
  one row may be `Ready`; no two rows may be `In progress`.
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
| `CSHARP-03-T01-W06` | `Complete` | `crates/mpk-cli/tests/csharp_practical_probes.rs#CSHARP-03-T01-W06` | `SELF` |
| `CSHARP-03-T01-W07` | `Ready` | `crates/mpk-cli/tests/csharp_practical_probes.rs#CSHARP-03-T01-W07` | `—` |
| `CSHARP-03-T01-W08` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W08` | `—` |
| `CSHARP-03-T01-W09` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W09` | `—` |
| `CSHARP-03-T01-W10` | `Blocked` | `crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W10` | `—` |
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
