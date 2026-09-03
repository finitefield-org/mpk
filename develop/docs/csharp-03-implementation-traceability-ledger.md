# CSHARP-03 Implementation Traceability Ledger

Status: `CSHARP-03-T01-W01/W02` complete (2026-09-03). The entry audit and
consumer inventory are closed, `CSHARP-03-T01-W03` is ready, and every later
work item remains blocked by its serial predecessor. No practical-profile
identity, production acceptance path, candidate, or active registry entry was
introduced by W01/W02.

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
| `CSHARP-03-T01-W02` | `Complete` | `crates/mpk-vc/tests/csharp_practical_inventory.rs#CSHARP-03-T01-W02` | `SELF` |
| `CSHARP-03-T01-W03` | `Ready` | `crates/mpk-cli/tests/csharp_practical_build_inputs.rs#CSHARP-03-T01-W03` | `—` |
| `CSHARP-03-T01-W04` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_probes.rs#CSHARP-03-T01-W04` | `—` |
| `CSHARP-03-T01-W05` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_probes.rs#CSHARP-03-T01-W05` | `—` |
| `CSHARP-03-T01-W06` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_probes.rs#CSHARP-03-T01-W06` | `—` |
| `CSHARP-03-T01-W07` | `Blocked` | `crates/mpk-cli/tests/csharp_practical_probes.rs#CSHARP-03-T01-W07` | `—` |
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
bind 4,920 family-to-path consumer hits. Every hit resolves through an ordered
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
