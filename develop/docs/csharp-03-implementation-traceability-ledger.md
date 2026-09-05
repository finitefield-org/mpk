# CSHARP-03 Implementation Traceability Ledger

Status: `CSHARP-03-T01-W01/W02/W03/W04/W05/W06/W07/W08/W09/W10`,
`CSHARP-03-T02-W01/W02/W03/W04/W05/W06/W07/W08/W09`, and
`CSHARP-03-T03-W01/W02/W03/W04/W05/W06/W07/W08/W09/W10` complete (2026-09-06). The entry audit, consumer
inventory, private frontend/toolchain closure proof,
Roslyn shape probes, primitive/string/numeric/codec runtime measurements and
candidate foundation/specialization/binding/data semantics and the successor
contract/boundary/transition/identity/limit freeze have historical completion
records. The authorized W08 expansion amendment resolves
`CSHARP-03-T01-W09-F01` without changing core; W09 then measures checker
capacity and completes the private freeze. W10 publishes the complete normative
but inactive profile/shared-artifact package, 700 vectors, owner closure,
upgrade matrix, and future release-gate decision. T02-W01 implements the closed
candidate successor registry, complete semantic context/request binding,
immutable dispatch, and predecessor projection behind an explicit private
injection boundary. T02-W02 implements the exact registered foundation,
root-driven closed specialization, concrete expansion, and canonical
monomorphic values behind the same private boundary. T02-W03 adds the closed
operation/check, linear-construction, application-binding commutation,
control/pattern, and explicit-exception vocabulary behind that boundary.
T02-W04 implements the context-bound successor source artifacts, semantic
bindings, closed operation/check tables, immutable input capture, source map,
both manifest stages, and boundary byte/value linkage behind that boundary.
T02-W05 implements the strict, context-bound successor VIR importer and its
generic-free, resource, graph, dominance, ownership, exception, binding,
foundation, and operation validation barriers. T02-W06 implements the
context-bound successor VC and theorem-skeleton schemas, closed ordinary-term
encoding routes and later proof ownership, deterministic limits and hashes,
and the proof-empty, theory-empty, zero-axiom ordinary-context assembly
profile. T02-W07 implements the candidate-only successor frontend request/result
protocol, exact source/sidecar inventory, sanitized phase-ordered diagnostics,
artifact-free failure, and complete source-map/manifest/input-set lineage.
T02-W08 implements private, typed migration of all four predecessor producers,
regenerates the sole v2 artifact family with exact successor context and hashes,
and pins complete-corpus equivalence in four semantic-difference reports.
T02-W09 closes all 18 inventoried private consumer edges over the sole
successor family, the registered foundation and retained Certificate v0 bytes.
T03-W01 adds the private pinned-Roslyn capture, dependency/declaration/generic/
effect firewall, complete declaration accounting, and finite source call/type
closure. T03-W02 adds private concise-syntax, exact-type, and name-resolution
normalization over that closure. T03-W03 adds private immutable data declarations,
closed enums, type graphs, and recursive default eligibility. T03-W04 adds
constructor flow analysis, exact synthesized IL checks, receiver-first calls,
and pending invariant obligations. T03-W05 adds ordered init/required
transactions and explicit finalization. T03-W06 adds shared structural
equality and canonical ordering over concrete descriptors.
T03-W07 adds ordered array bounds, initialization and ownership plans.
T03-W08 adds typed sequence plans, wrapper projection and construction elimination.
T03-W09 adds ordered map/set projection and typed operation handoffs.
T03-W10 adds UTF-16 string plans and the shared typed boundary codec relation.
`CSHARP-03-T03-W11` is ready and all later implementation items remain serially blocked. No public production acceptance
path, installed candidate, or active registry entry was introduced.

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
| `CSHARP-03-T01-W09` | `Complete` | `crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W09` | `17525292755c4e508acd9300cfa72d20cdf9bb92` |
| `CSHARP-03-T01-W10` | `Complete` | `crates/mpk-vc/tests/csharp_practical_spec.rs#CSHARP-03-T01-W10` | `d4459f16562c9f5a7d4d0074571c9d0af17c0dd5` |
| `CSHARP-03-T02-W01` | `Complete` | `crates/mpk-vc/tests/csharp_practical_registry.rs#CSHARP-03-T02-W01` | `4a9e8afef62eaf54a8184119b4e62e50cb73de06` |
| `CSHARP-03-T02-W02` | `Complete` | `crates/mpk-vc/tests/csharp_practical_vir_model.rs#CSHARP-03-T02-W02` | `026243eae673672c45ed96d348b3248afcde40b5` |
| `CSHARP-03-T02-W03` | `Complete` | `crates/mpk-vc/tests/csharp_practical_vir_model.rs#CSHARP-03-T02-W03` | `cb2c2eb419adceaf84d4b610a19deb4b8205bf96` |
| `CSHARP-03-T02-W04` | `Complete` | `crates/mpk-vc/tests/csharp_practical_source_artifacts.rs#CSHARP-03-T02-W04` | `c487b098403c326d1802fe9ea6d43ff8e76e5b80` |
| `CSHARP-03-T02-W05` | `Complete` | `crates/mpk-vc/tests/csharp_practical_vir_validation.rs#CSHARP-03-T02-W05` | `816280dc5ea298f6b215c63ae7185f89f0a240a9` |
| `CSHARP-03-T02-W06` | `Complete` | `crates/mpk-vc/tests/csharp_practical_vc_model.rs#CSHARP-03-T02-W06` | `3a3da6af26e301f95692dd787a8920b58fafb566` |
| `CSHARP-03-T02-W07` | `Complete` | `crates/mpk-cli/tests/csharp_practical_frontend_protocol.rs#CSHARP-03-T02-W07` | `ed9054603eec5d410c59a708ee81d79052ee27d1` |
| `CSHARP-03-T02-W08` | `Complete` | `crates/mpk-cli/tests/csharp_practical_migration.rs#CSHARP-03-T02-W08` | `a9b2c862e9386ec25b7183b8b4c9b8db4cc426a6` |
| `CSHARP-03-T02-W09` | `Complete` | `crates/mpk-cli/tests/csharp_practical_migration.rs#CSHARP-03-T02-W09` | `0c4cd3bac346d68d72111305192c9766342ec68e` |
| `CSHARP-03-T03-W01` | `Complete` | `crates/mpk-cli/tests/csharp_practical_capture.rs#CSHARP-03-T03-W01` | `c16bca82c8f39e34decc09959a4e3b34c18bc12d` |
| `CSHARP-03-T03-W02` | `Complete` | `crates/mpk-cli/tests/csharp_practical_syntax.rs#CSHARP-03-T03-W02` | `773591e4ef0900eca84724a15be2d785449c51ad` |
| `CSHARP-03-T03-W03` | `Complete` | `crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W03` | `d96594ad4cf3db973d04fb4057784d2356de358d` |
| `CSHARP-03-T03-W04` | `Complete` | `crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W04` | `ba7223fe8238e47703c8da983aa6ba3d8fa73526` |
| `CSHARP-03-T03-W05` | `Complete` | `crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W05` | `381a20a54249e4a1f38e0edea6f4ce1ac9de3d50` |
| `CSHARP-03-T03-W06` | `Complete` | `crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W06` | `47bde686b1ffd209cf981d5614836e09b4ed9fe6` |
| `CSHARP-03-T03-W07` | `Complete` | `crates/mpk-cli/tests/csharp_practical_collections.rs#CSHARP-03-T03-W07` | `dc36a561b7b445d450374663af8a96aa63ebad5e` |
| `CSHARP-03-T03-W08` | `Complete` | `crates/mpk-cli/tests/csharp_practical_collections.rs#CSHARP-03-T03-W08` | `b2ecf2949c82bb6687b0916e292f4f4bfbcbdc86` |
| `CSHARP-03-T03-W09` | `Complete` | `crates/mpk-cli/tests/csharp_practical_collections.rs#CSHARP-03-T03-W09` | `6714e88e963fba6f06c050989072a2cedfeefedd` |
| `CSHARP-03-T03-W10` | `Complete` | `crates/mpk-cli/tests/csharp_practical_codecs.rs#CSHARP-03-T03-W10` | `SELF` |
| `CSHARP-03-T03-W11` | `Ready` | `crates/mpk-cli/tests/csharp_practical_numbers.rs#CSHARP-03-T03-W11` | `—` |
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

## 13. CSHARP-03-T01-W10 completion record

### 13.1 Published normative package

W10 consumes the complete W09 commit
`17525292755c4e508acd9300cfa72d20cdf9bb92` without changing its frozen values.
It publishes these normative but inactive members:

| Member | Bytes | Raw SHA-256 |
| --- | ---: | --- |
| `develop/specs/CSHARP_PRACTICAL_PROFILE_V1.md` | 15,287 | `a45d0fda3322f17148201504f19e620c389a65ca24f79b5cbed4fa6a9c29b28b` |
| `develop/specs/CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md` | 10,586 | `d1d6618dba572d24c3706d5cd88d08c6532fa6828c356803fd63ea2bc32395ab` |
| `develop/specs/vectors/csharp-practical-profile-v1.json` | 522,760 | `d3b7be7d09d79742c8f339db3ca6313808e0257c30476cf6959aa7c5231d1b16` |

The vector schema is `mpk.csharp.practical.profile.conformance.v1`. Its
`frozen_contract` is byte-value-equivalent to the W09 private freeze with
domain-separated content SHA-256
`f292de00a79048ecd1ff2cbe52d90fad36654f1b3e74ad580b5ec3077afa28cb`.
Its 700 sorted vector rows are exactly the W09 private rows. The package binds
all 16 canonical W01-W09 evidence records, all three specification members,
the publication generator, and the incorporated design section 6-through-23
projection. `develop/specs/vectors/manifest.json` registers the package with
the exact schema, owner, and raw hash above; its resulting 12,390 bytes have
raw SHA-256
`2f71697db72900b7bdbc2cb36b5f23db41865a06847a0b7ffc0b9e69ead6985b`.

The deterministic publication generator
`develop/probes/csharp-03/profile_package.py` is 28,890 bytes with raw SHA-256
`3b6e4e9849408d28bd6c0d6021fbd54d9aba0eb52848cd130f9d1feb3816f1ae`.
It freezes the publication-time predecessor gate hashes as data rather than
reading mutable post-activation paths, so `--check` remains reproducible after
the planned gate replacement.

The historical W02 inventory remains immutable. The package records its raw
hash and a closed three-path publication extension containing only the two new
specifications and the published vector. The W02 repository fingerprint owner
excludes only those three W10-owned paths; the W10 owner independently closes
their names, hashes, manifest registration, and non-activation. Any other new,
removed, or changed historical consumer remains a W02 fingerprint failure.

### 13.2 Complete owner and upgrade closure

The package maps each of the ten T01 freeze requirements to its exact work item
and specification/probe test owner. It records exactly 63 downstream work-item
owners for every T02-T08 implementation/release item, including each item's
exact title, `Owns`, exit-gate, verification, and plan anchor text, with only
T02-W01 ready and all later items serially blocked. Every implementation
surface and release criterion is therefore attached to a work item and its
primary production-test pair. Every one of the 700 vector rows resolves to one
of those same implementation items and owners.

The flattened inventory closes 243 unique identity/hash-domain names (102
successor identities, 42 successor domains, 88 retained identities, and 11
retained domains), 71 strict root/nested/union/expression shapes, 29 diagnostic
families, and 67 practical-plus-retained limits. It also binds four canonical
upgrade-observation sets containing 181, 65, 144, and 154 unique IDs and twelve
disjoint excluded-source families. No excluded family has a positive v1
vector. Future admission requires a new semantic-profile identity and a new
atomic release; the exact value-type `T?` exception is not widened.

### 13.3 Release-gate and non-activation decision

The future practical candidate and release invocation is exactly:

```text
sudo ./scripts/check-csharp-practical-release.sh
```

T07-W05 owns the private script, T07-W06 owns its receipt, and T07-W05/W06 plus
T08-W06/W09/W10 are the exact invocation owners. Before atomic activation,
`scripts/check-java-frontend.sh` remains the sole installed-release gate and
the practical script is absent. At T08-W10, the practical gate replaces and
retires the Java-named gate atomically; `scripts/check-all.sh` delegates only
to the practical gate. The replacement does not extend the installed gate by
running both names.

W10 adds no production source, public route, installed registry entry, bundle,
candidate executable, compatibility selector, application dependency, core or
checker rule, proof-node kind, theory certificate, axiom, or GitHub workflow.
The active semantic and bundle registries contain no practical identity.

### 13.4 Verification and review

Completed local verification:

| Command | Result |
| --- | --- |
| `python3 develop/probes/csharp-03/profile_package.py --check` | pass: 10 freeze owners, 63 downstream owners, 700 vectors |
| `python3 scripts/check-spec-vectors.py --check` | pass: 26 manifested vector sets |
| `python3 scripts/check-artifact-paths.py` | pass: 256 canonical JSON artifacts |
| `cargo test -p mpk-vc --test csharp_practical_spec --test csharp_practical_inventory` | pass: 20 owner/spec/inventory tests |
| `cargo test -p mpk-vc --test canonical_json` | pass: 8 canonical JSON and closed-manifest tests |
| `cargo fmt --all -- --check` | pass |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

First-pass review findings: `4`.

- The publication generator originally re-read the mutable installed Java and
  aggregate gate bytes. Their W10 predecessor hashes are now frozen constants,
  while the W10 non-activation test separately proves that the current files
  still match. The normative package therefore remains reproducible after the
  planned T08-W10 gate replacement.
- The W02 historical repository search originally trusted whichever exclusion
  paths the new package supplied. Its owner now independently requires exactly
  the two W10 specifications and one W10 vector path, so the extension cannot
  conceal another added consumer.
- The first package draft routed each of the 63 downstream items by title and
  anchor only. It now carries every item's normalized `Owns`, exit-gate, and
  verification contract as well as its exact primary production-test pair,
  closing implementation surfaces and release criteria rather than merely
  pointing at them.
- The task-contract normalizer initially inserted spaces into words split at a
  Markdown line-ending hyphen or slash. It now joins those continuations,
  owner tests require every field, the historical W09 record remains
  unchanged, and current charter/roadmap routing points to T02-W01.

Verification follow-up findings: `1`.

- The repository-wide canonical JSON test retained the predecessor manifest
  cardinality of 25 after W10 registered its one normative vector container.
  The independent closed-manifest assertion now requires exactly 26 entries;
  its focused test and the complete fast gate both pass.

Final review findings: `0`. The final pass rechecks the W09 value and
700-row identity, all source/spec/generator/manifest hashes, ten freeze and 63
downstream owner contracts, flattened name/schema/diagnostic/limit ownership,
upgrade exclusions, exact future gate replacement, historical inventory
closure, active-registry/gate non-activation, documentation routing, changed-
path scope, and deterministic regeneration. No production behavior changed.

## 14. CSHARP-03-T02-W01 completion record

### 14.1 Closed private successor registry

W01 adds `crates/mpk-vc/src/csharp_practical_registry.rs` as an explicitly
constructed, candidate-only semantic-registry implementation. It does not
replace the installed revision-3 registry, add a public CLI/API route, install
a release tuple, add a compatibility selector, read an ambient staging root,
or perform a dual-registry lookup. The implementation accepts only revision 4
under `mpk.semantic_profile.registry.v2` and requires exactly five entries in
canonical `(source_language, semantic_profile)` order: practical C#, retained
scalar C#, Go, Java, and Rust.

Every entry has the exact v2 entry schema, semantic-parameter and selection
schemas, registered foundation descriptor reference, nine compiled-contract
identities, and recomputed entry hash. The complete registry has the immutable
root SHA-256
`1cad5b32ce432eac39655240a84ec83ba6f347c335452b5e143fca3ba2cb78c8`.
Its exact entry hashes are:

| Entry | Entry SHA-256 |
| --- | --- |
| `csharp` / `mpk.csharp.practical.v1` | `9a5b4737e928a93dfa07f71e72d49181d32a84200e3e786fc3a8914914676661` |
| `csharp` / `mpk.csharp.scalar.v0` | `ff99f04464d3485f7239460da0562b8b812abbf25577bbb35ce05f07c5273bc3` |
| `go` / `mpk.go.fixed.v0` | `8fa92fb20f37a0aef96f496d68b8d6d62370be0ea25fb4590aa4bba716d0d986` |
| `java` / `mpk.java.scalar.v0` | `cf6a4b2432a15f89196d0469ef67729d2d9d9a97dd5596ed48c43b905fa6fd51` |
| `rust` / `mpk.rust.checked.v0` | `a224764969f554caadf8b205a9a5f34db833dbb622d306ba048fc6d854725c75` |

The practical entry pins descriptor schema
`mpk.csharp.foundation_descriptor.v1`, descriptor identity
`mpk.csharp.practical.foundation.v1`, and content SHA-256
`d8c2a023f1c445470123519f5024a17aaca1766553331a2fed4733fecf7deec1`.
The same validated reference is carried by the successor context and request;
callers cannot substitute a descriptor or allowlist.

### 14.2 Context, selection, contracts, and limits

The production validator implements strict duplicate-key-preserving JSON
transport checks, LF termination, exact shape/type/unknown-field rejection,
canonical serialization, domain-separated parameter/selection/context/request/
contract hashes, immutable profile dispatch, and complete equality linkage
between the registry entry, semantic context, validated request, selection,
foundation reference, and compiled envelope. C# dispatch distinguishes scalar
and practical profiles; all other languages have no C# frontend dispatch arm.

The W10 package assigns exactly 50 rows to this work item. The primary owner
executes all 44 schema and 6 context vectors through production validation,
including each missing field, each wrong type, unknown and duplicate fields,
later-schema inputs, crossed known identities, and independently rehashed
mutations. It also checks the full five-profile by nine-contract matrix (45
immutable contract identities), registered parameter and selection shapes for
all five profiles, context/request/hash linkage, and rejection of unknown or
mixed language/profile pairs.

All ten inherited registry/common counters are closed with below, inclusive-
maximum, and above-limit tests: canonical registry bytes, transport bytes,
JSON nesting, identifier bytes, source-language bytes, profile count,
parameter bytes, selection bytes, compiled-profile bytes, and revision. Hash
helpers apply the owning family limit rather than a wider common allocation
ceiling.

### 14.3 Append-only predecessor and non-activation proof

Revision-1, revision-2, and revision-3 published registries continue to pass
their original validators and reject under the candidate validator. Candidate
bytes reject under every predecessor parser. Only the installed revision-3
registry can be projected into the candidate: the projection retains every
predecessor profile, parameter/selection schema, and compiled-contract identity
unchanged, then appends the practical entry. Revision 1 or 2, partial,
reordered, unknown, and mixed candidates cannot project.

Projection source: installed revision 3 only.

The active registry resolver still reports revision 3 and exposes only its
four released profiles. The W10 normative package and its three publication
paths remain byte-identical; the historical W02 consumer fingerprint adds
only the exact W01 production module and primary test to its post-freeze
exclusion set. Thus the new implementation is reachable only by explicit test
construction and does not alter installed selection or release behavior.

### 14.4 Verification and review

Completed local verification:

| Command | Result |
| --- | --- |
| `python3 develop/probes/csharp-03/profile_package.py --check` | pass: 10 freeze owners, 63 downstream owners, 700 vectors |
| `cargo test -p mpk-vc --test csharp_practical_registry --test semantic_profile_registry --test semantic_profile_registry_runtime` | pass: 17 registry/context/predecessor tests |
| `cargo test -p mpk-cli --test successor_atomic_cutover` | pass |
| `cargo test -p mpk-vc --test csharp_practical_inventory` | pass: 4 ledger/inventory tests |
| `cargo clippy -p mpk-vc --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

First-pass review findings: `3`.

- The first schema-fragment validator admitted later context/request schemas
  and generic unregistered parameter envelopes. Dispatch now selects the exact
  registered schema and its closed validator before accepting a fragment.
- Entry ordering could originally be checked before a malformed entry's exact
  shape, changing error precedence. Each entry now completes strict shape and
  type validation before canonical ordering is evaluated.
- Identifier syntax and hash-input ceilings were initially broader than their
  predecessor contracts. Registry identifiers now use the retained lowercase
  grammar, selected callable roots use their separately bounded canonical-ID
  grammar, and each hash helper enforces its own family limit.

Verification follow-up findings: `6`.

- Request/selection coverage initially exercised the practical entry and all
  five contexts but did not validate a complete request for each retained
  selection schema. The owner now builds and validates all five profile
  requests before checking the complete contract matrix.
- The practical selection initially applied only local component checks to
  normalized paths and accepted arbitrary printable root IDs. It now reuses
  the existing portable normalized-path predicate and requires the frozen
  domain-separated `mpk.csharp.source.<64 lowercase hex>` callable identity;
  repaired-hash mutations cover both former escapes. Request outer-shape
  validation is separate from registered selection validation, so these
  failures reach the selection-owned phase instead of collapsing to shape.
- The candidate module must be exported so its integration-test owner can call
  production code, but that export was initially visible in generated API
  documentation. It is now explicitly Rustdoc-hidden and remains unreachable
  from every installed CLI/API/release route.
- Context shape validation initially performed registered-value checks before
  the context-owned binding phases, making the later parameter-value and
  foundation errors unreachable for malformed values. Outer JSON typing is
  now separated from full standalone-document validation; repaired context and
  request hashes reach their owning parameter, foundation, and hash phases.

Final review findings: `0`. The final pass rechecks task-only scope; all 50
published W01 vectors; exact schemas, identities, domains, hashes, ordering,
limits, context and compiled-envelope linkage; the five-by-nine dispatch;
revision-1/2/3 preservation and revision-3-only projection; active-release
non-activation; historical-inventory closure; formatting, lint, focused
regressions, and the complete local fast gate.

## 15. CSHARP-03-T02-W02 completion record

### 15.1 Registered foundation and root-driven specialization

W02 adds `crates/mpk-vc/src/csharp_practical_vir_model.rs` as a Rustdoc-hidden,
explicitly invoked implementation. It accepts only the exact registered
`mpk.csharp.foundation_descriptor.v1` object and exact definition member,
recomputes the descriptor under `MPK-CSHARP-PRACTICAL-FOUNDATION-1.0`, and
requires content SHA-256
`d8c2a023f1c445470123519f5024a17aaca1766553331a2fed4733fecf7deec1`.
It also independently checks the raw definition and semantics members at
SHA-256
`25738447bf793e37dc2125e7a07da55a03fb15f2fa4dfb87b25646a16cc9d1b4`
and
`29c5986e3c7ce2ab018e36eea61caaf9d9e53d6b8e47f0229ef4681db8c3fc8b`.
Changed descriptors, member bodies, missing/extra fields, duplicate keys,
floating JSON, non-finite tokens, and non-canonical transport reject before
specialization.

The validated inventory contains exactly all twelve templates and four
non-template definitions. The closed-root input accepts only the frozen closed
type grammar, registered template arities, admitted derivation origins,
canonical provenance IDs, and an acyclic source-value graph. User parameters,
unknown templates/types, nested option, construction/exception data payloads,
non-total keys, invalid currency types, duplicate roots, and over-depth graphs
reject. Roots remain explicit private inputs: W02 does not infer source facts,
trust caller allowlists, or expose a public application route.

The specialization engine recursively collects nested arguments and ordered
dependency recipes, derives domain-separated concrete instance IDs, unions and
sorts provenance, emits entries in lexical instance-ID order, retains recipe
order for `dependencyN`, deduplicates serialized dependency IDs, removes only
unavailable compare operations, substitutes every parameter/reference with a
concrete ID, and recomputes every entry, counter, operation body, and the whole
closed-set hash. The all-template fixture deterministically produces 13 closed
instances, 83 operations, and 863 recipe nodes. Full-object recomputation
rejects omitted, duplicated, reordered, unreachable/caller-injected, stale,
or residual-generic entries rather than trusting a submitted hash.

### 15.2 Canonical monomorphic values and limits

The same private module provides closed typed representations for unit,
Boolean and fixed-width integer values; chars and bounded UTF-16 strings;
binary32/binary64 bits and decimal sign/scale/coefficient; source and
`DayOfWeek` enums; immutable source products; arrays and bounded sequences;
ordered entries/maps/sets; option, lookup, result, validation and boundary
arms; date/time/duration/internal instant and GUID; money and transition
values; all parse-error arms; and built-in/source closed exceptions. Each
representation validates its exact concrete type ID, active payload shape,
field order, carrier range and family bound before canonical encoding.

Ordered map keys and set elements are strictly increasing under the frozen
semantic order, not JSON lexical order. This includes signed integers,
decimal value equivalence (including scale and signed zero), UTF-16 strings,
enum carriers, structural values, tagged arms, GUID N-field order, money's
currency-before-amount order, and transition field order. Equal or descending
adjacent values reject. Nested failures preserve their owning validation code,
and recursively counted live value cells reject above 65,536.

The primary owner performs canonical encode/import/re-encode round trips for
every listed value family and verifies family bounds, semantic ordering,
wrong nested values, non-canonical transport, and attempted generic-value
injection. No value transport contains a template application or parameter,
and no caller-defined framework or serializer representation is accepted.

### 15.3 Verification and review

Completed local verification:

| Command | Result |
| --- | --- |
| `python3 develop/probes/csharp-03/foundation_package.py --check` | pass: exact 12-template/4-non-template package and 2,051 vectors |
| `python3 develop/probes/csharp-03/profile_package.py --check` | pass: 10 freeze owners, 63 downstream owners, 700 vectors |
| `cargo test -p mpk-vc --test csharp_practical_vir_model` | pass: 19 descriptor, 57 specialization, and 12 shared limit vectors plus all canonical value families |
| `cargo test -p mpk-vc --test csharp_practical_registry --test semantic_profile_registry --test semantic_profile_registry_runtime` | pass: successor and revision-1/2/3 registry regressions |
| `cargo test -p mpk-vc --test csharp_practical_inventory` | pass: ledger state and historical consumer fingerprints |
| `cargo test -p mpk-cli --test successor_atomic_cutover` | pass: candidate remains unavailable to installed routes |
| `cargo clippy -p mpk-vc --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

First-pass review findings: `6`.

- The definition member was byte-checked but its paired semantics member was
  represented only by descriptor constants. The registered semantics bytes
  are now embedded and independently checked for exact size and raw digest.
- Nested ordered-entry/map, money, and transition validation converted a child
  invariant/type failure into a total-cell failure. Child results are now
  propagated before checked cell addition, with a regression assertion on the
  exact error.
- The general product arm admitted specialized ordered-entry/money/transition
  IDs and could bypass their dedicated type relations. General products are
  now limited to exact captured source products; each specialized family uses
  its closed representation.
- Ordered maps and sets initially checked only duplicate serialized values.
  They now require strict frozen semantic ordering, including numerically
  ordered multi-digit integers and value-equivalent decimal encodings.
- The concrete enum arm initially omitted the frozen framework-owned
  `DayOfWeek` carrier. It now accepts exactly i32 carriers 0 through 6 while
  retaining declaration-bound checks for source enums.
- Canonical concrete-value coverage initially stopped at the generated
  specialization object. The owner now round-trips every required family and
  covers per-family bounds, total cells, generic injection, and canonical-byte
  rejection.

Verification follow-up findings: `2`.

- The nullable-lookup vector intentionally has `inputs: null` and carries its
  positive type in `expected.value`; the first harness read the wrong field.
  It now validates the frozen positive type itself.
- Direct structural-limit vectors distinguish `closed_instance_count/depth`
  from closure's `instance_count/depth` errors. Separate frozen error values
  now preserve both contracts without changing specialization precedence.

Final review findings: `0`. The final pass rechecks W02-only production scope;
the exact descriptor/member bytes and hashes; all template identities,
arities, dependencies, operations and expansions; root/source/type closure;
identity/order/dedup/provenance and all counters; whole-object closed-set
recomputation; complete monomorphic value typing, canonical ordering and
round trips; generic-free output; predecessor preservation; non-activation;
historical inventory closure; formatting, lint, focused regressions, and the
complete local fast gate. At that W02 handoff, T02-W03 became the sole ready
item.

## 16. CSHARP-03-T02-W03 completion record

### 16.1 Closed operation and check vocabulary

W03 extends the Rustdoc-hidden, explicitly invoked
`crates/mpk-vc/src/csharp_practical_vir_model.rs` implementation. It defines
the exact `mpk.csharp.operations.v1` and `mpk.csharp.required_checks.v1`
identities and closed Rust enums for 14 operation kinds and four required-check
kinds. String-to-tag conversion rejects every unknown value; the vocabulary
contains no iterator, `yield`, async, await, task, scheduler, suspension, or
continuation state.

Operation signatures admit only primitive, captured source, or derived closed
instance type IDs. Registered-foundation operations are looked up from W02's
recomputed concrete expansion and must repeat its exact operands, result, and
check precedence. Frozen boundary codec names, parser precedence, formatter
output-bound obligation, decimal/floating/string/date/time/duration/GUID/
instant and lifted-data operation families, structural comparison, binding,
and exception operation shapes have closed tags and reject unknown IDs.
Invocation validation repeats the exact operand/result types and check order,
requires one normal successor, and pairs each exception check in order with
its concrete exception type and explicit exceptional successor.

This is the in-memory typed vocabulary only. W03 does not create a serialized
operation/check artifact, canonical hash domain, source manifest member, or
public route. T02-W04 remains the sole owner of the concrete serialized
operation/check tables and their complete source/context/artifact linkage.

### 16.2 Linear construction and binding commutation

The derived monomorphic `sequence_construction<T>` metadata determines the
exact element and published `bounded_sequence<T>` types. Its non-storable
linear state records construction identity, unique owner, version, length,
publication-role bound, definitely initialized bitmap, optional immutable
borrow, and active/frozen/discarded status. Allocation distinguishes
default-eligible and initially uninitialized storage; negative or over-16,384
lengths reject. First fill, complete-only rewrite/borrow/transfer/freeze,
read authorization, exact borrow end, old-owner invalidation, partial discard,
publication bounds, and post-terminal-use rejection are explicit transitions.
Branch merge requires identical instance, owner, version, lifetime, borrower,
length, publication bound, and status, then intersects definite-initialization
sets.

Application-binding projections are paired unary source-to-semantic and
semantic-to-source operations over concrete types. Operation commutation
checks receiver-first operand projections, result projection, source versus
semantic signature shape, ordered normal/error/exception branches, and any
required failure projection. Duplicate projection or projection-operation IDs,
type disagreement, reordered checks, and a diagram that does not use its
declared receiver binding reject structurally. W04 supplies the context-bound
binding records; W06 later proves the universal round trips and commuting
diagrams rather than trusting this structural check as an assertion.

### 16.3 Explicit control, patterns, and exceptions

W03 adds five abrupt-completion kinds, 15 control-node kinds, nine
construction actions, and 14 pattern kinds. The graph validator checks one
ordered entry and exit, concrete branch/return types, explicit normal and
exceptional edges, reducible loop headers/backedges and canonical loop IDs,
single governing-value evaluation, lexical guard/arm order, exhaustive versus
explicit switch-exception behavior, total pure property access, finite sealed
type patterns, and bounded slice-free list patterns. Catch-all patterns must be
last, exhaustive, and unguarded.

The closed exception universe fixes the nine frozen built-in tags and appends
only sorted captured sealed direct `System.Exception` source types with their
exact immutable payload members. Broad `catch`, `catch (Exception)`,
`SystemException`, and resource/runtime exceptions remain outside the admitted
catch vocabulary. Typed catches preserve lexical shadowing rules, filters are
Boolean and preserve the original search exception when filter evaluation
throws, and handler entry identities are distinct. Rethrow retains its exact
active-catch identity. Finally completion preserves its incoming abrupt value
on normal completion, replaces it on throw, and rejects source return/break/
continue from the finally body.

Every exceptional successor has exactly one unwind plan keyed by source node
and check ID. Plans agree with the source region stack, select only a compatible
closed catch, execute exited finally regions inner-to-outer, and connect the
edge to the first finally or final handler. An uncaught exception instead
terminates at the method exit after all enclosing finally regions. This covers
exceptional operation edges, explicit throw/rethrow, and a non-exhaustive
switch edge; it is not limited to explicit throw nodes and does not require a
synthetic handler region at method root.

The implementation is pinned to the already frozen evidence without changing
it: Roslyn control/exception/pattern record SHA-256
`b1215ad7f4a0e08dc269834229d7158158d31c0e9475218fa0791feea5a1629a`,
W07 runtime record SHA-256
`0055835ce456fb9c438336332bc0e2a214d900c137eca34f90c3fcddd2688769`,
W08 runtime record SHA-256
`6ef1194e1398d5822c676248ea6ccbbb31381b95cfd32c8b8a65e68376118064`,
foundation definition SHA-256
`25738447bf793e37dc2125e7a07da55a03fb15f2fa4dfb87b25646a16cc9d1b4`,
foundation semantic specification SHA-256
`29c5986e3c7ce2ab018e36eea61caaf9d9e53d6b8e47f0229ef4681db8c3fc8b`,
and freeze record SHA-256
`83954067c156e58cb349dbf07da44edf60a3ec550e628e6d2f1a890889d574e3`.
No C# compiler or runtime is invoked by the W03 owner test.

### 16.4 Verification and review

Completed local verification:

| Command | Result |
| --- | --- |
| `python3 develop/probes/csharp-03/foundation_package.py --check` | pass: exact registered foundation and 2,051 vectors |
| `python3 develop/probes/csharp-03/profile_package.py --check` | pass: 10 freeze owners, 63 downstream owners, 700 vectors |
| `cargo test -p mpk-vc --test csharp_practical_vir_model` | pass: all W02 regressions and W03 closed-tag, operation, binding, construction, pattern/control, handler, unwind, and mutation cases |
| `cargo test -p mpk-vc --test csharp_practical_registry --test semantic_profile_registry --test semantic_profile_registry_runtime` | pass: successor and revision-1/2/3 registry regressions |
| `cargo test -p mpk-vc --test csharp_practical_inventory` | pass: ledger state and historical consumer fingerprints |
| `cargo test -p mpk-cli --test successor_atomic_cutover` | pass: candidate remains unavailable to installed routes |
| `cargo clippy -p mpk-vc --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

First-pass review findings: `11`.

- Codec/data operation admission initially used open prefixes and omitted the
  frozen lifted families. It now uses exact codec/data carrier and operation
  sets; unknown suffixes reject.
- Partial discard initially changed the initialization bitmap, and immutable
  borrow/transfer initially admitted a partial construction. Discard now
  publishes nothing and preserves the partial state; borrow and transfer
  require completeness.
- Construction publication initially lacked the distinct target-role bound.
  State now carries that bound independently of allocation capacity and checks
  it at freeze and merge.
- A directly reconstructed frozen construction state initially did not repeat
  `length <= publication_length_maximum`. State validation now rejects this
  mutation independently of the transition that produced it.
- Binding diagrams initially ignored duplicate operation IDs and whether the
  declared binding owned the receiver projection. Both identities and the
  receiver-first relation are now checked.
- The exception-universe arm vector was initially caller-mutable. Its field is
  now private and only the checked constructor can create a universe.
- Broad `Exception`/`SystemException` catches were initially admitted even
  though they include runtime/resource exceptions outside the closed sum.
  Catch types must now be exact universe arms while exact admitted ancestor
  arms such as `ArgumentException` retain their closed hierarchy behavior.
- Control validation initially required different exception types to target
  different blocks. It now distinguishes edges by check ID and permits several
  exact exceptions to enter one handler.
- Unwind records initially covered only explicit throw/rethrow nodes, required
  a handler region even for a method-root throw, and were not completely tied
  to source edges. They now cover every exceptional edge exactly once and
  support operation, pattern, caught, and uncaught paths.
- Entry/exit, loop-header inventory, pattern-local loop metadata, guarded
  catch-all, exceptional-check identity, and handler-entry uniqueness were
  initially underconstrained. Exact cross-table checks and mutations now cover
  each case.
- Construction and control mutations initially missed publication overflow,
  partial ownership transfer, duplicate handler/check identities, broad catch,
  shared handler targets, and root uncaught propagation. Each is now an owner
  regression case.

Verification follow-up findings: `3`.

- The transfer-guard edit omitted one opening delimiter; formatting rejected
  the module before compilation. The delimiter is restored.
- The strengthened unwind comparison consumed its expected-region vector
  before checking the first destination. It now compares by borrowed iterator
  and retains the vector for edge validation.
- Raising the recorded first-pass finding count after the final construction-
  state review initially left the inventory owner's exact ledger assertion at
  the old count. The assertion now tracks the reviewed count and the complete
  fast gate passes with the ledger and owner test in agreement.

Final review findings: `0`. The final pass rechecks W03-only private scope;
closed tag and operation sets; concrete operands/results and ordered checks;
normal and exceptional successors; construction ownership, initialization,
publication and merge behavior; binding commutation; loop/pattern structure;
closed exception values, catches, filters, rethrow, finally and all unwind
paths; absence of iterator/async/scheduler vocabulary; W04 artifact-table
ownership; predecessor preservation; non-activation; documentation state;
formatting, lint, focused regressions, and the complete local fast gate.
T02-W04 is the sole ready item.

## 17. CSHARP-03-T02-W04 completion record

### 17.1 Canonical artifacts and closed context linkage

W04 adds the Rustdoc-hidden, explicitly invoked
`crates/mpk-vc/src/csharp_practical_source_artifacts.rs` implementation and its
sole owner test. The implementation preserves the practical schemas' frozen
field order rather than applying lexicographic object-key sorting. Its strict
JSON transport rejects duplicate, missing, unknown, reordered, wrong-typed,
noncanonical, over-16-MiB, and over-128-level inputs; retains the full unsigned
64-bit range; writes every U+0000 through U+001F code point with the frozen
lowercase `\uXXXX` spelling; and preserves lone UTF-16 surrogates in that same
form while requiring shortest UTF-8 for valid surrogate pairs.

A `PracticalArtifactContext` can be created only from a W01-validated practical
semantic request and the W02-validated registered foundation. It retains the
full semantic context, compilation identity, selection hash, normalized source
and sidecar paths, selected roots, foundation identity/content hash, and a
canonical private linkage key. Artifact references have private fields and are
created only from a validated artifact or from one of the exact opaque
successor VIR, VC, and certificate-skeleton identities owned by later tasks.
Every built artifact repeats and validates the same context, compilation,
selection, foundation, schema, and predecessor linkage. W04 introduces no new
hash domain beyond the T01-W09 frozen inventory and does not install or expose
a candidate route.

Context equality alone is not accepted as artifact lineage. Validated private
references retain their captured input-set identity and source artifact body;
the manifest builder rechecks the exact operation/check/closed-instance,
source-map/VIR, boundary-capture/contract, selection, and input-snapshot links
before it serializes any reference.

The strict contract validator covers the frozen selection, type, method,
semantic-binding, boundary-contract, and transition-contract roots. Boundary
input and output references deliberately cannot pass through that general
path: only their retained-byte/value builders can create them, preventing a
caller from asserting boundary evidence without the original transport or
reparsed result.

### 17.2 Complete source, binding, specialization, and operation accounting

Original-input capture requires exactly every selected source and sidecar path
once, normalizes each relative path, owns the unchanged bytes, records its raw
SHA-256, and invokes the existing `MPK-INPUT-SET-0.1` input-set algorithm.
Semantic binding entries preserve W08's exact eleven fields: source type/hash,
role-specific member and tag maps, ordered inferred arguments, exact default,
fixed bound, operation map, and binding hash. Entry hashes retain W08's sorted-
object canonical preimage while the enclosing context/compilation-bound set
uses the successor schema-ordered form. Duplicate source types, unknown roles
or operations, noncanonical/colliding tag carriers, wrong role members,
defaults, argument arities, or bounds, and more than 128 entries reject. Source
types, stored members, concrete arguments, and mapped source callables must use
their exact frozen canonical ID families; all twelve published W08 positive
binding specimens pass the production artifact validator.

The closed-instance artifact does not trust a supplied table. It invokes W02's
whole-set validator against the supplied root/provenance set and closed bytes,
then requires every instance's provenance identity to name one of those roots.
The concrete operation/check builders invoke W03 signature validation, require
exactly every operation expanded by that registered closed set once, reject
extra or conflicting definitions, deduplicate identical checks, and hash-link
the check table, operation table, closed instances, foundation, and practical
context. No template, constructed generic, caller allowlist, or unregistered
foundation value is emitted.

### 17.3 Source maps, manifests, and boundary evidence

The source map reuses the W08 declaration and stored-member identity builders,
requires every supplied reachable identity and provenance label once, rejects
duplicate VIR node IDs, checks selected-root coverage and captured-file
ordinals, and bounds every byte span by its immutable input. A declaration ID
hashes only its logical signature. Its separate provenance hash uses exactly
the declaration ID, normalized path, captured source hash, byte start, and byte
length; compilation and caller labels cannot alter either frozen preimage.
Declaration provenance and the complete map are then hashed under only their
frozen domains.

The frontend manifest links the exact selection, input set, type/method/
semantic/boundary/transition artifacts, registered foundation, derived closed
instances, concrete operations/checks, successor VIR, and source map. The
certificate manifest extends that exact frontend manifest with its successor
VC and certificate-skeleton references. The final
`mpk.frontend.source_artifacts.v2` builder reopens the validated frontend
manifest and requires every repeated reference, context member, selection,
binding, map, foundation, and boundary/transition member to agree before
computing the root hash.

Boundary input capture retains the exact original adapter/input bytes, their
provenance, raw hash, and size separately from the canonical verification
document. Only the latter is parsed into the typed value and supplies the
canonical document/value hashes; both identities enter the enclosing capture
hash. Boundary output capture canonicalizes
the typed source value, reparses it, requires field-complete equality, and
links the source value, reparsed value, canonical output bytes, and all hashes.
Dedicated validators rebuild each complete capture from the retained evidence;
byte, value, output, context, contract, or hash substitution rejects.

### 17.4 Verification and review

Completed local verification:

| Command | Result |
| --- | --- |
| `python3 develop/probes/csharp-03/foundation_package.py --check` | pass: exact registered foundation and 2,051 vectors |
| `python3 develop/probes/csharp-03/profile_package.py --check` | pass: 10 freeze owners, 63 downstream owners, and 700 vectors |
| `cargo test -p mpk-vc --test csharp_practical_source_artifacts` | pass: all six W04 owner tests, including all 16 frozen schema vectors and all 12 published positive W08 binding specimens |
| `cargo test -p mpk-vc` | pass: complete `mpk-vc` regression suite |
| `cargo test -p mpk-cli --test successor_atomic_cutover` | pass: candidate remains unavailable to installed routes |
| `cargo clippy -p mpk-vc --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

First-pass review findings: `12`.

- Generic validation initially allowed boundary-input/output artifacts without
  retained raw/reparsed evidence. Those schemas now reject on the general path
  and can be constructed only by the dedicated evidence builders.
- Closed-instance linkage initially checked provenance membership without
  rederiving the complete set from the supplied roots. It now invokes W02's
  independent whole-set validator before creating the artifact.
- Original-input capture initially recomposed the frozen input-set domain with
  a new preimage. It now calls the existing exact `MPK-INPUT-SET-0.1`
  implementation.
- Semantic-binding root validation initially underchecked nested members, arm
  mappings, bounds, dependencies, and type arguments. Each nested record now
  has an exact field/type/order/identity validator.
- The first context linkage used an implementation-private hash-domain name.
  It now uses the canonical bytes of the complete validated linkage tuple, so
  every emitted hash domain is present in the frozen inventory.
- Source-map declarations initially trusted caller-supplied declaration IDs.
  They now repeat the full declaration identity and independently recompute its
  frozen-domain hash.
- Source-content linkage initially accepted the raw hash of any captured input,
  including a sidecar. Type, method, and semantic-binding artifacts now require
  that the digest names captured source bytes; an exact sidecar-hash
  substitution is a regression case.
- Canonical source names and namespaces initially reused the wider internal-ID
  character set. They now independently enforce the scalar C# ASCII identifier
  grammar and dot-separated namespace components; punctuation aliases and
  empty namespace components reject.
- Canonical JSON strings initially used only Rust UTF-8 `String`, which cannot
  retain a lone UTF-16 surrogate. A bounded reversible parse path now carries
  exact UTF-16 units, emits lone surrogates as lowercase `\uXXXX`, and rejects
  escaped valid pairs because their canonical form is shortest UTF-8.
- Canonical output initially checked the transport limit after appending a
  complete string. Both UTF-8 scalar and UTF-16-unit writers now stop at the
  first character that exceeds 16 MiB, including object-key output.
- Individual semantic bindings initially used an implementation-invented
  context, compilation, binding ID, version, dependency, and array-record
  shape. They now use exactly W08's eleven fields and role inventories,
  preserve ordered (including equal) inferred arguments, and reproduce the
  published sorted-object binding hash inside the schema-ordered successor
  root.
- Canonical declaration IDs initially included the compilation ID and source-
  map provenance initially hashed a caller label and ordinal-bearing location.
  The source map now reuses the published logical declaration/stored-member ID
  builders and hashes the exact path/source-hash/start/length provenance tuple.

Verification follow-up findings: `14`.

- The first aggregate frozen-vector helper inverted the remove-each and
  wrong-type result aggregation. It now accepts the vector only when every
  required mutation rejects with the expected code.
- Clippy found an over-wide manifest-builder argument list. Related links now
  travel in one typed `FrontendSourceArtifactLinks` record.
- Clippy found an owned-vector API where a borrowed slice is sufficient. The
  reference-normalization helper now accepts a slice.
- Historical T01-W02 repository fingerprints initially classified the two new
  post-freeze W04 owner paths as frozen-input drift. The inventory owner now
  excludes those exact implementation paths, while the frozen inventory bytes
  and fingerprints remain unchanged.
- The first ledger-evidence assertion spanned a Markdown line break and could
  not match the intended sentence. It now checks a stable contiguous phrase
  while retaining the exact W04 evidence requirement.
- The source-only capture helper rename initially left one builder precheck at
  the former method name. Compilation caught it; both construction and import
  paths now invoke the same source-only predicate.
- Two broad count-only documentation edits temporarily matched earlier W01/W02
  completion records. The final ledger review restored those immutable counts
  and scoped the W04 counters to section 17.
- Semantic-binding fields initially accepted generic vocabulary strings where
  W08 requires canonical source-declaration, stored-member, concrete-type, and
  source-callable ID families. The structural validator now enforces those
  families, and the owner test imports all twelve published positive bindings.
- Ordered-unique contract-member arrays initially rejected only adjacent
  duplicates. Their validator now rejects a repeated member at any position,
  with a nonadjacent regression mutation.
- Boundary input capture initially treated original adapter bytes as the
  canonical verification document. It now retains and hashes them separately,
  parses only the independently supplied canonical document, and detects a
  substitution on either side.
- The manifest builder initially relied on semantic-context equality alone, so
  artifacts from another input snapshot, VIR, closed-instance/check table, or
  boundary contract in the same context could be combined. Private references
  now retain lineage and every cross-artifact edge is reopened and compared.
- Frontend-manifest input rows initially renamed the retained input-set fields
  and represented a verification-overlay sidecar as `sidecar`, even though the
  hash used the unchanged `MPK-INPUT-SET-0.1` `contract` row. The emitted rows
  now use the exact retained `kind`, `normalized_path`, `size_bytes`, and
  `sha256` preimage, and the owner test independently recomputes the manifest
  hash from those rows.
- After the retained row shape was restored, input capture still ordered rows
  by kind before path, while the retained source-manifest validator orders by
  normalized path before kind. Capture and its regression assertion now use
  that exact canonical order, so the emitted preimage is directly importable.
- Declaration identities, contracts, source-map nodes, and boundary captures
  initially admitted broad printable strings in fields whose frozen types are
  compilation IDs, canonical source IDs, canonical member IDs, provenance IDs,
  or schema-owned canonical IDs. Each field now enforces its exact family and
  grammar before hashing or linkage, with malformed-family and malformed-
  grammar regressions.

Final review findings: `0`. The final pass rechecks W04-only private scope;
canonical field order and JSON tokens; exact frozen identities and domains;
context, selection, compilation, source, foundation, and schema isolation;
complete source/input/declaration/provenance accounting; closed-set
rederivation; operation/check completeness; boundary byte/value/reparse
linkage; source-map and two-manifest chains; generic-free output; predecessor
preservation; non-activation; formatting, lint, focused mutations, all
`mpk-vc` regressions, and the complete local fast gate. T02-W05 is the sole
ready item.

## 18. CSHARP-03-T02-W05 completion record

### 18.1 Strict successor transport and immutable authority

W05 adds the strict, Rustdoc-hidden `mpk.vir.v2` importer in
`crates/mpk-vc/src/csharp_practical_vir_validation.rs` and its sole owner test.
The module is available only through explicit internal invocation; no installed
profile, bundle, CLI/API route, runtime dependency, or public acceptance path
changes. The successor parser checks the frozen input/canonical-byte, nesting,
string, node, function, block, operation, construction, loop, pattern,
exception, binding, parameter, call-argument, phi-incoming, and CFG-edge
limits before VC construction. Duplicate, unknown, missing, reordered,
noncanonical, wrong-family, and old-schema inputs reject. The predecessor v0
and v1 parsers continue to reject v2, and the v2 parser rejects predecessor
bytes.

The canonical root binds the exact semantic-context bytes, compilation and
selection identities, captured-input snapshot hash, registered foundation
descriptor, closed-instance artifact, semantic-binding artifact, required-
check table, operation table, source-type closure, expanded foundation,
exception declarations, binding obligations, functions, and `MPK-VIR-2.0`
hash. The import context accepts only W01/W04 validated private capabilities
and immutable transports. It independently revalidates the registered
foundation bytes and content hash, validates the supplied roots, reconstructs
the complete closed-instance closure, rebinds it to W04 capture provenance,
and recomputes dependency order, identities, deduplication, resource limits,
source-type DAG, and concrete expansion. A supplied table cannot become
authority merely by repeating a valid digest.

The generic barrier runs on the strictly scanned untyped tree before typed
shape decoding. Type parameters, generic definitions, constructed generics,
generic calls, semantic-template IDs, and surviving template nodes reject;
only the independently expanded monomorphic foundation and concrete value
types reach the validated result.

### 18.2 Complete graph, value, operation, and binding validation

Every function has one registered `source_call` signature with exact parameter
and result types. Selected roots and reachable callees are accounted for
exactly, the call graph and source-type graph are finite and acyclic, and all
function/block/value/node identifiers are globally stable and unique. The W03
control validator is supplemented with entry reachability, predecessor and
region-edge checks, dominator computation, ordered phi validation, and typed
use-site validation. An operation result must traverse its one dedicated
normal-successor edge before it becomes available; exceptional and bypass
paths cannot consume it. Return and abrupt-completion metadata must agree with
the function signature.

The importer reconstructs the required-check and operation tables from their
canonical W04 transports, validates every signature and hash link, and requires
the consumed operation set to equal the complete registered table. Ordinary
invocation cannot bypass the linear `sequence_construction` protocol.
Construction actions carry actual typed input/result value references, and
allocation, read, fill, rewrite, borrow, transfer, freeze, discard, merge, and
terminal-state rules are checked through W03's ownership machine. No active or
borrowed construction may escape a terminal.

Each handler-entry node defines exactly one typed handler-entry exception
value; declared catch/finally regions must equal the graph's handler/finally
entry set, closed catch ancestry and filter values are checked, and every
exceptional successor has a compatible unwind plan. Semantic bindings,
projection types, project/reconstruct operations, member operation maps, and
commutation obligations are re-read from the W04 artifact and matched exactly
against the VIR. Cross-context, cross-profile, cross-source-snapshot,
cross-foundation, cross-artifact, schema, and hash substitutions reject without
a compiler, Roslyn, CLR, filesystem, or runtime callback.

### 18.3 Verification and review

Completed local verification:

| Command | Result |
| --- | --- |
| `python3 develop/probes/csharp-03/foundation_package.py --check` | pass: exact registered foundation and 2,051 vectors |
| `python3 develop/probes/csharp-03/profile_package.py --check` | pass: 10 freeze owners, 63 downstream owners, and 700 vectors |
| `cargo test -p mpk-vc --test csharp_practical_vir_validation` | pass: all eight W05 owner tests, including bounded parser fuzz seeds and cross-artifact/profile/hash mutations |
| `cargo test -p mpk-vc` | pass: complete `mpk-vc` regression suite |
| `cargo test -p mpk-cli --test successor_atomic_cutover` | pass: candidate remains unavailable to installed routes |
| `cargo clippy -p mpk-vc --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

First-pass review findings: `12`.

- The first importer draft lacked a transport surface for several W03 models
  and a private successor-reference constructor. Exact deny-unknown Serde
  transports and the input-lineage-preserving constructor now form the only
  W05 bridge.
- Foundation and semantic-context data were initially inaccessible without
  duplicating authority. Narrow read-only accessors now reuse W01/W02/W04
  validation instead of creating a second registry path.
- Void return validation initially required one result value. Unit signatures
  now require zero result IDs and non-unit signatures require exactly one.
- The positive owner fixture initially exercised only an empty specialization
  closure. A registered `option<i32>` specimen now proves nonempty concrete
  expansion and absence of residual template/generic IDs.
- Operation use initially guaranteed only membership, allowing unused extra
  operation rows and functions without an exact source-call signature. Import
  now requires whole-table equality and signature/parameter/result agreement.
- Operation results initially became available by block dominance alone. Each
  result is now gated by its unique normal-successor edge, so an exception or
  bypass cannot observe it.
- The root initially relied on context equality without binding the immutable
  input capture. It now carries and validates the exact source snapshot hash,
  and its returned artifact reference retains that lineage.
- The nested semantic context initially decoded to an equal value even when
  its member order or string spelling was noncanonical. Its raw nested bytes
  must now equal W01's canonical context bytes exactly.
- Construction actions initially carried element types but not the actual
  values read, filled, rewritten, or frozen. Their typed value references are
  now registered and checked at each action ordinal.
- A caller could initially express a sequence-construction foundation
  operation as an ordinary invocation. Those operations now require the
  linear ownership action path.
- Handler entries initially named only a graph node. Each now owns one explicit
  closed exception carrier value available at handler entry.
- W03 exception-region validation initially allowed orphan handler/finally
  entry nodes. Region declarations must now exactly cover both graph entry
  sets, with a regression mutation in the W03 owner suite.

Verification follow-up findings: `11`.

- An early handler fixture used `System.Exception` and widened catch ancestry
  to admit it, contradicting the frozen closed-catch vocabulary. Catch types
  are again exact universe arms, the positive fixture uses the admitted
  `System.ArgumentException` arm, and ancestry still orders admitted arms.
- Retained per-function parameter, call-argument, block-parameter, phi,
  CFG-edge, loop-depth, pattern-arm, exception-region, and live-construction
  limits were initially incomplete. The strict observer and typed pass now
  cover the full W05 inventory and their limit-plus-one mutations.
- The first JSON-node cap introduced an unowned two-million-node limit below
  the frozen transport budget. It now derives from the retained maximum input
  bytes rather than inventing a stricter semantic boundary.
- The independently reconstructed closed set was initially compared to W04's
  set but not rebound to the exact captured source provenance. Import now
  invokes the W04 binding step and validates that rebound identity and order.
- The first exception resource pass bounded the number of try records rather
  than the frozen count of individual catch and finally regions. It now counts
  every catch and optional finally exactly once and rejects the 33rd region.
- Binding linkage initially stopped after artifact hashes, projection IDs, and
  operation commutation. The importer now independently checks that each bound
  type is reachable, source-owned and identity-insensitive; its source hash is
  exact; every mapped member belongs to that type; tag arms exhaust the actual
  enum carrier; and inferred closed arguments are rederived from the real
  member types. A valid instant projection and a wrong-carrier mutation cover
  this path.
- The resource-observer operation branch initially retained a collapsible
  match and formatting drift after aggregate-limit hardening. It now uses one
  guarded value arm, and the full target passes formatting and warning-denied
  Clippy checks.
- Ownership edge validation initially required every predecessor output to be
  byte-for-byte equal to the target input and therefore bypassed W03's
  compatible branch-merge rule. The importer now reconstructs predecessor
  merges through W03, intersects definite-initialization facts, and rejects a
  caller-supplied non-intersection state.
- The concrete-type and vocabulary predicates exposed for W05 linkage were
  initially made public even though no consumer outside `mpk-vc` needs them.
  Their visibility is now crate-local, preserving the private successor
  boundary.
- Explicit-control validation initially collapsed W03 exception failures into
  the generic control phase. W05 now preserves the exception phase at its
  boundary, with an importer-level broad-`System.Exception` rejection proving
  both the closed vocabulary and stable error classification.
- Four W03 in-memory-only construction/control helper types initially received
  transport derives despite never appearing in the W05 wire schema. Those
  derives are removed; only types actually nested in `mpk.vir.v2` gained a
  serialization surface.

Final review findings: `0`. The final pass rechecks W05-only private scope;
strict canonical transport and parser-family separation; exact immutable
context, input, foundation, specialization, binding, check, and operation
linkage; generic/template absence; graph finiteness, reachability, region and
dominance rules; normal-edge value availability; construction ownership and
terminal state; explicit exception values and unwind coverage; every retained
resource limit; predecessor preservation; non-activation; formatting, lint,
focused fuzz/mutation coverage, all `mpk-vc` regressions, and the complete
local fast gate. T02-W06 is the sole ready item.

## 19. CSHARP-03-T02-W06 completion record

### 19.1 Inputs, scope, and immutable linkage

W06 entered from clean `main` commit
`816280dc5ea298f6b215c63ae7185f89f0a240a9`, tree
`fc334ba157a62147c1989cea9beb3858d65e0a69`, after the W05 strict VIR
boundary completed. The normative practical-profile, shared-artifact, and
manifested-vector SHA-256 values were respectively
`a45d0fda3322f17148201504f19e620c389a65ca24f79b5cbed4fa6a9c29b28b`,
`d1d6618dba572d24c3706d5cd88d08c6532fa6828c356803fd63ea2bc32395ab`,
and `d3b7be7d09d79742c8f339db3ca6313808e0257c30476cf6959aa7c5231d1b16`.
The private freeze SHA-256 remained
`83954067c156e58cb349dbf07da44edf60a3ec550e628e6d2f1a890889d574e3`;
the registered foundation descriptor raw SHA-256 remained
`2ddc0cd4381a574b093b2f208acfaf7028c1db78bbdcd43daea4b616f5f40781`, with its independently
validated content hash
`d8c2a023f1c445470123519f5024a17aaca1766553331a2fed4733fecf7deec1`.

W06 adds the Rustdoc-hidden
`crates/mpk-vc/src/csharp_practical_vc_model.rs` boundary and its sole owner
test. It accepts only a W05 `ValidatedPracticalVir` capability paired with the
same W04 artifact context and captured-input snapshot. The importer repeats
and validates the complete canonical semantic context, compilation ID,
input-set digest, registered descriptor tuple, VIR schema/hash/size, ordinary
encoding profile, verification-limit profile, and validated lineage. Neither
a caller-provided digest nor an opaque successor reference can substitute for
that capability chain.

The new `mpk.vc.v3` root has schema-declared member order and the
`MPK-VC-3.0` hash preimage. Its type, operation, required-check, and explicit-
control entries are deterministically rebuilt from validated W05 data. Type
IDs have only checked-Bool-leaf, registered monomorphic Boolean-cube, or
application-projection routes. Every W03 operation, check, control-node, and
pattern tag has one exhaustive ordinary-term route and an explicit later
`CSHARP-03-T06-W02` through W09 proof owner. There is no intrinsic, runtime,
host-evaluation, generic theorem, proof-node, theory, or axiom route. Sorted
obligation groups name all reconstructed subjects, retain exact dependency
order, and begin with the ordinary-foundation group.

The model records only Certificate v0's `Sort`, `Var`, `Const`, `App`, `Lam`,
`Pi`, and `Let` terms and the frozen inclusive maxima of 262,144 ordinary term
nodes, 8,192 generated declarations, binder depth 256, and 16,384 static
transformers. It deliberately generates no proof and invokes no checker;
actual definition/proof construction and counter enforcement remain with the
already frozen T06-W09 owner.

### 19.2 Skeleton, assembly profile, and predecessor preservation

`mpk.vc.cert_skeleton.v3` is reconstructed from the accepted VC. Each sorted
obligation group yields one theorem interface with the same subjects,
dependencies, function scope, and later proof owner. Its complete canonical
bytes are linked to the exact semantic context, foundation, VIR, and VC and
hashed under the frozen successor VC domain.

The separate `mpk.program_certificate.ordinary_context.v2` plan is bound to
that context, VIR, VC, and skeleton and self-hashed under
`MPK-PROGRAM-ASSEMBLY-2.0`. It admits generated `Def` and `Theorem` plans over
the seven ordinary term forms and structurally fixes imports, proof-node table,
theory-certificate table, axiom entries, axiom dependencies, and all five axiom
counts to empty or zero. A direct validator applies the same intersection to a
real `mpk_cert::Certificate`: imports, proof nodes, theory certificates,
`Axiom`, and `TheoryPrimitive` declarations reject, and the complete axiom
report must equal an independent recomputation with total count zero.

The fixed owner vector produces VC, skeleton, and assembly hashes
`2a66a554285ed4f5f2263a7996bfbb87c35da421c38e81052dbe5369db5df598`,
`03dbe460131c1240bf35aac8b7b954b9cac5a215a184efef66eea0735fd2f420`,
and `90002cbc6f509d02365c2ec04c12c708112c7ab2ecaf3ad40448bf5ca7ac38c1`.
Old `mpk.vc.v1`, `mpk.vc.v2`, Certificate v0, certificate hash domains,
program-certificate implementations, kernel/checker code, and their
acceptance rules are unchanged. The practical structure validator is opt-in;
no existing producer or checker calls it. No CLI/API dispatch, installed
registry, bundle, release descriptor, fixture, application source, or public
route changed.

Application build/output inspection, source-generic and iterator/async source
cases, application semantic-binding proof discharge, boundary/transition
execution, and native Linux release evidence are `not_applicable` at this
model-only item under task-document section 4. Their first applicable owners
remain T03 through T08. W06 nevertheless consumes the registered foundation,
complete independently validated closed operation set, and W05's generic-free
VIR capability; it cannot accept a residual template or cross-input binding.

### 19.3 Verification and review

Completed local verification on Darwin arm64:

| Command | Result |
| --- | --- |
| `python3 develop/probes/csharp-03/foundation_package.py --check` | pass: exact registered foundation and 2,051 vectors |
| `python3 develop/probes/csharp-03/profile_package.py --check` | pass: 10 freeze owners, 63 downstream owners, and 700 vectors |
| `cargo test -p mpk-vc --test csharp_practical_vc_model` | pass: all five W06 owner tests, including fixed hash vectors, canonical/linkage/limit mutations, exhaustive route tables, and direct proof/theory/axiom rejection |
| `cargo test -p mpk-vc` | pass: complete `mpk-vc` regression suite |
| `cargo test -p mpk-cli --test successor_atomic_cutover` | pass: candidate remains unavailable to installed routes |
| `cargo clippy -p mpk-vc --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

First-pass review findings: `4`.

- A first boundary draft lacked access to the complete W05-validated operation
  set, including registered foundation operations not invoked by a particular
  function. W05 now retains that independently reconstructed sorted table
  behind one crate-local accessor; W06 never trusts a caller operation list.
- Successor references initially exposed only schema/hash/size and context
  linkage. A crate-local predicate now also requires the exact captured-input
  digest and revalidates capture-to-context linkage before VIR, VC, or skeleton
  capabilities can be composed.
- The ordinary type route was initially internal while the exhaustive W03 tag
  routes were externally owner-tested. Its narrow public, Rustdoc-hidden route
  function now makes all three accepted type families and unknown-family
  rejection directly auditable without exposing an application API.
- The real Certificate structure check initially had only a test-time
  `mpk-cert` dependency. It is now an ordinary `mpk-vc` dependency so the
  production W06 boundary validates the actual Certificate v0 model rather
  than a parallel test shape.

Verification follow-up findings: `6`.

- The first Certificate validator imported `DeclarationKind` from a root that
  does not re-export it. Compilation moved the import to the canonical public
  `mpk_cert::encode` module without changing its meaning.
- Initial hash assertions checked only lowercase digest shape. The owner test
  now pins the exact VC, complete-skeleton, and assembly bytes-to-hash vectors
  above and retains independent hash-field mutations.
- The first typed decoder disabled Serde's recursion limit without a preceding
  W05-style structural scan. W06's flat reference-table schema needs no such
  relaxation, so the final decoder retains the bounded default recursion
  guard for untrusted input.
- The frozen T01 consumer fingerprint initially counted the two new W06 owner
  paths as historical input drift. The inventory owner now excludes exactly
  the W06 production and test paths, as it already does for W01-W05, while the
  frozen consumer inventory bytes and fingerprints remain unchanged.
- The assembly plan initially duplicated the Certificate v0 format literal,
  and its negative suite covered only the task's minimum proof/theory cases.
  It now takes the format directly from `mpk-cert` and additionally exercises
  reordered and duplicate VC members, noncanonical skeleton bytes, assembly
  hash mutation, imports, and `TheoryPrimitive` declarations.
- Required-check ownership initially depended only on the four broad W03 tags,
  which would have assigned construction checks to the data owner and a
  transition event bound outside T06-W08. The VC boundary now independently
  validates all 37 frozen ID/tag pairs and maps construction, data, boundary,
  exception, and transition checks to their exact later proof owners.

Final review findings: `0`. The final pass rechecks W06-only private scope;
exact schema order and canonical transport; context/input/foundation/VIR/VC/
skeleton lineage; exhaustive monomorphic type, operation, check, control, and
pattern routing; obligation sorting/dependencies and later owners; frozen
limits/domains and fixed hashes; absence of an intrinsic/proof/theory/axiom
route; real Certificate v0 structural rejection; predecessor parser,
certificate, assembly, and checker preservation; non-activation; formatting,
warning-denied lint, focused mutations, all `mpk-vc` regressions, and the
complete local fast gate. T02-W07 is the sole ready item.

## 20. CSHARP-03-T02-W07 completion record

### 20.1 Candidate frontend request and input inventory

W07 adds the Rustdoc-hidden `csharp_practical_frontend_protocol` module to
`mpk-cli`; it is callable only through explicit typed injection and has no
runner, CLI command, registry lookup, fallback, or installed release route.
The `mpk.frontend.request.v2` constructor and importer require an already
validated practical semantic request and the exact immutable W04 capture set.
They emit and revalidate the frozen root and nested field order for
`source_snapshot_v2`, `source_snapshot_entry`, `sidecar_set_v2`, and
`sidecar_ref`, reject unknown/later schemas and duplicate fields, and recompute
the outer `MPK-FRONTEND-REQUEST-2.0` hash.

The source and sidecar rows must exactly partition the validated selection and
captured paths, raw hashes, sizes, and registered sidecar schemas. A sidecar
descriptor's schema must also equal the root schema in its exact captured
canonical bytes. Source and sidecar subset digests reuse `MPK-INPUT-SET-0.1`
only with its unchanged retained `InputEntry` preimage; the complete capture-
set digest remains the cross-artifact lineage identity. W07 applies the frozen
per-file, aggregate, entry-count, path, and request-transport limits before
returning a request. Sorted arrays are emitted by canonical bytes and duplicate
rows reject.

### 20.2 Success, rejection, map, and manifest closure

A success envelope is constructible only from the complete typed
`mpk.frontend.source_artifacts.v2` root. The artifact context must match every
validated selection field, the request and artifact semantic contexts must be
field-complete equal, the artifact selection hash must match, and the exact
captured input-set lineage must survive through the source map, frontend
manifest, and source-artifact root. W07 extends the existing W04 artifact
capability only with narrow typed predicates and propagation of that already
validated lineage. Certificate manifests likewise retain the frontend
manifest's input set and reject a conflicting validated VC or skeleton input
set. No map or manifest schema, field, hash domain, or active predecessor
implementation changes.

Rejected output has the exact `rejected` status and can contain no artifact
field. Raw request identity is reduced to its hash and byte count before parse.
The linkage arm is `unvalidated` only before request validation and otherwise
repeats the exact frontend request hash and complete schema-ordered semantic
context. Callers supply only one of the 29 typed diagnostic families and an
optional numeric source location; W07 emits the sole frozen 63-byte public
message. It selects the earliest failing phase, discards later-phase findings,
sorts within that phase by the frozen family/location order, validates every
byte range against the retained raw source size, and enforces the 1,024-issue
budget. The API accepts no compiler message, source text, path, exception,
culture, generated type name, or stack string, so those values cannot enter a
failure artifact.

The private CLI transport is one canonical JSON document plus one trailing
newline. Success maps only to exit 0 and rejection only to exit 3; missing,
truncated, duplicate, noncanonical, unknown-version, wrong-exit, signaled,
stderr-limit, and canonical-size failures all reject with a closed public
protocol code. The active `mpk.frontend.cli.v1`, predecessor producers,
Certificate v0, checkers, application source/output, registry/release files,
and installed behavior remain unchanged.

### 20.3 Verification and review

Completed local verification on Darwin arm64:

| Command | Result |
| --- | --- |
| `python3 develop/probes/csharp-03/foundation_package.py --check` | pass: exact registered foundation and 2,051 vectors |
| `python3 develop/probes/csharp-03/run-checker-capacity-probe.py --check` | pass: all 48 dual-checker capacity invocations remain accepted |
| `python3 develop/probes/csharp-03/run-recursor-probe.py --check` | pass: all 60 dual-checker feasibility verdicts remain unchanged |
| `python3 develop/probes/csharp-03/profile_freeze.py --check` | pass: 17 identity families, 15 strict schemas, 35 practical limits, and 700 vectors |
| `python3 develop/probes/csharp-03/profile_package.py --check` | pass: 10 freeze owners, 63 downstream owners, and 700 vectors |
| `python3 scripts/check-spec-vectors.py --check` | pass: all 26 registered specification-vector hashes |
| `cargo test -p mpk-cli --test csharp_practical_frontend_protocol` | pass: all six W07 owner tests, including all 26 owned schema vectors, all 29 diagnostic families, complete request/success/diagnostic round trips, map/manifest lineage, truncation, duplicates, later versions, limits, mutations, deterministic precedence, and byte fuzzing |
| `cargo test -p mpk-vc --test csharp_practical_source_artifacts` | pass: complete W04 map/manifest/source-artifact regression suite |
| `cargo test -p mpk-cli --test successor_atomic_cutover` | pass: candidate remains unavailable to installed routes |
| `cargo clippy -p mpk-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

First-pass review findings: `3`.

- The first context-lineage predicate compared only the validated context and
  selection digest. It now compares the selection schema, compilation ID,
  complete source/root/sidecar arrays, and digest so a hash never substitutes
  for the required field-complete equality.
- The first W04 bridge retained input lineage on maps and frontend manifests
  but not on the enclosing source-artifact and certificate-manifest roots. The
  roots now propagate the already validated input-set identity and reject any
  conflicting typed subordinate identity.
- Oversized canonical result documents initially surfaced as generic JSON
  failure through the shared practical parser. The protocol now checks that
  bound first and returns the closed limit failure deterministically.

Second-pass review findings: `2`.

- An unvalidated raw request larger than `u32` initially used the inventory
  limit helper even though raw request sizing belongs to transport. Request
  creation, diagnostic emission, and diagnostic validation now share one
  transport-phase size conversion.
- A caller-supplied sidecar descriptor initially needed only a registered
  schema ID and matching selected path. Request construction now also parses
  the captured canonical bytes and requires their root schema to equal that
  descriptor, preventing a valid registered label from being attached to a
  differently typed sidecar.

Third-pass review findings: `1`.

- The complete-selection predicate initially repeated the frozen practical
  selection schema as a local string literal. It now uses the registry-owned
  schema constant so the context and registry validators cannot drift.

Verification-gate findings: `3`.

- The repository-wide inventory test still classified W07 as ready and used
  W06's pre-commit `SELF` marker. It now classifies W07 as complete, records
  W06's published commit, assigns `SELF` to W07, and makes W08 the sole ready
  item.
- The post-freeze search exclusions stopped at W06, so W07's new protocol
  implementation and owner-test paths were incorrectly compared with the
  historical W02 consumer fingerprints. Both W07 paths now join the same
  explicit post-freeze exclusion set as W01-W06; the published W02 inventory
  bytes and their W09/W10 bindings remain unchanged.
- The first complete gate showed that exporting the W07 production module
  legitimately changes the checker/build source inventory even though it does
  not change either checker rule or installed route. Both independent checker
  probes were rerun: all 48 capacity invocations remain accepted and all 60
  feasibility verdicts remain unchanged. Their evidence records, the private
  freeze, its vectors, the W10 conformance package, and the vector manifest
  were regenerated with the checked-in deterministic tools.

The W07 revalidation records are now 74-file inventories. The capacity record
has raw SHA-256
`1cbe1b2c453432e100bc46abcea129c3f695a5cc318f029fdb72430593a23ba4`
and source-inventory SHA-256
`e2f0932f20f452e6d45068079bd9f87a2c08a3445b9b1237c4bb35f8992b9e97`;
the recursor record has raw SHA-256
`97fb98167d207193030e4918a0dcf66951e0fa1d53e18e525f1b2be1228a2b02`
and source-inventory SHA-256
`ad0de6c600b4d57e7e71119a46748bff230bdcf0105274d61d62a3ff53a081ae`.
The regenerated private freeze has content SHA-256
`3182c77c5b8fefb68d9d44ed202291f2ebd0fedf6c4e75be81315f24c31def82`
and raw SHA-256
`2caf30720c3ae67ac7e975a8f3382601aa5f1863079ea884a458adc0e7589323`;
its vector record has raw SHA-256
`2ba41ef31f12c5806a34d10fc3ebb8067b1b69d630cd9d0d3c150f00e46889c1`.
The conformance package and manifest raw SHA-256 values are respectively
`57945110a447736425edcf185ba724dbbe0ae5572ccf9dd73ef9391ac03916f2`
and
`432fc4e17b60c2a0677d654d026e443becbdbcbfcc9ef97879c48141f591dc80`.
Sections 11-13 retain their original completion-time hashes; these W07 values
are the current-file revalidation identities and change no schema, vector ID,
limit, checker verdict, or hash domain.

Final documentation review findings: `1`.

- The W07 evidence-revalidation commands were initially inserted into the
  historical W04 verification table. They now appear only in this W07
  completion record, alongside the evidence identities they validate.

Final review findings: `0`. The final pass rechecks W07-only private scope;
all 26 frozen owner vectors; schema/member/hash order; exact source/sidecar
inventory and retained input-set preimages; original-byte source-map coverage;
frontend, source-artifact, and certificate-manifest linkage; complete success
versus artifact-free failure; raw-request linkage; all 29 diagnostic families,
phase/location sorting and budgets; public-data sanitization; one-line
transport and exit mapping; fuzz/mutation/limit behavior; predecessor
preservation; non-activation; formatting, warning-denied lint, focused
regressions, and the complete local fast gate. T02-W08 is the sole ready item.

## 21. CSHARP-03-T02-W08 completion record

### 21.1 Private predecessor boundary and sole successor output

W08 adds the Rustdoc-hidden `csharp_practical_migration` module to `mpk-cli`.
It has no runner, CLI command, registry lookup, environment switch, fallback,
or installed release route. Its migration entry accepts only an already
validated active revision-3 frontend envelope, a separately validated
revision-4 request, and the immutable captured input bytes. Source language,
semantic profile, semantic parameters, selection, active predecessor registry,
and registered successor foundation descriptor must all match. The private C#
dispatch has exactly the scalar-v0 and practical-v1 arms and rejects every
ambient selector and non-C# profile.

For each accepted predecessor envelope, the adapter canonically regenerates
the `mpk.frontend.request.v2`, `mpk.vir.v2`, `mpk.source_map.v2`,
`mpk.source_manifest.frontend.v2`, empty context-bound semantic-binding and
closed-instance roots, `mpk.frontend.source_artifacts.v2`, and
`mpk.frontend.success.v2` artifacts. It replaces every contract context with
the complete validated revision-4 context, recomputes every changed hash with
the frozen successor domain, and preserves the original captured-input set,
unit behavior, and obligation projection. A predecessor receives zero
practical closed instances and the equivalence receipt records zero axioms.
The retained input and artifact limits are enforced both before parsing old
bytes and after canonical successor regeneration.

Every predecessor failure becomes one artifact-free
`mpk.frontend.diagnostic.v2` document. Only the fixed public message and typed
family escape; predecessor compiler prose, paths, snippets, stack text, and
generated names do not. The root phase is derived from the predecessor phase
and must equal the frozen phase of its diagnostic family. Both success and
failure transports are one canonical JSON document followed by one LF. The 34
W08 schema vectors execute the same strict production validator used on every
generated result rather than a test-local shape approximation.

### 21.2 Complete-corpus equivalence and reports

The primary owner executes all 178 W08 vectors: 34 diagnostic, 4 dispatch, 10
frontend-linkage, 96 retained-limit, and 34 schema cases. It directly migrates
the 13 accepted and 8 rejected checked-in Go frontend cases twice and the 13
accepted checked-in Rust artifacts plus all 73 frozen Rust rejection cases
twice. The scalar C# and Java paths bind all 30/88 and 49/99 accepted/rejected
case IDs respectively to their established real-frontend owners; every
rejected case is directly migrated twice. Their accepted real-frontend and
interpreter/VC/checker executions remain the pinned Linux/cache-gated owners,
while W08 composes those frozen results with the same typed migration adapter
whose accepted branch is exercised by every Go and Rust artifact.

The four checked-in reports are:

- `develop/migrations/csharp-03/predecessor-producer-csharp-scalar-difference.json`
  (raw SHA-256
  `33f9b44a9ff234722b9e3b134130899af570c08bf30525e245405983a0a2f2ce`);
- `develop/migrations/csharp-03/predecessor-producer-go-difference.json`
  (raw SHA-256
  `401eb7ada97db2c7f4f61064e4f34970a103f79648a09546d460719e65ca57e2`);
- `develop/migrations/csharp-03/predecessor-producer-java-difference.json`
  (raw SHA-256
  `38daf161d97e20ec230b476c159c13c3513acd323112122657ea6d331b8282d5`);
- `develop/migrations/csharp-03/predecessor-producer-rust-difference.json`
  (raw SHA-256
  `9b2dccfa71f7870b714102d05a11a38a111cab5497ccc40e79866d8cbb3bcd6e`).

Each report binds the exact corpus bytes and case-ID digest, the real frontend
and interpreter/VC/checker owner, two-run equality, the v1-to-v2 artifact
identities, zero behavior/obligation/verdict/axiom differences, zero practical
foundation instances, unchanged Certificate v0, and absence of a public
selector or installed route. The adapter and primary owner raw SHA-256 values
at completion are respectively
`bcbdd41bfb06cd7171169757e57b7cf486d191170384b8017c0feaa13947b930`
and
`cacdf3612a64d5701ada23f27696f63671ba9af7d08f4ad6968cdc286e896930`.

### 21.3 Verification and review

Completed local verification on Darwin arm64:

| Command | Result |
| --- | --- |
| `python3 develop/probes/csharp-03/foundation_package.py --check` | pass: exact registered foundation and 2,051 vectors |
| `python3 develop/probes/csharp-03/run-checker-capacity-probe.py --check` | pass: all 48 dual-checker capacity invocations remain accepted |
| `python3 develop/probes/csharp-03/run-recursor-probe.py --check` | pass: all 60 dual-checker feasibility verdicts remain unchanged |
| `python3 develop/probes/csharp-03/profile_freeze.py --check` | pass: 17 identity families, 15 strict schemas, 35 practical limits, and 700 vectors |
| `python3 develop/probes/csharp-03/profile_package.py --check` | pass: 10 freeze owners, 63 downstream owners, and 700 vectors |
| `python3 scripts/check-spec-vectors.py --check` | pass: all 26 registered specification-vector hashes |
| `cargo test -p mpk-cli --test csharp_practical_migration` | pass: all seven W08 owner tests and all 178 owned vectors |
| `cargo test -p mpk-cli --test csharp_practical_frontend_protocol` | pass: all six W07 protocol regressions |
| `cargo test -p mpk-vc --test csharp_practical_source_artifacts` | pass: all six W04 artifact/linkage regressions |
| `cargo test -p mpk-vc --test csharp_practical_inventory` | pass: all four inventory and ledger closure tests |
| `cargo test -p mpk-cli --test csharp_frontend_vectors --test java_lowering` | pass: frozen corpus ownership; two Java native tests remain explicitly Linux/cache-gated |
| `cargo test -p mpk-cli --test csharp_policy_verify --test java_policy_verify` | pass: portable owners; native C#/Go/Rust and Java dual-checker owners remain platform-gated |
| `cargo test -p mpk-cli --test successor_atomic_cutover` | pass: candidate remains unavailable to installed routes |
| `cargo clippy -p mpk-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

The W08 revalidation records are now 75-file source inventories. The capacity
record has raw SHA-256
`5525760fa2db1f022affb9d9c06907c76cf426939536f8cd4254355854842366`
and source-inventory SHA-256
`e153e971b5b90e47300b1d213d7bb2bc4e62df3a6f9813a849da6d2b9e463682`;
the recursor record has raw SHA-256
`b30df0edceee6d170422520abdbcbdf23313558a9f86ff1061e0b134489497fa`
and source-inventory SHA-256
`2f0b7adc7047f69fcdd9234c359eb18279b14d3c7ec2dc7365534bb979a11aae`.
The regenerated private freeze has content SHA-256
`6f7c4e61f7ad8830901e27267e57ecd219eb5030f01359d5adef138f015ccb3d`
and raw SHA-256
`ce87461072c9446d2e86202bdc3ecedad6840d006b635e0b7a4073b641c10aa2`;
its vector record has raw SHA-256
`2a493ef9647d38c6f2cc06650e3214f9124b279744982d6229fc8515669ce373`.
The conformance package and vector manifest raw SHA-256 values are respectively
`dfce6f3c46627586d14b515f2068ab2f5f54c79c27975d427d556ccbb220a67e`
and
`236da4c266b2f2b4127691212c51969e6ec73836f211c6cd417c7959ebdbb522`.
These identities change only because W08's production source and evidence
owners joined the private inventory; no schema ID, vector ID, limit, checker
verdict, hash domain, active registry, release tuple, or installed route
changed.

First-pass review findings: `1`.

- The 34 schema vectors initially executed test-local shape helpers. They now
  call a strict production validator in the migration module, and every
  generated success, diagnostic, and source-artifact root passes that same
  boundary before return.

Second-pass review findings: `3`.

- Source-error and lowering conversions could initially pair a diagnostic
  family with a different root phase. The adapter now reuses W07's typed family
  registry, maps each predecessor phase to the matching frozen family, and
  rejects mixed-phase documents.
- The first identity check accepted a validated envelope from an older
  predecessor registry. Migration now requires the exact installed revision-3
  identity, and a revision-2 envelope has a direct rejection test.
- Canonical artifact limits were initially checked only against predecessor
  input bytes. VIR, source-map, and frontend-manifest limits are now checked
  again after successor context and hashes have been added.

Verification-gate findings: `2`.

- Adding the W08 production module changed the checker/build source inventory
  without changing checker behavior. Both independent probes, the freeze, its
  vectors, and the conformance package were regenerated; all dual-checker
  results remain identical.
- Package regeneration changed the practical conformance-vector bytes. Its
  global vector-manifest digest was updated, after which all 26 registered
  vector sets passed the closed manifest check.

Documentation review findings: `1`.

- README and executive-summary wording could imply that the private migration
  implementation itself was normative. They now state separately that the
  specification package is normative but inactive and that migration remains
  private and uninstalled.

Final review findings: `0`. The final pass rechecks exact W08 scope; active
revision-3 and candidate revision-4 identity binding; all four producer paths;
complete context replacement and hash regeneration; strict schema and retained
limit enforcement; sanitized, phase-consistent artifact-free diagnostics;
two-run determinism; all 178 frozen vectors; complete corpus/report identities;
zero behavior, obligation, verdict, axiom, and practical-instance differences;
Certificate v0 preservation; absence of a format selector or installed route;
formatting, warning-denied lint, predecessor regressions, inventory closure,
and the complete local fast gate. T02-W09 is the sole ready item.

## 22. CSHARP-03-T02-W09 completion record

### 22.1 Sole private consumer closure

W09 adds one Rustdoc-hidden, non-dispatchable consumer orchestrator. It accepts
only an explicitly validated revision-4 registry, a test-injected successor
release descriptor, a W08 migration, and retained Certificate v0 bytes. Before
calling any downstream consumer it independently checks canonical transport,
the last hash field and frozen domain of every W08 root, complete semantic-
context equality, selection identity, foundation identity, every schema/hash/
size reference, the source-artifact root embedded by the success result, and
the predecessor equivalence/axiom receipt. There is no old-schema branch,
format selector, environment override, installed-registry lookup, or fallback.

The production identity inventory is the exact 17-family successor projection
from the T01-W09 freeze. It executes all four frozen W09 identity vectors and
rejects duplicate successor identities, duplicate successor hash domains, and
an old schema paired with a successor context. The deterministic accepted
receipt names all 18 W09-owned inventory edges. Those edges cover semantic-
context/API linkage, source artifacts and frontend result consumption, both
registered-foundation edges, VC/skeleton, program assembly/profile binding,
policy/evidence and its CLI paths, AI request/report and CLI preparation, API
request/session/response, release model/resolver/re-export, and launch-free
fixture validation.

Successful predecessor output is consumed as `mpk.vc.v3`,
`mpk.vc.cert_skeleton.v3`, and
`mpk.program_certificate.ordinary_context.v2`, followed by the v3 policy,
AI, and API families. All serializers immediately pass their bytes through an
independent strict importer. A predecessor diagnostic stops with exactly the
v2 request and diagnostic; its W09 receipt records zero downstream artifacts.
The checked-in implementation receipt is
`develop/migrations/csharp-03/private-consumer-migration-receipt.json`.

### 22.2 Release and foundation gate

The candidate-only release consumer validates a test-injected `mpk.release.bundle_registry.v2`
root and exactly five profile tuples. It
recomputes the frontend/toolchain content hashes and root hash under their
frozen domains; requires sorted unique bundle IDs, profile claims, contracts,
portable ASCII member paths (including case-folded uniqueness), member raw
hashes, per-file and per-bundle byte limits; binds the revision-4 registry,
profile-entry hashes and registered foundation descriptor into every tuple;
and requires each profile to have exactly one frontend and one toolchain. It
does not read or write `release/bundles`, materialize a bundle candidate, or
make the practical profile discoverable.

Program consumption independently validates the registered practical
foundation descriptor/definition bytes, reloads every retained checker
foundation including `proofs/program/base/std-program-base.hex`, and passes
the supplied unchanged Certificate v0 bytes through the Rust source-free
checker. Certificate v0 bytes remain unchanged, proof and theory tables remain
empty, and both the consumer assembly and checker report require zero axioms.
The independent reference-checker regression remains part of the local
certificate suites; native installed-image execution remains owned by T07 and
T08.

### 22.3 Verification and review

Completed local verification on Darwin arm64:

| Command | Result |
| --- | --- |
| `cargo test -p mpk-cli --test csharp_practical_migration` | pass: all W08 regressions plus six W09 owner tests and all four frozen W09 identity vectors |
| `cargo test -p mpk-vc --test csharp_practical_inventory` | pass: all inventory, ledger, and post-freeze search checks |
| `cargo test -p mpk-cli --lib program_certificate::tests` | pass: 4 retained Certificate v0 assembly/checker protocol regressions |
| `cargo test -p mpk-cli --test program_certificate` | platform-gated: 0 tests on Darwin; native integration remains in the later Linux release gate |
| `cargo test -p mpk-cli --test csharp_policy_verify --test java_policy_verify` | pass: portable predecessor semantic/certificate regressions |
| `cargo test -p mpk-cli --test successor_atomic_cutover` | pass: no candidate is available to installed routes |
| `cargo clippy -p mpk-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

Final evidence regeneration binds the completed W09 production-source set.
The checker-capacity record has raw SHA-256
`8f9c40c19e2af7fd209975528a72b66010d81ac78823542db0d9acdc37ac8ad5`
and binds 76 source records under inventory digest
`edd01f537d1c87ec63f170c31220109da1e48f895c0a5d6c1b4d1fbeec5ad409`.
The recursor record has raw SHA-256
`b0cd0fd692749d34dabe2065a8d5efba79c36968275b72dc7dbe50450bdcf36d`,
binds that capacity record exactly, and has its own 76-record inventory digest
`f00428012b54eccac59dfc0d89da91afb372ad4919d7fe266c4d78f92307bb26`.
The regenerated freeze has raw SHA-256
`480ddd2399b3bc139b0ff9911f7efb5e0d87332ee41339a16c81165c6463523b`
and internal content SHA-256
`b1dc1c9c1b6124410965c0a1763b9cedc902eadee221058aa093a95f2d7000b5`;
its vector file has raw SHA-256
`0d4c7322974c211231c6e2216d737f250f4edd0fad672d37f995a953be8db4a3`.
The manifested 700-vector package has raw SHA-256
`dd3b45276d91086b62dce8757cef4f79e16fe1dedcfce398af0ff979ad2489ac`,
and the 26-set manifest has raw SHA-256
`1bf650796e5ccbe76d004ada782b80afaeb7f21e339f42c24ed14f62cb0b2f4f`.
These changes bind the added private consumers; no frozen semantic value,
identity, hash-domain name, limit, or checker verdict changed.

Implementation-verification findings: `2`.

- W08 artifacts use schema-ordered practical JSON, while their nested
  validated semantic request retains the registry's canonical-key order. The
  first consumer used one canonicalization rule for both. It now validates
  the outer artifact with the practical codec and re-encodes the nested request
  with the registry codec before registry validation.
- W08's non-routing assertion initially rejected any later private reference
  from the successor frontend-runner module. W09 legitimately adds its typed,
  launch-free consumer there, so the W08 assertion now guards actual CLI and
  installed runner entrypoints while W09 separately proves that its consumer
  is absent from `main` and has no dispatch spelling.

First-pass review findings: `3`.

- The first identity table contained only representative W09-consumed names.
  It now contains every successor identity and hash domain in all 17 frozen
  families, and the owner test compares every ordered list directly with the
  freeze before executing the duplicate and mixed-family mutations.
- The first fixture projection flattened equal relative member names from
  different bundles and rejected any name containing an unrelated historical
  word. Member names are now qualified by bundle kind and ID, while validation
  enforces only the closed portable-path rules.
- The first raw artifact importer checked the outer hash but not every root's
  exact ordered field set and nested successor reference types. It now applies
  root-specific request, context, source/map/manifest, binding, closed-instance,
  foundation, hash, byte-size, and collection checks before graph linkage.

Second-pass review findings: `4`.

- Release members initially lacked ASCII case-fold collision checks and the
  retained file/bundle byte ceilings, while bundle IDs allowed noncanonical
  separators. The release builder and importer now enforce all three.
- Predecessor VC/skeleton/assembly references initially omitted canonical byte
  sizes and explicit registered-foundation linkage, and their hash preimages
  passed through a key-sorted generic map. The documents now carry complete
  artifact references and foundation descriptors and hash dedicated
  schema-ordered preimage structs.
- The certificate edge initially accepted only an assembly schema/hash pair.
  It now independently consumes and revalidates the complete assembly bytes,
  zero-axiom report, empty proof/theory/import tables, linkage identities, and
  assembly hash before invoking the retained Certificate v0 checker.
- Policy, AI, and API raw re-import initially relied on builder-validated
  context inputs. They now validate the complete strict successor context at
  every raw boundary; policy hashes also use dedicated ordered preimages and
  all nested reference schema/hash pairs are checked.

Documentation review findings: `1`.

- README and executive-summary wording briefly described the already
  published normative specification package as private. They now distinguish
  the normative-but-inactive package from the private, uninstalled producer
  and consumer migrations.

Final review findings: `0`. The final pass rechecks all 18 inventory edges;
the exact 17-family identity/domain freeze and four owner vectors; canonical
artifact/hash/reference/context linkage; deterministic accepted and rejected
receipts; strict release root, bundle, member, tuple, foundation and domain
mutations; raw VC/policy/AI/API importer mutations; retained Certificate v0 and
zero-axiom checks; complete predecessor regressions; absence of a candidate,
public route, installed metadata change, or dual-format fallback; formatting,
warning-denied lint, and the complete local fast gate. T03-W01 is the sole ready item.

## 23. CSHARP-03-T03-W01 completion record

### 23.1 Private source capture and finite closure

W01 adds one internal capture gate that is deliberately absent from the active
`csharp2vir.csproj`, installed build-input manifests, CLI dispatch, and release
registries. The gate accepts only the frozen
`mpk.selection.csharp_members.v1` selection and test-injected captured inputs.
It retains independent immutable snapshots of every selected `.cs` file and
exact JSON sidecar, assigns deterministic ordinals, validates sorted unique
portable paths and selected-root IDs, decodes strict BOM-free UTF-8, and
enforces the frozen file-count, per-file, total-byte, selected-method,
syntax-node, contract-input, and source-data-exception-type limits. Project,
package, binary, generated-source, analyzer-config, editor-config, ambient
source, and ambient reference inputs fail without artifacts.

Compilation uses the exact pinned .NET 10 reference projection of 167 files
and the pinned Roslyn public API and options. The gate checks that reference
closure byte-for-byte, rebuilds canonical in-memory references from the checked
PE snapshots, and compiles only those snapshots in canonical order. It rejects
every active compiler error or warning, admits only the frozen informational/
hidden diagnostic disposition, and sanitizes every failure to the single public
message with phase, family, stable private code, and zero artifacts.
Dependencies reject MPK packages, assemblies, namespaces, attributes, base
types, interfaces, and generated sources, so selected application source
remains MPK-independent.

Every ordinary declaration in every selected file is accounted for before
reachability. The closed gates reject unsupported dead declarations, delegates
and method-group values, lambdas, expression trees, query/LINQ, dynamic,
reflection and runtime code generation, external effects, concurrency,
source-written attributes, every user generic declaration or method, and every
source-visible constructed generic. Exact nullable value types are immediately
concretized; all other framework types and API symbols must match the closed
allowlist, and `StringComparison`/`MidpointRounding` constants are admitted only
as the exact argument operation of their allowlisted intrinsic. Compiler-
synthesized init/required metadata is observation-only and cannot become a
callable source API.

Selected roots then produce complete deterministic method and source-data-type
closures, including constructor initializers. Dead declarations, call cycles,
type cycles, and inclusive closure limits reject before lowering. Source type
and method identities use the registered canonical declaration algorithm, and
each admitted foundation-owned closed instance uses the registered closed-
instance algorithm. The executable Rust/C# cross-check fixes both source IDs
and representative `bounded_sequence`/`option` instance IDs to the already
registered foundation; W01 introduces no alternative identity or generic
runtime representation.

### 23.2 Exact executable evidence

`crates/mpk-cli/tests/csharp_practical_capture.rs` is the primary owner. It
binds the source and executable C# harness through
`develop/migrations/csharp-03/capture/capture-inputs.json`, proves that neither
file enters an installed route, projects the three frozen
`source_data_exception_types` vectors, and cross-checks registered foundation
and source identities. The build-input helper materializes and executes the
harness only from this canonical manifest and the pinned Linux/amd64 closure;
the read-only repository mount cannot supply an unrecorded source.

The current manifest raw SHA-256 after W02's directive pass-through correction
is `6249750d3a71cc918e29a7f6cbcf93b219ce0dd1093fa876382e8223c2a724e0`.
Its two records are:

| Path | Bytes | SHA-256 |
| --- | ---: | --- |
| `crates/mpk-cli/tests/csharp_practical_capture_harness.cs` | 55,730 | `5332a5af2d434bb00e30f05d679ba054923e43f0e0168de56e0a4238e07daafc` |
| `csharp-tools/csharp2vir/PracticalCapture.cs` | 124,249 | `ea1bc3fa3ea9980a8d85132f67a9ee3351f9177416b6c954578175c7af992aa8` |

The harness covers deterministic multi-file closure; immutable source and
sidecar capture; source selection, path, encoding, reference, and every frozen
limit boundary; dead and unsupported declarations; all compiler severities;
dependency, declaration, type, generic, and effect phase precedence; exact
nullable and closed framework types; incidental transitive generic metadata;
context-bound intrinsic constants; constructor-initializer closure; call/type
cycles; synthesized metadata; and artifact-free failures.

### 23.3 Verification and review

Completed local verification on Darwin arm64, with the capture harness also
compiled and executed in the exact Linux/amd64 environment:

| Command | Result |
| --- | --- |
| `cargo test -p mpk-cli --test csharp_practical_capture` | pass: all six W01 owner tests; its Linux/cache harness launch is platform-gated on Darwin |
| `docker run --rm --platform linux/amd64 ... scripts/build-csharp-practical-frontend.sh --test-capture` | pass: exact manifest-only .NET 10/Roslyn build and all executable C# vectors |
| `scripts/build-csharp-practical-frontend.sh --check-build-inputs` | pass |
| `scripts/build-csharp-practical-frontend.sh --self-test` | pass |
| `cargo test -p mpk-vc --test csharp_practical_inventory` | pass: status, ledger, source inventory, and route-absence closure |
| `cargo test -p mpk-vc --test csharp_practical_source_artifacts --test csharp_practical_vir_model` | pass: predecessor identity and artifact regressions |
| `cargo test -p mpk-cli --test csharp_practical_build_inputs --test csharp_practical_migration --test csharp_practical_frontend_protocol --test successor_atomic_cutover` | pass: build-input and predecessor/candidate routing regressions |
| all six CSHARP-03 freeze/package/vector `--check` commands | pass |
| `cargo clippy -p mpk-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

Implementation-verification findings: `2`.

- A `Guid` member mutation initially reached the generic framework-API failure
  before its frozen effect-family result, and nullable symbols were initially
  identified from display spelling. The gate now orders those checks by the
  frozen phase and identifies nullable values from Roslyn symbols.
- The first executable path could use a locally available compiler closure
  without independently binding every reference. It now checks the exact
  167-file .NET 10 reference projection and every captured implementation input
  before compilation.

First-pass review findings: `8`.

- Review closed MPK dependency spelling, portable-path and immutable-byte
  aliases, constructor-initializer reachability, reflection/runtime-codegen
  aliases, effect-protocol families, method-limit inclusivity, exact nullable
  admission, and framework constructed-type admission. Each now has a direct
  executable mutation and an artifact-free failure assertion.

Second-pass review findings: `9`.

- Review removed an enum self-edge, rejected noncanonical identifier spellings,
  closed required/init metadata use, validated diagnostic locations, corrected
  empty and missing selected roots, made call/type traversal deterministic,
  added closed source and foundation identity cross-checks, closed nested
  framework-type recursion, and strengthened public-message sanitization.

Final-hardening review findings: `8`.

- Review corrected recursive array/nullable unwrapping and declaration-before-
  type-cycle precedence; made generic rejection precede the framework firewall;
  removed three implementation-invented limits; counts only reachable
  source-data exception types; rejects verbatim, Unicode, and escaped identifier
  spellings; validates nullable-reference and nested generic components;
  makes nested `bounded_sequence<option<T>>` identity construction iterative;
  and retains sidecar bytes in the immutable result closure.

Documentation review findings: `1`.

- Status prose still named T02-W09/T03-W01 as the next item after W01 became
  complete. The design, plan, README, executive summary, and this ledger now
  consistently identify T03-W02 as the only ready item.

Final review follow-up findings: `13`.

- Dependency resolution initially classified an application-owned local,
  parameter, member, or type spelled exactly `Mpk`, and local symbols in an
  application compilation whose ID was `mpk`, as MPK dependencies. The gate
  now distinguishes source-owned symbols from external MPK ownership while
  retaining phase-1 rejection for real and unresolved MPK dependencies.
- `nameof` could create false call/type reachability, a property getter could
  be selected as a root, and a class/struct primary constructor could escape
  ordinary-declaration accounting. Semantic `nameof` subtrees are now pruned,
  roots are exact methods or Roslyn `MethodKind.Constructor` instance
  constructors, and primary constructors reject as unsupported declarations.
- Exact nullable members initially bypassed the framework-member firewall,
  while the `StringComparison` and `MidpointRounding` carriers rejected their
  positive intrinsic uses and their constants were not position-bound. The
  nullable allowlist and exact Roslyn argument-operation checks now close both
  directions, with positive and negative executable vectors.
- Capture/selection enumerables were initially materialized without their
  frozen count bounds, and UTF-16-to-UTF-8 provenance conversion plus several
  declaration/graph lookups could grow quadratically. Bounded single-pass
  capture, a precomputed byte-offset table, symbol/edge indexes, an owner-indexed
  getter table, and a work queue make the retained work linear in the bounded
  input and graph sizes.
- The exact-input harness initially validated repository files before a lengthy
  materialization/build interval but did not bind the copied compiler/runtime
  inputs, and its owner assertion retained the earlier validation-call count.
  Every copied W01 source and runtimeconfig is now re-hashed against its
  in-memory manifest record, with the owner assertion fixed to the complete
  three-path validation and copy-binding contract.
- A caller-provided `PortableExecutableReference` implementation or a later
  file mutation could differ from the bytes whose reference inventory was
  checked. The gate now reads each bounded PE exactly once, validates the
  canonical 167-file inventory, rebuilds standard in-memory Roslyn references
  from those snapshots, and orders them canonically before compilation.

Final review findings: `0`. The final pass rechecks exact W01 ownership;
manifest hashes and manifest-only execution; frozen Roslyn/reference/options,
input, diagnostic, declaration, dependency, generic, effect, framework API,
call/type closure, identity, and limit barriers; all positive and negative C#
vectors; predecessor regressions; no active project, registry, bundle, CLI,
API, installed metadata, or application-source MPK dependency; formatting,
warning-denied lint, and the complete local fast gate. T03-W02 is the sole ready item.

## 24. CSHARP-03-T03-W02 completion record

### 24.1 Private normalization and exact-type handoff

W02 adds one internal normalizer over the W01-validated immutable source
closure. It remains absent from `csharp2vir.csproj`, active build-input
manifests, installed bundles, both registries, CLI/API dispatch, and application
source. Before normalization it independently snapshots and checks the exact
167-file .NET 10 reference projection, recompiles only the retained W01 source
bytes under the frozen C# 14/options tuple, and rejects any new compiler error
or warning without producing an artifact.

Expression-bodied methods, void methods, expression-bodied properties, and
getter arrows become the same semantic operation stream as their explicit
block/return/expression-statement forms. Constructors, setters, local
functions, and every other arrow owner remain rejected by their frozen
declaration precedence. The operation stream is prefix encoded with exact
child counts and traversed iteratively, so the W01 syntax-node bound does not
become an implementation call-stack limit.

Every ordinary local has a deterministic callable-local ordinal and an exact
normalized type. A direct `var` local is accepted only with one initializer,
one declarator, no alias/source type named `var`, no ref-shaped declaration,
and an exact admitted Roslyn type. Anonymous and dynamic types, target-typed
object/default/collection/stack-allocation forms, and implicit arrays that
require a best-common-type conversion reject. Homogeneous implicit arrays,
nullable values, and recursive array/reference nullability retain their exact
closed-instance and annotation structure. The reusable type normalizer and
its canonical structural key are iterative. Compilation-local, nullability-
aware caches ensure repeated occurrences of one deeply nested semantic type
reuse both normalization and the lazily built key; positive `foreach`,
including its `var` variable, remains exclusively T04-W02-owned.

Ordinary namespace imports at compilation-unit or namespace scope affect only
Roslyn name resolution. The semantic stream records resolved source IDs and
closed framework symbols, never source spelling. Global, static, unsafe,
alias, alias-qualified, generated, unresolved, ambiguous, or MPK imports,
`extern alias`, and disposal `using` forms reject with frozen precedence. The
sole admitted directive is one exact redundant file-wide `#nullable enable`;
all scoped, targeted, conditional, pragma, line, region, error/warning, and
other directives reject. Successful canonical bytes and per-callable body
bytes are immutable copies; failures retain the W01 fixed public message and
zero artifacts.

### 24.2 Exact executable evidence

`crates/mpk-cli/tests/csharp_practical_syntax.rs` is the primary owner. It
binds the implementation and executable C# harness through
`develop/migrations/csharp-03/syntax/syntax-inputs.json`, proves that the
normalizer has no active route, checks the published W02 owner and T04-W02
`foreach` handoff, and runs the pinned harness when the complete cache is
present. The build helper validates and copies only those manifest-bound bytes
before compiling and executing them in the closed Linux/amd64 toolchain.

The final manifest raw SHA-256 is
`8c5512550f2795b0cc69b080ca7f028cf911b6dd1e6f3b461a03fe07fecd995b`.
Its three records are:

| Path | Bytes | SHA-256 |
| --- | ---: | --- |
| `crates/mpk-cli/tests/csharp_practical_syntax_harness.cs` | 33,839 | `a33785dbb9fd95080b16a146f1ca68d848f01125c9b4a07f25eefbff3d7341ac` |
| `csharp-tools/csharp2vir/PracticalCapture.cs` | 124,249 | `ea1bc3fa3ea9980a8d85132f67a9ee3351f9177416b6c954578175c7af992aa8` |
| `csharp-tools/csharp2vir/PracticalSyntaxNormalization.cs` | 62,040 | `1e4df29c1187743c30f774133860eb19e978e4524c19150612f9a06b6c273fdb` |

The harness compares full canonical bytes, semantic hashes, callable IDs,
body bytes, and exact-type bindings across arrow/block, `var`/explicit,
import/fully-qualified, and directive/no-directive pairs. Its negative vectors
cover every W02 inference, import, directive, generated-input, deferred-
`foreach`, immutability, deterministic-order, sanitization, and artifact-free
failure boundary.

### 24.3 Verification and review

Completed local verification on Darwin arm64, with the syntax harness also
compiled and executed in the exact Linux/amd64 environment:

| Command | Result |
| --- | --- |
| `cargo test -p mpk-cli --test csharp_practical_syntax` | pass: all five W02 owner tests; its cache launch is platform-gated on Darwin |
| `docker run --rm --platform linux/amd64 ... scripts/build-csharp-practical-frontend.sh --test-syntax` | pass: exact manifest-only .NET 10/Roslyn build and all executable C# vectors |
| `scripts/build-csharp-practical-frontend.sh --check-build-inputs` | pass |
| `scripts/build-csharp-practical-frontend.sh --self-test` | pass |
| `cargo test -p mpk-cli --test csharp_practical_capture` | pass: complete W01 capture regression suite |
| `cargo test -p mpk-vc --test csharp_practical_inventory` | pass: status, ledger, source inventory, and route-absence closure |
| `cargo test -p mpk-vc --test csharp_practical_source_artifacts --test csharp_practical_vir_model` | pass: predecessor identity and artifact regressions |
| `cargo test -p mpk-cli --test csharp_practical_build_inputs --test csharp_practical_migration --test csharp_practical_frontend_protocol --test successor_atomic_cutover` | pass: build-input, predecessor, protocol, and non-activation regressions |
| all six CSHARP-03 freeze/package/vector `--check` commands | pass |
| `cargo clippy -p mpk-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `./scripts/check-fast.sh` | pass |
| `/usr/bin/git diff --check` | pass |

Implementation-verification findings: `5`.

- Roslyn does not expose the drafted `ILocalSymbol.IsImplicitlyTyped` member,
  and inferred reference nullability is carried by initializer `TypeInfo`.
  W02 now recognizes direct `var` syntax and retains that exact top-level
  annotation independently of the local symbol.
- Implicit arrays were initially rejected as a whole. The gate now admits only
  exact homogeneous element types and rejects each best-common-type
  conversion, as required by the frozen design.
- Several mutations initially expected W02 diagnostics even though compiler,
  dependency, type, or effect gates own earlier precedence. The executable
  assertions now bind the exact observed family and private code.
- The generated-global-import case initially omitted the generated input from
  the selected inventory. It now reaches and proves the intended W01
  `generated_source` rejection.
- W01's position-bound `StringComparison` and `MidpointRounding` constants are
  not general admitted value types. W02 now uses closed internal operation
  carriers for those exact intrinsic arguments, with qualified/imported and
  explicit/`var` equivalence tests.

First-pass review findings: `6`.

- The first handoff trusted the already checked reference objects while
  rebuilding the semantic model. It now re-snapshots every PE image, verifies
  the exact canonical inventory, and recompiles from the snapshots.
- Void expression-bodied methods and rejected constructor/setter/local-
  function arrows lacked direct equivalence or routing cases. All now have
  executable owners.
- Initial reference-nullability coverage was top-level only. Recursive array
  element nullability and its explicit/`var` byte equivalence are now checked.
- Alias-qualified namespace imports could pass the ordinary-import rule.
  `using global::...` now joins global/static/alias imports in the closed
  rejection set.
- A `ref var` type wrapped the identifier and bypassed the direct `var` check.
  Every variable-declaration type subtree is now inspected and only the exact
  direct by-value shape is admitted.
- Exact-type/key construction and semantic-operation emission initially used
  recursive traversal. Both now use bounded iterative work stacks and a flat
  prefix operation stream under W01's existing syntax-node/input bounds.

Second-pass review findings: `2`.

- The allowed-arrow test covered a non-init setter rejected by W01, but not an
  init accessor that reaches W02. A direct init-arrow mutation now proves that
  only method/property/getter arrows enter normalization.
- Duplicate and unresolved namespace imports relied only on the general
  compiler gate. Direct cases now freeze their earlier `compiler_diagnostic`
  precedence alongside W02's import-shape mutations.

Third-pass review findings: `1`.

- W01's prerequisite filter required `#nullable enable` to be the first
  non-whitespace trivia, which incorrectly rejected a harmless leading source
  comment even though the frozen rule is relative to the first non-directive
  token. The pass-through check now uses that token boundary; W02 proves the
  commented and comment-free forms have identical semantic bytes.

Fourth-pass review findings: `1`.

- Iterative traversal removed call-stack growth, but repeated operation and
  local uses of one deeply nested type still rebuilt its full normalization
  and structural key. Compilation-local `IncludeNullability` caches and a
  lazy key cache now bound that repeated work by unique semantic types.

Verification follow-up findings: `1`.

- The W01 owner counted only its original build-helper validation and copy
  sites. Its explicit regression assertion now includes W02's additional
  manifest validation, project-file check, and copied implementation/harness
  inputs. The preceding directive-boundary fix repinned the W01 implementation
  and manifest hashes recorded above; this assertion repair did not change
  either hash again.

Documentation review findings: `2`.

- Four shared-status sentences exceeded the repository's prose wrapping
  convention after W02 completion was recorded. They are reflowed without
  changing the completion state or implementation claims.
- The verification follow-up incorrectly said the W01 manifest and
  implementation hashes were unchanged, although the preceding directive-
  boundary fix had repinned both. It now distinguishes that repinning from the
  later owner-assertion repair, which did not change either hash again.

Final review findings: `0`. The final pass rechecks W02-only private scope;
manifest-bound immutable inputs; pinned reference and compilation identity;
expression-body, exact-type, nullability, inference, namespace import, and
directive semantics; byte-identical explicit equivalence; deterministic
ordering and immutable output; artifact-free sanitized failures; exact
diagnostic precedence; T04-W02 `foreach` ownership; predecessor preservation;
non-activation; formatting, warning-denied lint, focused regressions, and the
complete local fast gate. T03-W03 is the sole ready item.

## 25. CSHARP-03-T03-W03 completion record

### 25.1 Private declaration and type validation

W03 adds `PracticalDataTypes.cs` and the `CSharpPracticalDataTypes.Validate`
handoff over the W01/W02 immutable source closure. Its limit, declaration,
and type checks run inside the corresponding capture phases. The two W03
structural counters therefore precede dependency and compiler errors, while
ordinary declaration failures precede generic/effect diagnostics. Existing
callers retain their preceding behavior without installing a practical route.

The model admits closed source enums with all eight exact integer carriers,
implicit values, aliases, and full-width signed/unsigned constants. The
metadata-backed `System.DayOfWeek` is checked against its exact seven names,
values, and `int` carrier. Numeric/nullable-enum conversions, implicit numeric
zero, enum arithmetic, unknown member references, and ineligible zero/default
publication reject. An exact already-typed enum value remains a value of that
closed type; boundary decoding and boundary validation remain T05-owned.

Ordinary data declarations require the exact implicit `object`, `ValueType`,
or `Enum` base. Non-generic readonly structs and sealed classes retain fields
and properties in canonical source order and types in acyclic dependency
order. Mutable/static/constant/virtual/ref-like state, source initializers,
custom init bodies, other base lists, and unsupported declarations reject.
Only the frozen auto-property backing-storage category is retained; W04 owns
its complete construction and synthesized-constructor proof. The model checks
identity operations, direct mutation, and array mutation through member,
parameter, local-alias, reassignment, nested-array, and ref-argument paths.
Fresh local array construction remains a later ownership/lowering handoff.

Recursive defaults are independently built from exact zero/null semantics.
They require a declared enum zero, eligible stored members, no required
members, and a nullable arm for reference defaults. Computed getters add no
stored zero cell. Shared default DAG nodes avoid exponential expansion through
repeated product members. Explicit struct constructors remain distinct from
`default(T)` and implicit zero construction. The exact nullable reference
default syntax also restores a Roslyn operation annotation omitted by the
preceding normalizer. Opaque sidecars cannot prove public invariants or role
defaults: affected source products and enums retain `default_invariant_pending`, do not
claim `default_eligible`, and cannot publish a default until the W04/W13 owners
resolve those claims. W03 emits no construction/invariant proof or success
artifact.

`ClassifySourceException` preserves the source ID and original immutable
syntax reference for a class with an exception ancestor, including candidates
whose actual base is not admissible. It neither validates a base clause nor
admits an exception. Such declarations cannot enter the ordinary data result;
T04-W04 remains the sole owner of source-exception base admission, diagnostics,
and lowering.

### 25.2 Executable evidence and ownership

`crates/mpk-cli/tests/csharp_practical_types.rs` is the primary owner. The
canonical `develop/migrations/csharp-03/types/types-inputs.json` binds the W03
implementation and harness plus the W01/W02 sources. The predecessor capture
and syntax manifests are rebound to the shared sources changed in this task;
the earlier completion sections retain their historical byte/hash receipts.
No installed project, active build manifest, bundle, registry, CLI/API route,
or application-visible MPK dependency includes the new implementation.

The executable harness tests the declaration/member/default matrix, exact
integer carrier boundaries and aliases, the closed framework enum, excluded
exception candidates and original-source handoff, forbidden declaration and
identity/mutation forms, immutable deterministic descriptor bytes, diagnostic
precedence, and original-source .NET runtime differential comparisons. It owns
all six frozen W03 limit vectors: 31/32/33 fields/properties and 15/16/17
structural member edges, including earlier-phase limit precedence.

Final manifest SHA-256 receipts:

| Manifest | SHA-256 |
| --- | --- |
| `capture-inputs.json` | `4a523fe7aed211aaf45a60b3c61bf90a68f7ef422f0a79929530d55980c1261f` |
| `syntax-inputs.json` | `79bee39a6967ad7baab82263fa349594d585b1f2b02f0651cbdc5035a0de450b` |
| `types-inputs.json` | `35b75c8bde037a3d7f49a966843b621b212d9530d00ef5431b2754b8895daa0f` |

### 25.3 Verification and review

The pinned Roslyn harness runs in the existing Linux/amd64 container with network
disabled, a read-only repository, and manifest-bound copied inputs. This is
private frontend evidence, not a native release-gate receipt.

| Command | Result |
| --- | --- |
| `cargo test -p mpk-cli --test csharp_practical_types --test csharp_practical_capture --test csharp_practical_syntax` | pass: all 15 owner tests on the final manifests |
| `docker run --rm --platform linux/amd64 ... scripts/build-csharp-practical-frontend.sh --test-types` | pass: pinned compiler and all W03 executable vectors |
| same closed container with `--test-capture` and `--test-syntax` | pass: predecessor executable regressions |
| `scripts/build-csharp-practical-frontend.sh --check-build-inputs` | pass |
| `scripts/build-csharp-practical-frontend.sh --self-test` | pass |
| `./scripts/check-fast.sh` | pass: formatting, strict obsolete-interface scan, warning-denied workspace lint, workspace tests, CLI build, and accept/reject fixture checks |
| `/usr/bin/git diff --check` | pass |

Review corrections cover nullable reference default annotation, nullable-enum
conversion escape, implicit enum zero construction, published-array aliases
and ref arguments, mutation after construction publishes an aliased array,
and scalar-copy versus shared-array alias classification,
phase-0 structural limit precedence, independent type failures in a batch
containing generics, and nullable-none eligibility with opaque sidecars.
Direct executable regressions cover each correction. The tests also distinguish C# compiler
rejections and full captured-call-closure requirements from implementation
failures. Status references in the cross-language design and both practical
plan summaries now consistently name W04 as the next serial item.

Final review findings: `0`. The final pass covers the full W03 diff,
source/runtime regressions, frozen limit and diagnostic precedence, immutable
type/default handoff, exception classification ownership, exact manifest-only
execution, predecessor preservation, and absence of any installed route.

T03-W04 is the sole ready item.

## 26. CSHARP-03-T03-W04 completion record

### 26.1 Private construction and call handoff

`csharp-tools/csharp2vir/PracticalConstruction.cs` composes the W01 capture,
W02 normalization, and W03 immutable type gates. The constructor counter runs
at phase 0, signature exclusions at phase 2, and construction analysis after
source-closure validation and before the phase-7 effect firewall. All failures
remain sanitized and artifact-free. This task installs no compiler project,
frontend route, bundle, registry entry, or success envelope.

The private constructor plans retain reachable CFG blocks, explicit assignment
and direct-read order, delegation targets, and incoming must/may assignment
sets. A fixed point covers branches, early exits, and back edges. Delegation
propagates explicit assignments; implicit CLR zero storage does not count as
a write. A compiler capture used exclusively as a direct assignment location
is distinguished from an unfinished receiver escape. Compound assignments and
increments cannot bypass assignment multiplicity. Missing member values need
eligible recursive zero storage; nullable-none remains a distinct value.
Uninvoked implicit struct zero constructors do not impose default eligibility
on an explicitly constructed, default-ineligible struct.

The exact synthesized parameterless class constructor and auto-property
accessors are cross-checked against emitted metadata and IL from the pinned
compiler: inert object base call, exact backing-field token, no locals or
exception regions, and no extra instructions. Getter and method bodies retain
W02 normalization. Instance functions expose receiver parameter zero; direct
call plans retain the actual immutable operation operands in receiver/argument
order and a separate null check at the language call point. Each call and
creation retains a source site so repeated uses are not collapsed.

Member-domain, recursive-default, public, and construction invariant
obligations remain undischargeable by this frontend handoff. Private typed
claims bind to an exact source type and captured source SHA-256, check Boolean
expression types and member attachment, require complete per-source-type
coverage when supplied, and retain every declared expression at its applicable
construction/default site. The private expression algebra covers Boolean/i32
literals, stored-member references, Boolean connectives, and exact scalar
equality; it is not a second sidecar format. T06-W01 still owns the frozen JSON
contract-expression parser and its broader vocabulary; opaque sidecar bytes
cannot assert a claim here. Declared claims leave recursive default eligibility
pending, including nested default obligations. T06-W02 owns semantic discharge.
W05 owns object-initializer and required/init completion; initializer source
forms still reject at this stage.

### 26.2 Executable evidence and scope

`crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W04` owns all three
frozen `constructors_per_type` vectors (7/8/9, inclusive maximum 8). The
canonical `develop/migrations/csharp-03/construction/construction-inputs.json`
binds all four private frontend sources and the construction harness. Shared
capture/syntax/types manifests are rebound; prior ledger receipts remain
historical. `--test-construction` uses the existing closed offline Linux/amd64
compiler/runtime runner, sanitized errors, and manifest-bound copied bytes.
The runner self-test mutates construction paths, hashes, sizes, and file count.

The executable matrix covers delegation, mutually exclusive and duplicate
writes, loop writes, conditional RHS captures, unfinished receiver/member
reads, exact signatures, synthesis and IL mutations, invariant attachment,
incomplete/ill-typed claims, pending defaults, getter/method normalization,
receiver-first runtime equivalence and argument-before-null-check behavior,
object-initializer handoff, and diagnostic precedence. Existing W01-W03
harnesses remain regression owners.

Final manifest SHA-256 receipts:

| Manifest | SHA-256 |
| --- | --- |
| `capture-inputs.json` | `2a2505d977225adb5c675adb7e8f2bf590adcccdd335650e5db03f3caab95f5f` |
| `syntax-inputs.json` | `4963b6ac17d6b3e80316ce6f8dbc0a9d21aa6c7a53e6e68d610d09cac1806727` |
| `types-inputs.json` | `ad297425d2e4c669df0df713e4d9f6326008a1a314a18278d7271a862a0c8162` |
| `construction-inputs.json` | `0545b787eaa58b635a2450d3dac88e31c72649ae9b7c66417925b8821fb0ccb8` |

### 26.3 Verification and review

Local verification uses `./scripts/check-fast.sh`, the construction/types/
capture/syntax pinned harnesses, `--check-build-inputs`, `--self-test`, the Rust
owner and inventory tests, and `/usr/bin/git diff --check`. Pinned harnesses run
with networking disabled and the repository mounted read-only; these are
private frontend checks, not a native release-gate receipt.

Review corrections cover compiler assignment-location captures, compound
writes with branching RHS expressions, unused implicit struct constructors,
non-returning delegation, source-bound typed claims, pending default eligibility and nested obligations,
exact synthesized IL, receiver nullability, and phase-ordered construction
failures. Each semantic correction has an executable regression.

| Verification | Result |
| --- | --- |
| `./scripts/check-fast.sh` | pass: format, strict source scan, warning-denied lint, workspace tests, CLI build, accepted/rejected fixtures |
| Rust capture/syntax/types owners | pass: 18 tests |
| Rust implementation inventory | pass: 4 tests |
| Pinned `--test-construction` | pass |
| Pinned `--test-capture`, `--test-syntax`, `--test-types` | pass |
| `--check-build-inputs` and `--self-test` | pass |
| `cargo fmt --all -- --check` and `/usr/bin/git diff --check` | pass |

Final review findings: `0`. The reviewed diff includes the private implementation,
all phase hooks, executable regression matrices, manifest-only build paths,
owner tests, and current task statuses. No frozen public schema or installed
route changes are included.

T03-W05 is the sole ready item.

## 27. CSHARP-03-T03-W05 completion record

### 27.1 Private initializer transactions

`CSharpPracticalInitialization` composes the private W04 construction gate in
`csharp-tools/csharp2vir/PracticalConstruction.cs`. W04 retains its earlier
initializer rejection boundary. W05 exposes immutable per-creation plans with
argument evaluations, a fresh transaction, the exact constructor call,
construction-invariant obligation, ordered RHS evaluations and init assignments,
public-invariant obligation, and finalization. The source site identifies an
expression; each execution of `Begin` allocates a fresh transaction. Only
`Finalize` publishes a value. Exceptional RHS paths discard the transaction,
produce no usable value, and skip later assignments. A constructor with no
normal exit never reaches initializer evaluation or publication.

The source member set and constructor must/may assignment summaries determine
required coverage and duplicate writes, including conditional and delegated
constructor assignments. Only directly declared init-auto properties are
initializer targets. Every creation independently finalizes, including one
without an initializer. Eligible defaults are checked for members left
unassigned at finalization. Init members may retain temporary zero storage at
constructor exit; this does not claim a completed default-eligible value.
W03's default gate makes a narrow handoff for initialized source structs, while
`default(T)` retains its existing eligibility checks. Required properties cannot
be assigned by a constructor or bypassed with source compiler-marker attributes.
Nested member/collection/indexer initializers and post-publication mutation
remain rejected. An explicit nested `new` creates its own independent transaction.

W02 normalizes the compiler's implicit initializer receiver as non-null only
when its exact enclosing object-creation operation establishes that provenance.
W04's pinned synthesized-constructor/backing-field/accessor IL validation and
W01's compiler-marker validation remain prerequisites. Construction and public
claims are attached to their exact constructor and creation sites; they remain
undischarged obligations owned by T06-W02. No installed project, active profile,
registry, CLI/API route, success envelope, or frozen public schema is changed.

### 27.2 Executable ownership and verification

`crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W05` owns the task.
The frozen package assigns no new numerical limit vectors to W05. The canonical
`develop/migrations/csharp-03/initialization/initialization-inputs.json` binds
four shared private frontend sources and the new executable harness. The shared
syntax/types/construction manifests are rebound; prior completion receipts
remain historical. `--test-initialization` uses the existing pinned offline
Linux/amd64 compiler/runtime runner with a read-only repository and manifest-bound
copied bytes. Its self-test mutates hashes, sizes, paths, and file count.

Tests cover source-order versus declaration-order assignments, constructor vs
initializer final-state equivalence, optional defaults, required class and
readonly-struct initialization, per-creation required coverage, constructor and
delegation duplicate writes, ordinary/custom setters, attribute bypasses,
nested member initializers, nested independent construction, published array
aliases, and constructor/public invariant attachment. A small independent scalar
plan interpreter verifies evaluation-once, no publication before finalization,
exceptional discard, and skipped later RHSs against original-source .NET
results. Runtime vectors distinguish division failure from later overflow and
check increment evaluation order. All failures remain sanitized and artifact-free.

Final manifest SHA-256 receipts:

| Manifest | SHA-256 |
| --- | --- |
| `syntax-inputs.json` | `aa260487a97e67f4a95935a673bb2a0d847ed804c25a02817db9028a8e33ead2` |
| `types-inputs.json` | `87f1f9766b94356f0762bdde556cc377db90fa6d0b0e85023883e34a0fab829f` |
| `construction-inputs.json` | `9253f3da8989c4bc6b6c49b0702fae10603af147ca5f7d7afee06f086c06cde9` |
| `initialization-inputs.json` | `85ab3f3efcce76f6e7315bf45569f516533f88c40e89b45dde8ad64f1d73a8fb` |

### 27.3 Review and verification

Review corrections cover temporary readonly-struct zero storage, exact
initializer-receiver nullability, and rejection tests that previously stopped
at an unrelated source-type cycle. The final receiver-escape regression uses
an acyclic type graph and requires the exact unfinished-receiver diagnostic.

| Verification | Result |
| --- | --- |
| `./scripts/check-fast.sh` | pass: format, strict source scan, warning-denied lint, workspace tests, CLI build, accepted/rejected fixtures |
| Rust capture/syntax/types owners | pass: 20 tests |
| Rust implementation inventory | pass: 4 tests |
| Pinned `--test-initialization` | pass |
| Pinned `--test-capture`, `--test-syntax`, `--test-types`, `--test-construction` | pass |
| `--check-build-inputs` and `--self-test` | pass |
| `cargo fmt --all -- --check` and `/usr/bin/git diff --check` | pass |

Final review findings: `0`. The full diff review covers transaction order,
required/default state, exceptional non-publication, invariant sites,
predecessor admission boundaries, exact private manifests and runners, and
consistent task statuses. The local pinned harnesses are private frontend
evidence, not a native release-gate receipt.

T03-W06 is the sole ready item.


## 28. CSHARP-03-T03-W06 completion record

### 28.1 Shared structural specialization

`generate_structural_program` specializes a concrete T02 type into a shared
DAG of scalar, stored-product, sequence, canonical-entry, and active-payload
recipes. Contract equality, canonical comparison and map/set ordering use the
same recursive evaluator. Decimal compares mathematical values across scale
and signed-zero encodings; GUID follows unsigned N-text field order; strings
compare UTF-16 code units; products retain stored declaration order. Nullable
references project to option values, with null first. Sequences compare elements
before length, sums compare tags before active payloads, money compares currency
before amount, and transitions compare state/events/response.

IEEE equality preserves non-reflexive NaN and equal signed zeros, including
nested values. The whole concrete type determines order eligibility before
value inspection: empty float sequences and absent float options still reject
canonical comparison. Closed-operation signature validation applies the same
matrix. Canonical map/set validation shares the comparison engine and rejects
unsorted and numerically duplicate decimal keys. Unknown types, construction
tokens, mismatched operand types and malformed values fail closed.

`PracticalStructural.cs` projects the W06 source types to the T02 descriptor
vocabulary without implementing a second comparison engine. The pinned source
fixture covers all 19 available primitive/business scalar types, enum aliases,
readonly structs, sealed classes, init properties, nullable references/value
types and array type declarations. The executable Roslyn harness checks its
exact projection against `source-routes.json`; the Rust routing test passes
those projections through T02 validation and the production generator. The
fixture's explicit pure helper retains its original captured body and direct
call. It is not replaced by a structural-equality assertion, and its
field-completeness is not assumed. Source primitive/string operators remain
captured operations. W06 also records a typed `structural_equal` handoff
for each source `==`/`!=`, retaining left then right evaluation exactly once,
nullable reference adaptation and final negation for `!=`. Exact ordinal
static and instance `String.Equals` overloads use that same handoff; instance
calls retain the receiver-null check after argument evaluation. CLR identity,
virtual equality and hashing reject.

The source normalizer handles Roslyn's implicit object conversions only for
reference/null equality by retaining the exact nullable reference operand type.
No object value or boxing form is added. `CSharpPracticalStructural.Validate`
is the explicit W06 entry; the earlier W03-W05 entry points preserve their
reference/null rejection boundary. W07-W13 must extend the same projection
and generator routing tests for newly admitted source forms; W14 owns final
actual-source completeness. Core proof discharge remains with T06. This work
adds no installed frontend, public schema, registry entry or success envelope.

### 28.2 Ownership and verification

`crates/mpk-cli/tests/csharp_practical_types.rs#CSHARP-03-T03-W06` executes all
23 frozen `ordering` vectors and owns the source-routing, finite-domain algebra,
recursive NaN, declaration-order, enum, decimal/GUID/null/UTF-16, canonical
collection and business/sum regressions. The pinned .NET checks mirror numeric
and ordinal corner rules under invariant, comma and Arabic decimal separators with a hostile negative sign. The new
`structural-inputs.json` binds production frontend code, source fixture,
projection and harness bytes. Existing capture/syntax/types/construction/initialization
manifests are rebound; earlier receipts remain historical. The wrapper's
`--test-structural` route uses the existing pinned offline Linux/amd64 runner.
Its self-test rejects changed hashes, sizes, paths and file count.

### 28.3 Review and verification

Review corrections preserve the earlier phase rejection boundary, add explicit
source operator/ordinal-equality handoffs with receiver-null timing, and peel
parenthesized invocations before the existing exact-intrinsic-argument check.
The source regressions execute the affected forms and retain identity/hash and
non-ordinal rejections. Total-key signature traversal memoizes shared type DAGs.
The full diff review covers shared recursive semantics, type-based rejection,
source projection/operation routing, private manifests/runners and task statuses.
Final review findings: `0`.

| Verification | Result |
| --- | --- |
| Rust capture/syntax/types owners | pass: 27 tests, including all 23 frozen ordering vectors |
| Rust implementation inventory | pass: 4 tests |
| Pinned `--test-structural` and `--test-capture` | pass, including parenthesized ordinal calls |
| Pinned `--test-syntax`, `--test-types`, `--test-construction`, `--test-initialization` | pass |
| `--self-test` and `--check-build-inputs` | pass |
| `cargo fmt --all -- --check` and `/usr/bin/git diff --check` | pass |

`./scripts/check-fast.sh` passed: formatting, strict source scan, warning-denied
lint, workspace tests, CLI build and accepted/rejected certificate fixtures.
The pinned tests run offline on Linux/amd64 with the repository read-only;
these are private frontend checks, not a native release-gate receipt.

Final manifest SHA-256 receipts:

| Manifest | SHA-256 |
| --- | --- |
| `capture-inputs.json` | `c8059dd9769bf284363f8012d64d80606a49203eb7a5f0d3239fdfd535dc9007` |
| `syntax-inputs.json` | `619673fd89eab5158f50ddd7e693808a93999ea5fc0715ff7a9dc58d98bd9054` |
| `types-inputs.json` | `cd23b3ad86322f029a25eaf002779f17ca6537febad923abacee8dfadd6980aa` |
| `construction-inputs.json` | `ab5f2e6709c6978aa01a5b8d5e8469c4240cde003b49de32cf43f29de20fde28` |
| `initialization-inputs.json` | `eb7318587c943267555f3d4cee5d99d58e273ee0b10c46fc4feb4d7ec557af4b` |
| `structural-inputs.json` | `172872a7ef8dbd2c191c51202c6094eec08a77f921ebb133b7b0328432114363` |

T03-W07 is the sole ready item.


## 29. CSHARP-03-T03-W07 completion record

### 29.1 Ordered source array handoff

`csharp-tools/csharp2vir/PracticalArrays.cs` adds the private W07 entry point.
It composes W04-W06 construction, initialization and structural capture, and
replaces the preliminary flow-independent array write barrier only for this
entry point. Earlier entry points retain their existing rejection boundaries.
The result preserves typed Roslyn source/operand operations, allocation
identities, branch paths and ordered steps; it emits no public artifact.

Each allocation evaluates its exact `int` length once, records the pinned
`OverflowException` edge for invalid C# lengths, then the separate inclusive
4096-element profile predicate and unique allocation. Constant lengths and
initializers above the profile maximum reject with `CSHARP_PRACTICAL_LIMIT`.
Input and result arrays retain profile-bound obligations, with reference
presence handled separately. Indexed accesses evaluate the array and exact
`int` index once, then record null, lower-bound and upper-bound edges. Simple
stores evaluate the RHS before those checks; compound stores read first.

Default-eligible elements use the frozen recursive default; zero length is
complete for every admitted element type. Initializers evaluate left to right
with element invariant obligations. Other arrays track definite and possible
initialized cells. Branch joins intersect definite initialization and union
possible initialization/freeze state. A partial array permits first writes
and initialized reads; complete unique arrays permit ordinary rewrites.
Symbolic writes retain initialization/completeness obligations, including the
conditional transition from first writes to rewrites, without claiming proof
discharge. Return, calls, storage and aliases require complete initialization
and freeze or transfer ownership. Unpublished storage is discarded on exit.

The exact explicit, local and identity-inferred one-dimensional forms are
covered. Unsupported dimensions, jagged/covariant storage, framework collection
and storage types, stack allocation, collection expressions, ranges/from-end,
non-int indices/lengths, `Array.Empty<T>()` and best-common-type inference reject
before emission. The T03 gate rejects loops and exception-control forms;
`RequireWritable` exposes the active-foreach read-borrow conflict to T04-W02.
W08 owns specialization into monomorphic construction state and wrapper
bindings. T04 owns positive loop/foreach lowering; T06 discharges obligations;
T07 remains the sole owner of public frontend activation.

### 29.2 Ownership and verification

`crates/mpk-cli/tests/csharp_practical_collections.rs#CSHARP-03-T03-W07`
executes the three frozen array-limit vectors through the production
monomorphic value validator and the shared W06 structural generator. The
exact private manifest `develop/migrations/csharp-03/arrays/arrays-inputs.json`
binds the seven compiler/harness input files.
`--test-arrays` uses the pinned SDK 10.0.400, Roslyn 5.6.0 and runtime 10.0.11
on offline Linux/amd64 with the repository read-only.

### 29.3 Review and verification

The review/fix iterations closed symbolic initialization/completion tracking,
RHS-before-store-check ordering, declaration-initializer publication handling,
phase-0 array-limit precedence, and untracked binding/control constructs such
as `??=` and patterns. Existing field-declaration initializer rejection remains
in force. Negative source tests reject internal protocol failures as test
failures, so adapter faults cannot masquerade as intentional source rejection.
The new runner exposed a shared Linux-only test issue: the archive inventory
was read from the vector package instead of the private build descriptor.
Capture, syntax, types and arrays owner tests now validate the correct archive
inventory before the host-specific execution guard. This regression is checked
on macOS as well as Linux.

The final review covers source evaluation order, branch ownership joins,
initialization/publication obligations, unsupported source gates, shared
consumer routing, exact manifests/runners and current task statuses.
Final review findings: `0`.

| Verification | Result |
| --- | --- |
| Rust capture/syntax/types/collections owners | pass: 30 tests, including all three W07 limit vectors |
| Rust implementation inventory | pass: 4 tests |
| Pinned arrays harness | pass: source forms, ownership, bounds, runtime exception timing and borrow handoff |
| Pinned types/construction/initialization/structural harnesses | pass after the shared callback changes |
| `--self-test` and `--check-build-inputs` | pass |
| `cargo clippy -p mpk-cli --tests -- -D warnings` | pass on final owner tests |
| `cargo fmt --all -- --check` and `/usr/bin/git diff --check` | pass |

`./scripts/check-fast.sh` passed: strict source scan, formatting, warning-denied
lint, workspace tests, CLI build, and accepted/rejected certificate fixtures.
Final owner-test runner changes were additionally checked by the 30 targeted
tests and warning-denied lint above. The formal `--test-arrays` script route
also passed with the final bound inputs.
These offline Linux/amd64 runs are private frontend verification; a native
release gate is not required for this change and was not run.

Final manifest SHA-256 receipts:

| Manifest | SHA-256 |
| --- | --- |
| `capture-inputs.json` | `c8059dd9769bf284363f8012d64d80606a49203eb7a5f0d3239fdfd535dc9007` |
| `syntax-inputs.json` | `619673fd89eab5158f50ddd7e693808a93999ea5fc0715ff7a9dc58d98bd9054` |
| `types-inputs.json` | `0c58e9609f8b7ed8dcf44c18881330e22f9029b464930b44b3f45d810f408575` |
| `construction-inputs.json` | `c79e2fd8f5acd48a57f5aa13312ab77a979973639e840e8373fd89e8fc7b1eca` |
| `initialization-inputs.json` | `8f0895cc311f0e0212bb8e5f7c81c7e0f08cb6a69fde7467e3d08062142baabe` |
| `structural-inputs.json` | `aa63a74e0c6a4ce1a136e7ce32589bc5c80fd5b3647f0f0a5c9fb942566a8c5f` |
| `arrays-inputs.json` | `f3367bc2319ebaf0303ead560b4da6d41280d48cdc7d9e33f19ba3af4e0a162d` |

T03-W08 is the sole ready item.

## 30. CSHARP-03-T03-W08 completion record

### 30.1 Private typed sequence substrate

`csharp-tools/csharp2vir/PracticalSequences.cs` consumes W07's captured ordered
array operations. Every allocation specializes `sequence_construction<T>` and
`bounded_sequence<T>` with an exact closed element type, including separate
nullable-reference presence. Read-only source arrays receive the same sequence
projection. Allocation, initializer indices, fill/rewrite, read, length,
freeze, immutable publication, discard and merge retain source operands,
method/path identity, ordered checks and pending invariant/profile predicates.
The independently regenerated candidate bytes detect altered types, operands,
checks, ownership routes and wrapper selections. W08 enforces the frozen 32
construction identities per method and 8 simultaneously live states.

W08's source gate checks construction ownership/version compatibility at branch
joins. W07's earlier entry point retains its original array analysis. Once an
alias/call/storage/return freezes an array, later read/publication operations
consume an immutable sequence and do not freeze a second construction token.
Bound wrappers must be selected immutable non-generic source products containing
exactly one non-null array and admitted scalar metadata. Binding a getter in
place of the stored array, another type/member, multiple arrays or unsupported
metadata rejects. Constructor, getter, invariant and helper source remains
captured for verification; a projection is not a proof of those bodies.

`crates/mpk-vc/src/csharp_practical_sequences.rs` composes the existing shared
`SequenceConstructionState` validator with concrete element storage. Typed
fills (including explicit recursive defaults), reads, rewrites, borrow/transfer,
freeze and discard revalidate ownership and initialization before mutation.
Invalid values and transitions leave the state unchanged. The 16384 internal
capacity never raises the 4096 array/wrapper publication bound. Finishing a
method rejects every residual active construction, and successful finishing
returns only immutable monomorphic values. Wrapper projection revalidates the
source content hash, exact member and semantic type against validated roots,
then uses the common structural equality and conditional ordering generator.
Symbolic obligations remain pending for T06; concrete execution cannot bypass
them by claiming default eligibility or importing initialized-state flags.

### 30.2 Verification ownership and serial boundary

`crates/mpk-cli/tests/csharp_practical_collections.rs#CSHARP-03-T03-W08`
owns all nine frozen capacity/state-count vectors, zero/full publication,
straight-line initialization, invalid element/read/publication, transfer,
incompatible merge, residual-state and wrapper-binding mutations. The pinned
Roslyn harness regenerates `source-direct.json` from an actual direct-result
initializer; the Rust owner replays those exact types, length and element
values through construction elimination and checks deterministic results.

`develop/migrations/csharp-03/sequences/sequences-inputs.json` binds the private
compiler/harness and source-replay inputs. `--test-sequences` uses the same
SDK 10.0.400, Roslyn 5.6.0 and runtime 10.0.11 offline Linux/amd64 runner as W07.
The T03 source loop gate remains artifact-free: filtered count/allocate/fill
loops, framework collections, lambdas and source-visible builders reject.
Every positive two-pass source form belongs to T04-W01/W02; emitted data/loop
proofs belong to T06-W03/W04. No installed compiler route, public format,
Certificate v0 rule or checker acceptance changes.

### 30.3 Review and verification

Review found and corrected missing read-only array type projection, incompatible
construction merges, repeated freeze after immutable alias publication, and
missing value revalidation on length/read. Array and wrapper projections now
normalize to the same immutable representation before shared comparison. The
existing capture-route count test includes the new private runner. Targeted
regressions exercise each corrected behavior. Final task-diff review findings:
`0`.

The unfiltered `./scripts/check-fast.sh` passed the strict source scan,
formatting and warning-denied lint, then exposed two **pre-existing** historical
inventory failures: `csharp_03_t01_w02_inventory_closes_every_artifact_and_consumer_edge`
and `csharp_03_t01_w02_search_fixtures_reject_added_or_deleted_consumers`.
For every frozen search fixture, a comparison of the working-tree path set
against `/usr/bin/git grep` at unchanged HEAD
`dc36a561b7b445d450374663af8a96aa63ebad5e` found **zero changed path sets**.
Both have the same 17 fingerprint mismatches. For example, `registry.literal`
expects 400 paths but both HEAD and this change contain 398, with path-set
SHA-256 `d99eb1c12d7eb5c51fb891b19bc6cbfa2d0a2f5458ee0c19c38d11c8593824ec`.
No frozen inventory/specification was rewritten to conceal that baseline drift.
Those two baseline failures remain outside W08's implementation scope.

| Verification | Result |
| --- | --- |
| `cargo test -p mpk-cli --test csharp_practical_collections --test csharp_practical_capture` | pass: 18 tests, including all nine W08 boundary vectors |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass on final code |
| Offline pinned `--test-sequences` and `--test-arrays` | pass |
| Offline pinned capture/syntax/types/construction/initialization/structural runners | pass |
| Offline `--self-test` and `--check-build-inputs` | pass |
| Unfiltered `./scripts/check-fast.sh` | fails only at the two unchanged baseline inventory tests described above |
| `cargo test --workspace -- --skip csharp_03_t01_w02_inventory_closes_every_artifact_and_consumer_edge --skip csharp_03_t01_w02_search_fixtures_reject_added_or_deleted_consumers` | pass: all remaining workspace tests and doc tests |
| `cargo build -p mpk-cli` and the four `check-fast.sh` accepted/rejected certificate fixtures | pass |
| `cargo fmt --all -- --check` and `/usr/bin/git diff --check` | pass |

These are private compiler checks on Linux/amd64, not a native release gate.
No release gate was required or run.

Final private input-manifest SHA-256 receipts (shared source dependencies were
rebound without changing the installed build inputs):

| Manifest | SHA-256 |
| --- | --- |
| `capture-inputs.json` | `35a1dbe8688a8bd3fedc772ffe64a41e852a2268c1ba855d7fc5007548e3e71c` |
| `syntax-inputs.json` | `a3eff14b176eff38e504524da46f371fc3de42f4fb16a09e3c9aae3c3d4bc134` |
| `types-inputs.json` | `b3d4a05308ca98fb0dbae3ced67a45b155393409a4904703acf372ec6dc36064` |
| `construction-inputs.json` | `0053f2f20800ae11e78714223c24ae5f3b60039b60d44ace13b944f7165b906b` |
| `initialization-inputs.json` | `90c1c826ccb6b1ea76b0488efcdb5efbce1bd5c80f87cfd6b03e7058ed49dc4a` |
| `structural-inputs.json` | `9e7288c81aa0a438172e46c2b27005a6756b714697fb1cafd9507cb3be055d36` |
| `arrays-inputs.json` | `1cc274414d9557bfb0f1dbc466f6bbeba911d893d3f7023b1cd944b03c717782` |
| `sequences-inputs.json` | `2528e80d0ffc33e0689817f87be9a5091bc52a64d4a0a1e77accc2fa2ac1a09e` |

T03-W09 is the sole ready item.

## 31. CSHARP-03-T03-W09 completion record

Completed 2026-09-06. The private ordered-collection adapter consumes W08 array
plans and frozen typed binding roles for admitted element sets, immutable
non-generic source-entry maps, and optional non-generic wrappers. It records
exact key/value/entry/lookup identities, captured source hashes and the seven
binding obligations, including reconstruction of every stored extra field.
Every input/publication boundary retains pending length, total-cell, strict
ordering, uniqueness and public-element-invariant obligations. Projection
never sorts or removes duplicates; no source helper is replaced.

The shared Rust model revalidates concrete entry/member/content bindings,
projects canonical arrays/wrappers, uses the common key comparator and exposes
exact typed operation signatures. It distinguishes `missing_key` from
`found(none)`, rejects non-total key types, and offers map comparison only when
both keys and values are total. Concrete specification relations preserve
invalid-representation/duplicate/capacity precedence and existing-key replacement.
Their execution is private model evidence, not source-loop acceptance or a
commutation proof. Source count/membership/lookup/traversal/aggregation and
insertion/replacement/sort/dedup loops remain T04-W01/W02 work; proof discharge
remains T06-W03/W04. Framework collections, comparers, hashing, mutable/generic
entries, computed binding members and source loops reject artifact-free.

### 31.1 Verification ownership and review

`crates/mpk-cli/tests/csharp_practical_collections.rs#CSHARP-03-T03-W09`
owns all eight frozen collection vectors and six map/set/total-cell boundary
vectors. The error-list vector remains an ordinary sequence with order and
duplicates preserved; it is never normalized as a set. Tests cover direct
publication, binding/hash/member mutations even for empty arrays, duplicate
and reversed keys, nullable keys and values, decimal equivalent keys, the
canonical key-type matrix, conditional comparison with float values and exact
operation handoff mutations. Positive source-loop vectors remain assigned to
the control owner.

`develop/migrations/csharp-03/ordered/ordered-inputs.json` binds every private
compiler, harness and `source-ordered.json` input. The pinned Roslyn harness
regenerates the map/set identity and operation fixture from captured source;
the Rust owner compares it with registered foundation expansion, including
argument/result types and outcome order. `--test-ordered` uses the unchanged
offline Linux/amd64 SDK 10.0.400, Roslyn 5.6.0 and runtime 10.0.11 isolation.
Shared capture identity changes rebind the private phase manifests only.
Installed compiler inputs, active profiles and Certificate v0 remain unchanged.

Review caught an invalid lookup-type assumption: dependency IDs are stored in
canonical sorted order, not recipe order. Lookup now uses the exact expanded
operation result type; nullable-found and missing regressions exercise the fix.
Review also moved effective-capacity handling behind the duplicate decision:
an application capacity can only tighten the frozen maximum, and replacement
never consults it. Full-set, oversized-capacity and replacement regressions
exercise the precedence. Final task-diff review findings: `0`.

### 31.2 Local verification

The two historical T01-W02 inventory fingerprint failures recorded in section
30 remain baseline failures. The frozen inventory is not rewritten by W09.
Every frozen search path set matches unchanged HEAD
`b2ecf2949c82bb6687b0916e292f4f4bfbcbdc86`; both have the same 17 fingerprint
mismatches. The standard gate passed formatting, the strict obsolete-interface
scan and warning-denied lint, then failed only the two tests named in section 30.

| Verification | Result |
| --- | --- |
| `cargo test -p mpk-cli --test csharp_practical_collections --test csharp_practical_capture` | pass: 26 tests |
| Pinned offline `--test-ordered` plus W01–W08 harness regressions | pass |
| Offline `--self-test` and `--check-build-inputs` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| Unfiltered `./scripts/check-fast.sh` | two unchanged baseline inventory failures |
| Workspace tests with only those two baseline tests skipped | pass: all remaining tests and doc tests |
| CLI build and four accepted/rejected certificate fixtures | pass |

These are private Linux/amd64 compiler checks; no release gate was required or
run. New ordered-input manifest SHA-256: `5c08ce86376b9e58cff88968b12d3d3e1e69b88373ec9e11c25112707820c167`.

T03-W10 is the sole ready item.

## 32. CSHARP-03-T03-W10 completion record

Completed 2026-09-06. The private source-string adapter retains exact Roslyn
operands, evaluation order, ordinal overloads, UTF-16 constants, null/range
exception types and pending input/output length obligations. String/string and
string/char concatenation and restricted string/char interpolation share the
bounded UTF-16 concatenation relation. The source firewall admits the exact
string indexer and normalizes compiler-generated boxing for string/char concatenation;
explicit object conversion, char/char, numeric/aligned/formatted interpolation,
source parse/format, non-ordinal options and general framework APIs reject.

The Rust UTF-16 relation reproduces ordinal comparison/equality, null/empty,
contains/prefix/suffix, length, index, substring and concatenation without UTF-8
scalar replacement. Null receiver, null argument, index and substring errors
retain their exact C# exception subtypes; a profile/output bound is a pending
obligation, never an invented resource exception. Constant string switches and
source-loop control remain T04 work.

One type-dispatched `BoundaryCodec` relation handles the 18 frozen codec IDs:
signed/unsigned integers, normalized/fixed decimal, binary32/binary64 bits,
Gregorian date, seven-digit time, duration ticks, instant milliseconds and GUID
N/D. It uses bounded ASCII/UTF-16 and integer algorithms, retains every float
bit pattern and all five decimal rounding modes, and returns registered
`result<T,parse_error>` values. Unknown IDs/rounding configurations and crossed
concrete result/operand types reject. Parser precedence remains input bound,
syntax, canonicality, scale/precision and range; invalid fixed scale is checked
after syntax/canonicality, while format configuration rejects before output.
W11/W13 reuse this relation for their new source coverage. Money fields and
presence/outcome/document mapping remain T05-W01-W03 work. No installed route,
active release tuple, public document invocation or Certificate v0 rule changes.

### 32.1 Ownership, review and verification

`crates/mpk-cli/tests/csharp_practical_codecs.rs#CSHARP-03-T03-W10` owns the
three frozen string-length boundary vectors, all 396 recorded codec cases in
each of three hostile cultures, and 63 direct string runtime cases. Tests
exercise grammar/error precedence, decimal precision reduction at the 96-bit
limit, signed zero/NaN bits, round trips, surrogate/null handling, conditional
instance-equality null checks, concatenation equivalence and output bounds.
Contract mutation tests reject wrong codec value types and excess arguments.

`develop/migrations/csharp-03/codecs/codecs-inputs.json` binds all private source,
harness and `source-strings.json` inputs. The owner checks that the string
fixture is exactly the selected frozen runtime corpus. The new `--test-codecs`
runner compiles real admitted/rejected source and executes the string corpus
under three constructed hostile cultures using the unchanged offline
Linux/amd64 SDK 10.0.400, Roslyn 5.6.0 and runtime 10.0.11 closure. Existing
private manifests rebind their changed shared capture/normalizer/array inputs.

Review corrected missing string-indexer admission, compiler-generated char
boxing, overlapping expression-site suppression, and untyped codec signature
acceptance. Targeted source/runtime and contract mutation tests cover these
corrections. The final full-diff review found no remaining findings.

| Local verification | Result |
| --- | --- |
| `./scripts/check-fast.sh` | Formatting, strict active-GIR scan and clippy passed; workspace tests stopped at the two pre-existing T01-W02 inventory fingerprint failures described below. |
| `cargo test -p mpk-cli --test csharp_practical_codecs --test csharp_practical_capture` | 13 passed; the macOS harness availability guard is supplemented by the explicit Linux execution below. |
| `cargo test -p mpk-vc --test csharp_practical_vir_model` | 9 passed, including the existing operation-signature checks. |
| `cargo test --workspace -- --skip csharp_03_t01_w02_inventory_closes_every_artifact_and_consumer_edge --skip csharp_03_t01_w02_search_fixtures_reject_added_or_deleted_consumers` | 792 passed, 8 existing platform-dependent tests ignored, and only the two named baseline failures filtered out. |
| Offline Linux/amd64 private harnesses `--test-capture`, `--test-syntax`, `--test-types`, `--test-construction`, `--test-initialization`, `--test-structural`, `--test-arrays`, `--test-sequences`, `--test-ordered`, `--test-codecs` | All passed against the pinned closure; final W10 source-operation assertions were rerun with `--test-codecs` and passed. |
| Private `--self-test` and `--check-build-inputs` | Passed, including source, fixture and manifest mutation checks. |
| `cargo build -p mpk-cli` and the four `check-fast.sh` CLI fixtures | Build passed; both positive fixtures accepted and both negative fixtures rejected. |
| `/usr/bin/git diff --check` | Passed. |

The unfiltered gate still fails
`csharp_03_t01_w02_inventory_closes_every_artifact_and_consumer_edge` and
`csharp_03_t01_w02_search_fixtures_reject_added_or_deleted_consumers` on the
same 17 frozen search fingerprints reported by W09. Comparing every inventory
search against parent `6714e88e963fba6f06c050989072a2cedfeefedd` found no changed
path sets; all 17 mismatches already exist at that parent. The frozen inventory
was not rewritten. The full standard gate therefore remains non-green despite
the passing W10 and remaining-workspace verification. No release gate was
claimed or required for this private handoff.

Final private codec input manifest SHA-256:
`f84a880942694835781db682cfcb4ed6c4bbab71a127bf5e8319d923c89484c5`.

T03-W11 is the sole ready item.
