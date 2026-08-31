# JAVA-03 Implementation Traceability Ledger

Status: `JAVA-03-T01` through `JAVA-03-T06` complete (2026-08-31);
T07's private candidate/JVM runner is implemented, but native x86-64 Linux
acceptance is still pending. T08 through T10 are pending. Neither private
artifacts nor emulated tests establish native release isolation or Java
activation. The active release remains
Go/Rust/C# at registry revision 2.

This is an execution plan subordinate to `../specs/JAVA_PROFILE_V0.md`, the
exact Java/revision-3 vectors, and `SEMANTIC_PROFILE_REGISTRY_V1.md`. It does
not change immutable contracts or checker acceptance. Each requirement and
vector family below has one primary implementation owner; downstream users
consume that owner's result without redefining it.

## 1. Serial task graph and bounded deliverables

`CSHARP-02-T20 -> JAVA-03-T01 -> T02 -> T03 -> T04 -> T05 -> T06 -> T07 -> T08
-> T09 -> T10 -> DART-04`. Every task requires its predecessor's completed,
reviewed result. T01 froze the profile; T02 added the inactive offline build;
T03 added the inactive profile and artifact validators. T04 added capture,
the public compiler adapter and bounded failure diagnostics. T05 added source
admission, inert initialization, acyclic call closure and typed sidecars.
T06 added deterministic CFG/lowering, original-byte source maps and complete
frontend-stage manifests and success envelopes through a private pipeline.

| Task | Scope | Exit evidence |
| --- | --- | --- |
| `JAVA-03-T01` | frozen normative profile, exact vectors/hashes, JDK inventory, disposable API/JVM compatibility probes, this ledger | all 34 rows classified, no guessed digests/API observations, precise limitations, local checks and zero-finding specification review |
| `JAVA-03-T02` | inactive `java-tools/java2vir` source/build project and offline build script | exact pinned JDK, deterministic JAR/source/class inventory, two isolated builds, no active Java route |
| `JAVA-03-T03` | inactive compiled registry/parameter/selection/nine payload and source-artifact validators | all identity/hash/mutation cases, exact predecessor preservation, closed Java VIR/VC/map/manifest admission |
| `JAVA-03-T04` | capture, public compiler adapter, application-file-manager closure, diagnostics/transport | every capture/encoding/API/diagnostic vector, source-dead tree inventory, no ambient dependencies or generated output |
| `JAVA-03-T05` | source subset, inert interfaces, local rules, source-call closure, typed sidecars | every source rejection and contract case, explicit 255-unit signature check, every selected file/method/sidecar accounted for |
| `JAVA-03-T06` | CFG/lowering, stable IDs, maps and manifests | all operations/checks/conversions/CFG patterns, source origins, deterministic complete artifacts |
| `JAVA-03-T07` | candidate bundles and registered JVM sandbox branch | native inventory, exact launcher, hostile environment, cgroup/resource/process/filesystem/network enforcement and cleanup |
| `JAVA-03-T08` | VC/policy/evidence/certificate/AI/API integration | Java obligations and source-free same-byte dual-checking, complete context propagation, provider redaction, no public activation |
| `JAVA-03-T09` | complete conformance/differential/fuzz/upgrade/release rehearsal | every owning vector executor runs, two builds/runs, bounded recorded fuzz seeds, native Linux rehearsal, unchanged predecessor semantics/axioms |
| `JAVA-03-T10` | atomic release/docs/examples/gate cutover | revision 3 only, all five tuples, no executable staging/public compatibility route, complete local four-language Linux release gate |

T02-T09 may add private candidate harnesses. They cannot install a Java tuple,
discover a staging root from the public CLI/API, dual-load registry revisions,
accept raw toolchain paths, or change active release descriptors. T10 removes
executable staging entrypoints; archived probe evidence may remain.

## 2. Normative requirement ownership

Subrows divide a section only where the implementation boundary is distinct;
no requirement is assigned twice. "Consumer" means later use, not a second
primary owner.

| Spec requirement | Primary task | Implementation location | Owning test |
| --- | --- | --- | --- |
| 1: identity, parameters, registry-owned closed values | T03 | `crates/mpk-vc/src/semantic_profile_registry.rs` | `crates/mpk-vc/tests/java_profile_vectors.rs` |
| 2: selection validation | T03 | semantic registry finite Java selection validator | `crates/mpk-vc/tests/java_profile_vectors.rs` |
| 2: immutable files, path/type/inventory capture | T04 | `java-tools/java2vir/` capture boundary | `crates/mpk-cli/tests/java_frontend_vectors.rs` |
| 3: declaration/name/signature units/initialization/closure | T05 | `java-tools/java2vir/` subset/call closure | `crates/mpk-cli/tests/java_subset.rs` |
| 4: statement/expression/literal/type/conversion admission | T05 | Java source subset | `crates/mpk-cli/tests/java_subset.rs` |
| 5: emitted operations and exact ordered checks | T06 | Java lowering | `crates/mpk-cli/tests/java_lowering.rs` |
| 5: independent VIR/VC validation of admitted operations | T03 | `successor_source_artifacts.rs`, `safety_check.rs` | `crates/mpk-vc/tests/java_profile_vectors.rs` |
| 6: parse/analyze API, raw/tree/type/source-manager observations | T04 | Java compiler session | `crates/mpk-cli/tests/java_frontend_vectors.rs` |
| 6: canonical CFG and value/block IDs | T06 | Java CFG/lowering | `crates/mpk-cli/tests/java_lowering.rs` |
| 7: strict sidecar attachment, typing, normalization | T05 | Java contract adapter | `crates/mpk-cli/tests/java_contracts.rs` |
| 7: callee WP, body obligations and certificate assembly | T08 | `successor_vc.rs`, policy/certificate paths | `crates/mpk-cli/tests/java_policy_verify.rs` |
| 8: original UTF-8 transport | T04 | Java source decoder | `crates/mpk-cli/tests/java_frontend_vectors.rs` |
| 8: origin/manifest/artifact emission | T06 | Java emission | `crates/mpk-cli/tests/java_source_maps.rs` |
| 8: independent artifact/context linkage | T03 | `successor_source_artifacts.rs` | `crates/mpk-vc/tests/java_profile_vectors.rs` |
| 9: phases/normalized diagnostics/artifact-free failures | T04 | Java protocol/diagnostic adapter | `crates/mpk-cli/tests/java_frontend_vectors.rs` |
| 10: logical counters in capture/adapter | T04 | Java bounded transport/compiler traversal | `crates/mpk-cli/tests/java_frontend_vectors.rs` |
| 10: subset/contract counters | T05 | Java subset/contract parser | `crates/mpk-cli/tests/java_contracts.rs` |
| 10: lowering/emission counters | T06 | Java lowering and serialization | `crates/mpk-cli/tests/java_lowering.rs` |
| 11: exact offline input/build/JAR inventory | T02 | `scripts/build-java-frontend.sh`, Java build inputs | `crates/mpk-cli/tests/java_build_inputs.rs` |
| 11: registered JVM/native/host execution closure | T07 | `release_bundle_v1.rs`, `frontend_sandbox.rs` | `crates/mpk-cli/tests/java_frontend_runner.rs` |
| 12: nine compiled payloads/root/hash contracts | T03 | semantic registry/release source-artifact validators | `crates/mpk-vc/tests/java_profile_vectors.rs` |
| 12: policy/evidence/AI/API payload consumers | T08 | CLI successor routes, `successor_api.rs` | `crates/mpk-cli/tests/java_policy_verify.rs` |
| 13: aggregate conformance/upgrade/differential/fuzz | T09 | Java rehearsal/corpus tooling | `crates/mpk-cli/tests/java_release_gate.rs` |
| 13: production activation/rollback/release ownership | T10 | installed release registry and CLI/API/docs | `crates/mpk-cli/tests/java_activation.rs` |

Shared scalar, certificate and checker code may be reused, but public Java
artifacts retain Java context throughout. No Java source axiom is introduced.

## 3. Vector field and payload ownership

The T01 spec model `crates/mpk-vc/tests/java_profile_spec.rs` owns frozen data,
strict JSON/hash consistency and append-only registry review. It is not a
production executor. The table records the primary *implementation* executor
that must subsequently execute the frozen requirement.

| Java vector field/family | Primary task | Owning test |
| --- | --- | --- |
| schema/owner/spec/mechanism metadata and `profile_identity` | T03 | `crates/mpk-vc/tests/java_profile_vectors.rs` |
| `semantic_parameters`, `semantic_context_fixture`, `selection_fixture`, `selection_sha256` | T03 | `crates/mpk-vc/tests/java_profile_vectors.rs` |
| `contract_fixture`, `contract_sidecar_sha256`, `normalized_contract_fixture` | T05 | `crates/mpk-cli/tests/java_contracts.rs` |
| `toolchain_inputs` | T02 | `crates/mpk-cli/tests/java_build_inputs.rs` |
| `compiler_session`, `adapter_observations` | T04 | `crates/mpk-cli/tests/java_frontend_vectors.rs` |
| `launcher_contract`, `host_probe`, `isolation_cases` | T07 | `crates/mpk-cli/tests/java_frontend_runner.rs` |
| `case_harness` | T09 | `crates/mpk-cli/tests/java_release_gate.rs` |
| `profile_contracts`, `shared_envelope_limits`, identity/payload `mutation_cases`, `hash_cases` | T03 | `crates/mpk-vc/tests/java_profile_vectors.rs` |
| `type_mappings`, `conversion_rules`, `semantic_rows` | T05 | `crates/mpk-cli/tests/java_subset.rs` |
| `operation_mappings`, `cfg_patterns` | T06 | `crates/mpk-cli/tests/java_lowering.rs` |
| `accepted_cases` | T06 | `crates/mpk-cli/tests/java_lowering.rs` |
| `rejected_cases` | ID-level division below | exact one owner per row below |
| `precedence_cases`, `diagnostic_registry`, `diagnostic_normalization` | T04 | `crates/mpk-cli/tests/java_frontend_vectors.rs` |
| `limit_cases` | counter division below | exact one owner per counter below |
| `source_map_cases` | T06 | `crates/mpk-cli/tests/java_source_maps.rs` |
| `upgrade_cases` | T09 | `crates/mpk-cli/tests/java_release_gate.rs` |
| registry-v3 metadata/predecessor/entry/root/hash/append-only/mutation families | T03 | `crates/mpk-vc/tests/java_profile_vectors.rs` |
| registry-v3 `activation_cases` | T10 | `crates/mpk-cli/tests/java_activation.rs` |

All nine payload validators have T03 as the primary owner. Their consumers
are `vir`/`source_map`/`manifest` T06; `vc`/`policy`/`evidence`/`ai` T08;
`frontend`/`release` T07. Each payload field has exactly the member-level
value in the vector; a later consumer may not add a field or reinterpret it.

## 4. Matrix and source-case ownership

The 34 matrix rows are owned by T05's source admission gate. T06 consumes the
17 admitted forms; T04 supplies the pinned compiler facts for M33 and T05's
contract parser supplies M34. A matrix admission is not a proof verdict.

| Matrix row | Frozen disposition |
| --- | --- |
| `M01` | `accept_under_profile_restrictions`: boolean |
| `M02` | `accept_under_profile_restrictions`: signed-bv32-bv64 |
| `M03` | `reject_before_vir`: outside-java-scalar-v0 |
| `M04` | `reject_before_vir`: outside-java-scalar-v0 |
| `M05` | `reject_before_vir`: outside-java-scalar-v0 |
| `M06` | `reject_before_vir`: outside-java-scalar-v0 |
| `M07` | `accept_under_profile_restrictions`: left-to-right |
| `M08` | `accept_under_profile_restrictions`: boolean-operations |
| `M09` | `accept_under_profile_restrictions`: signed-integer-comparison |
| `M10` | `accept_under_profile_restrictions`: short-circuit-cfg |
| `M11` | `accept_under_profile_restrictions`: acyclic-cfg |
| `M12` | `accept_under_profile_restrictions`: initialized-locals |
| `M13` | `accept_under_profile_restrictions`: wrapping-arithmetic |
| `M14` | `reject_before_vir`: outside-java-scalar-v0 |
| `M15` | `reject_before_vir`: outside-java-scalar-v0 |
| `M16` | `accept_under_profile_restrictions`: divisor_nonzero-only |
| `M17` | `reject_before_vir`: outside-java-scalar-v0 |
| `M18` | `accept_under_profile_restrictions`: integer-bitwise |
| `M19` | `accept_under_profile_restrictions`: masked-int-count |
| `M20` | `reject_before_vir`: outside-java-scalar-v0 |
| `M21` | `accept_under_profile_restrictions`: signed-extension-and-truncation |
| `M22` | `reject_before_vir`: outside-java-scalar-v0 |
| `M23` | `reject_before_vir`: outside-java-scalar-v0 |
| `M24` | `reject_before_vir`: outside-java-scalar-v0 |
| `M25` | `reject_before_vir`: outside-java-scalar-v0 |
| `M26` | `reject_before_vir`: outside-java-scalar-v0 |
| `M27` | `accept_under_profile_restrictions`: acyclic-source-static-call |
| `M28` | `reject_before_vir`: outside-java-scalar-v0 |
| `M29` | `accept_under_profile_restrictions`: dominating-local-assignment |
| `M30` | `reject_before_vir`: outside-java-scalar-v0 |
| `M31` | `reject_before_vir`: outside-java-scalar-v0 |
| `M32` | `reject_before_vir`: outside-java-scalar-v0 |
| `M33` | `accept_under_profile_restrictions`: exact-toolchain-and-runtime |
| `M34` | `accept_under_profile_restrictions`: typed-sidecar |

Each accepted case is primarily executed by T06's `java_lowering.rs`, using
its exact sources/selection/contracts and validating full canonical artifacts,
operation/check projection and evaluation expectations. T09 runs the same
cases through independent compiled-Java differential execution in disposable
offline sandboxes, with zero-divisor runtime failure treated separately from
total VIR values.

| Accepted case ID | Primary owner |
| --- | --- |
| `boolean.identity` | T06 / `java_lowering.rs` |
| `int.identity` | T06 / `java_lowering.rs` |
| `int.minimum` | T06 / `java_lowering.rs` |
| `int.wrap_add` | T06 / `java_lowering.rs` |
| `int.wrap_sub` | T06 / `java_lowering.rs` |
| `int.wrap_mul` | T06 / `java_lowering.rs` |
| `int.bitand` | T06 / `java_lowering.rs` |
| `int.bitor` | T06 / `java_lowering.rs` |
| `int.bitxor` | T06 / `java_lowering.rs` |
| `int.negate` | T06 / `java_lowering.rs` |
| `int.bitnot` | T06 / `java_lowering.rs` |
| `int.division` | T06 / `java_lowering.rs` |
| `int.remainder` | T06 / `java_lowering.rs` |
| `int.shift_left` | T06 / `java_lowering.rs` |
| `int.shift_right` | T06 / `java_lowering.rs` |
| `int.shift_unsigned_right` | T06 / `java_lowering.rs` |
| `long.identity` | T06 / `java_lowering.rs` |
| `long.minimum` | T06 / `java_lowering.rs` |
| `long.wrap_add` | T06 / `java_lowering.rs` |
| `long.wrap_sub` | T06 / `java_lowering.rs` |
| `long.wrap_mul` | T06 / `java_lowering.rs` |
| `long.bitand` | T06 / `java_lowering.rs` |
| `long.bitor` | T06 / `java_lowering.rs` |
| `long.bitxor` | T06 / `java_lowering.rs` |
| `long.negate` | T06 / `java_lowering.rs` |
| `long.bitnot` | T06 / `java_lowering.rs` |
| `long.division` | T06 / `java_lowering.rs` |
| `long.remainder` | T06 / `java_lowering.rs` |
| `long.shift_left` | T06 / `java_lowering.rs` |
| `long.shift_right` | T06 / `java_lowering.rs` |
| `long.shift_unsigned_right` | T06 / `java_lowering.rs` |
| `boolean.operations` | T06 / `java_lowering.rs` |
| `integer.comparisons` | T06 / `java_lowering.rs` |
| `control.join` | T06 / `java_lowering.rs` |
| `control.early_return` | T06 / `java_lowering.rs` |
| `control.ternary` | T06 / `java_lowering.rs` |
| `control.constant_condition` | T06 / `java_lowering.rs` |
| `control.short_circuit_division` | T06 / `java_lowering.rs` |
| `conversion.widen_initializer` | T06 / `java_lowering.rs` |
| `conversion.widen_assignment` | T06 / `java_lowering.rs` |
| `conversion.widen_return` | T06 / `java_lowering.rs` |
| `conversion.explicit_widen` | T06 / `java_lowering.rs` |
| `conversion.explicit_narrow` | T06 / `java_lowering.rs` |
| `conversion.identity` | T06 / `java_lowering.rs` |
| `literal.parenthesized` | T06 / `java_lowering.rs` |
| `call.direct` | T06 / `java_lowering.rs` |
| `call.dead_branch` | T06 / `java_lowering.rs` |
| `call.ordered_arguments` | T06 / `java_lowering.rs` |
| `call.multiple_entrypoints` | T06 / `java_lowering.rs` |

Rejection vectors with explicit source bytes use those bytes. Mutation-only
vectors modify their named accepted baseline; they are concrete adversarial
harness obligations, not proof that the T01 model ran production Java. Source
errors remain distinct from unsupported valid forms and adapter failures.

| Rejection case ID | Primary task | Owning test |
| --- | --- | --- |
| `local.uninitialized` | T05 | `java_subset.rs` |
| `local.parameter_assignment` | T05 | `java_subset.rs` |
| `local.repeated_names` | T05 | `java_subset.rs` |
| `local.multiple_declarators` | T05 | `java_subset.rs` |
| `local.final` | T05 | `java_subset.rs` |
| `local.var` | T05 | `java_subset.rs` |
| `local.assignment_expression` | T05 | `java_subset.rs` |
| `local.increment` | T05 | `java_subset.rs` |
| `local.compound_assignment` | T05 | `java_subset.rs` |
| `control.loop` | T05 | `java_subset.rs` |
| `control.empty_statement` | T05 | `java_subset.rs` |
| `control.switch` | T05 | `java_subset.rs` |
| `control.throw` | T05 | `java_subset.rs` |
| `control.try` | T05 | `java_subset.rs` |
| `control.assert` | T05 | `java_subset.rs` |
| `control.synchronized` | T05 | `java_subset.rs` |
| `literal.hex` | T05 | `java_subset.rs` |
| `literal.octal` | T05 | `java_subset.rs` |
| `literal.binary` | T05 | `java_subset.rs` |
| `literal.separator` | T05 | `java_subset.rs` |
| `literal.unary_plus` | T05 | `java_subset.rs` |
| `literal.char` | T05 | `java_subset.rs` |
| `literal.minimum_parentheses` | T05 | `java_subset.rs` |
| `literal.positive_overflow` | T05 | `java_subset.rs` |
| `call.recursion` | T05 | `java_subset.rs` |
| `call.library` | T05 | `java_subset.rs` |
| `call.checked_overflow_library` | T05 | `java_subset.rs` |
| `call.floor_library` | T05 | `java_subset.rs` |
| `dispatch.instance` | T05 | `java_subset.rs` |
| `conversion.boxing` | T05 | `java_subset.rs` |
| `conversion.boolean_truthiness` | T05 | `java_subset.rs` |
| `heap.array` | T05 | `java_subset.rs` |
| `heap.null` | T05 | `java_subset.rs` |
| `types.float` | T05 | `java_subset.rs` |
| `types.decimal` | T05 | `java_subset.rs` |
| `types.unbounded` | T05 | `java_subset.rs` |
| `operations.unbounded` | T05 | `java_subset.rs` |
| `operations.unbounded_shift` | T05 | `java_subset.rs` |
| `conversion.range_checked` | T05 | `java_subset.rs` |
| `async.future` | T05 | `java_subset.rs` |
| `types.byte` | T05 | `java_subset.rs` |
| `types.short` | T05 | `java_subset.rs` |
| `types.char` | T05 | `java_subset.rs` |
| `operations.mixed_binary` | T05 | `java_subset.rs` |
| `operations.long_shift_count` | T05 | `java_subset.rs` |
| `conversion.mixed_ternary` | T05 | `java_subset.rs` |
| `declaration.class` | T05 | `java_subset.rs` |
| `declaration.field` | T05 | `java_subset.rs` |
| `declaration.inheritance` | T05 | `java_subset.rs` |
| `declaration.missing_public` | T05 | `java_subset.rs` |
| `declaration.overload` | T05 | `java_subset.rs` |
| `declaration.unrelated_method` | T05 | `java_subset.rs` |
| `declaration.annotation` | T05 | `java_subset.rs` |
| `declaration.import` | T05 | `java_subset.rs` |
| `declaration.generic` | T05 | `java_subset.rs` |
| `declaration.throws` | T05 | `java_subset.rs` |
| `identifier.dollar` | T05 | `java_subset.rs` |
| `identifier.unicode` | T05 | `java_subset.rs` |
| `identifier.contextual` | T05 | `java_subset.rs` |
| `identifier.ignored_control` | T05 | `java_subset.rs` |
| `source.unicode_escape` | T04 | `java_frontend_vectors.rs` |
| `source.bom` | T04 | `java_frontend_vectors.rs` |
| `source.crlf` | T04 | `java_frontend_vectors.rs` |
| `source.missing_lf` | T04 | `java_frontend_vectors.rs` |
| `source.nul` | T04 | `java_frontend_vectors.rs` |
| `source.utf8` | T04 | `java_frontend_vectors.rs` |
| `source.noncharacter` | T04 | `java_frontend_vectors.rs` |
| `capture.symlink` | T04 | `java_frontend_vectors.rs` |
| `capture.hardlink` | T04 | `java_frontend_vectors.rs` |
| `capture.unlisted` | T04 | `java_frontend_vectors.rs` |
| `capture.case_collision` | T04 | `java_frontend_vectors.rs` |
| `contract.missing` | T05 | `java_contracts.rs` |
| `contract.unused` | T05 | `java_contracts.rs` |
| `contract.duplicate` | T05 | `java_contracts.rs` |
| `contract.duplicate_key` | T05 | `java_contracts.rs` |
| `contract.requires_result` | T05 | `java_contracts.rs` |
| `contract.unknown_parameter` | T05 | `java_contracts.rs` |
| `contract.local` | T05 | `java_contracts.rs` |
| `contract.empty_ensures` | T05 | `java_contracts.rs` |
| `contract.division` | T05 | `java_contracts.rs` |
| `contract.shift` | T05 | `java_contracts.rs` |
| `contract.conversion` | T05 | `java_contracts.rs` |
| `contract.unsigned` | T05 | `java_contracts.rs` |
| `contract.negative_zero` | T05 | `java_contracts.rs` |
| `contract.profile` | T05 | `java_contracts.rs` |
| `lowering.div_missing` | T06 | `java_lowering.rs` |
| `lowering.div_extra` | T06 | `java_lowering.rs` |
| `lowering.overflow` | T06 | `java_lowering.rs` |
| `lowering.shift_unmasked` | T06 | `java_lowering.rs` |
| `lowering.shift_wrong_mask` | T06 | `java_lowering.rs` |
| `lowering.shift_unlinked` | T06 | `java_lowering.rs` |
| `lowering.unsigned_escape` | T06 | `java_lowering.rs` |
| `compiler.unknown_tree` | T04 | `java_frontend_vectors.rs` |
| `compiler.error_type` | T04 | `java_frontend_vectors.rs` |
| `compiler.unknown_diagnostic` | T04 | `java_frontend_vectors.rs` |
| `compiler.external_source` | T04 | `java_frontend_vectors.rs` |
| `compiler.external_lookup` | T04 | `java_frontend_vectors.rs` |
| `compiler.unexpected_output` | T04 | `java_frontend_vectors.rs` |
| `method.parameter_slots` | T05 | `java_subset.rs` |

## 5. Counter and executable test ownership

All exact-boundary tests assert counter behavior; a different earlier limit
may reject a materialized input, and the OS can exhaust resources before a
logical boundary. The owner must distinguish these outcomes rather than claim
an input reached the intended counter when another gate stopped it.

| Counter | Primary task | Owning test |
| --- | --- | --- |
| `source_files` = 256 | T04 | `java_frontend_vectors.rs` |
| `source_file_bytes` = 1,048,576 | T04 | `java_frontend_vectors.rs` |
| `source_total_bytes` = 16,777,216 | T04 | `java_frontend_vectors.rs` |
| `contract_files` = 128 | T04 | `java_frontend_vectors.rs` |
| `contract_file_bytes` = 1,048,576 | T04 | `java_frontend_vectors.rs` |
| `contract_total_bytes` = 8,388,608 | T04 | `java_frontend_vectors.rs` |
| `snapshot_entries` = 512 | T04 | `java_frontend_vectors.rs` |
| `snapshot_total_bytes` = 33,554,432 | T04 | `java_frontend_vectors.rs` |
| `normalized_path_bytes` = 1,024 | T04 | `java_frontend_vectors.rs` |
| `canonical_method_id_bytes` = 1,024 | T04 | `java_frontend_vectors.rs` |
| `selected_methods` = 32 | T04 | `java_frontend_vectors.rs` |
| `method_closure` = 128 | T05 | `java_subset.rs` |
| `syntax_nodes` = 250,000 | T04 | `java_frontend_vectors.rs` |
| `syntax_depth` = 256 | T04 | `java_frontend_vectors.rs` |
| `instructions_per_method` = 100,000 | T06 | `java_lowering.rs` |
| `instructions_per_closure` = 250,000 | T06 | `java_lowering.rs` |
| `cfg_blocks_per_method` = 1,024 | T06 | `java_lowering.rs` |
| `cfg_blocks_per_closure` = 8,192 | T06 | `java_lowering.rs` |
| `contract_clauses` = 64 | T05 | `java_contracts.rs` |
| `contract_nodes_per_method` = 1,024 | T05 | `java_contracts.rs` |
| `contract_nodes_per_closure` = 8,192 | T05 | `java_contracts.rs` |
| `contract_depth` = 32 | T05 | `java_contracts.rs` |
| `normalized_issues` = 1,024 | T04 | `java_frontend_vectors.rs` |
| `diagnostic_message_bytes` = 4,096 | T04 | `java_frontend_vectors.rs` |
| `diagnostic_total_message_bytes` = 2,097,152 | T04 | `java_frontend_vectors.rs` |
| `frontend_argument_bytes` = 131,072 | T04 | `java_frontend_vectors.rs` |
| `frontend_stdout` = 268,435,456 | T06 | `java_lowering.rs` |
| `frontend_stderr` = 2,097,152 | T06 | `java_lowering.rs` |
| `vir_canonical_bytes` = 201,326,592 | T06 | `java_lowering.rs` |
| `source_map_canonical_bytes` = 33,554,432 | T06 | `java_lowering.rs` |
| `source_manifest_canonical_bytes` = 4,194,304 | T06 | `java_lowering.rs` |
| `parameter_slots` = 255 | T05 | `java_subset.rs` |

The concrete files below each have exactly one primary creation/maintenance
task. Later aggregate gates invoke them without transferring ownership.

| Test file | Primary task | Scope |
| --- | --- | --- |
| `crates/mpk-vc/tests/java_profile_spec.rs` | T01 | strict frozen data/hash/spec model, no production Java acceptance |
| `crates/mpk-cli/tests/java_build_inputs.rs` | T02 | exact offline build/archive/JAR closure |
| `crates/mpk-vc/tests/java_profile_vectors.rs` | T03 | compiled registry and independent artifact/check validators |
| `crates/mpk-cli/tests/java_frontend_vectors.rs` | T04 | capture/compiler/transport/diagnostic executor |
| `crates/mpk-cli/tests/java_subset.rs` | T05 | declarations/names/types/closure/initialization |
| `crates/mpk-cli/tests/java_contracts.rs` | T05 | sidecar parser/normalization/bounds |
| `crates/mpk-cli/tests/java_lowering.rs` | T06 | operations/CFG/checks/emission bounds |
| `crates/mpk-cli/tests/java_source_maps.rs` | T06 | byte origins and manifest binding |
| `crates/mpk-cli/tests/java_frontend_runner.rs` | T07 | candidate registered JVM and Linux enforcement |
| `crates/mpk-cli/tests/java_policy_verify.rs` | T08 | call WP, certificate, evidence, AI/API context/redaction |
| `crates/mpk-cli/tests/java_release_gate.rs` | T09 | differential/fuzz/two-build/two-run/rehearsal/upgrade |
| `crates/mpk-cli/tests/java_activation.rs` | T10 | atomic installed five-tuple cutover/rollback |

## 6. Probe evidence, validation, and completion boundaries

T01 uses disposable public-API and JVM compatibility probes, not a Java
production frontend. Public compiler observations establish signed literal
folds, unchanged accepted tree structure, raw versus implicit method modifiers,
source positions, exact diagnostic codes, bounded listener abort, explicit
source/class/processor refusal, and the platform-file-manager limitation.

The pinned compiler accepts a 256-descriptor-unit method during `analyze()`;
T05 therefore owns an explicit 255-unit check. `--release 25` uses an internal
platform reference manager outside the forwarding application's callbacks;
T02's complete pinned JDK inventory and T07's filesystem/runtime closure
constrain that separate view. Wrapper callback coverage alone is insufficient.

The local Linux amd64 compatibility measurements run under CPU emulation.
They measure actual pinned JVM/compiler behavior and the recorded minimal
filesystem/privilege/network setup. They do not establish every production
cgroup limit, syscall policy, clone restriction, descendant-cleanup condition,
or native Linux resource-failure path. T07 owns their installed enforcement;
T09/T10 must execute the complete local native Linux release gates. No weaker
JIT/sandbox fallback is permitted when the frozen baseline fails.

T01 verification record (2026-08-31):

- `cargo test -p mpk-vc --test java_profile_spec`: 10 tests passed;
  `cargo test -p mpk-vc --test canonical_json`: 8 tests passed.
- `./scripts/check-fast.sh` passed the complete local fast gate.
- The minimal-root probe passed 20 public-API cases and 15 JVM compatibility
  checks. Repeated runs produced identical JSON bytes.
- Five extracted-JDK tamper cases rejected; a sparse archive declaring 1 TiB
  was rejected before archive-body processing.
- The complete native Linux release gate was not run on the ARM development
  host. The recorded x86-64 runs use emulation; native installed enforcement
  and release acceptance remain T07/T09/T10 obligations.

Normal verification uses `./scripts/check-fast.sh`. Before Java activation,
the current C# composed local Linux release gate remains active. T10 changes
`check-all.sh` and documentation together to the composed Java gate while
preserving every predecessor-language test. GitHub Actions/workflows are not
created, run, monitored or required.

T02 implementation record (2026-08-31):

- `java-tools/java2vir` contains the inactive main class, runtime identity
  smoke check, exact manifest and project notice. It performs no selected-
  source parsing, artifact emission or release registration.
- `scripts/build-java-frontend.sh` and `java_build_inputs.py` import only the
  hash-pinned archive, materialize its exact inventory, snapshot every project
  input and execute two separate offline builds. No host JDK or package
  resolver is used. Generated class/JAR bytes and canonical metadata match.
- `release/build-inputs/java/build-inputs.json` owns the closed recipe and
  source inventory; `candidate-inventory.json` owns class/JAR/notice hashes.
  `java_build_inputs.rs` independently binds those records and runs the
  hostile-input test owner. The provisioned integration test builds and
  exports under hostile Java/Docker/proxy environment settings.
- The candidate remains unregistered and rejects frontend arguments without
  stdout. The active release and all frozen semantic vectors remain unchanged.
  Native installed-runner enforcement and release acceptance remain T07/T09/T10.
- Verification passed: 13 hostile-input self-tests, three ordinary
  `java_build_inputs.rs` tests, the explicitly enabled two-build/export
  integration test, `--check-build-inputs`, `--check`, and `check-fast.sh`.
  The integration test's normal ignored status does not represent an unrun
  gate: it was run explicitly with the provisioned archive and pinned image.
  The final task review has no findings. These builds ran under Linux x86-64
  emulation on the ARM development host; no native installed Java release
  acceptance is claimed.

For commands and artifact formats, see `java-tools/README.md`. The local
two-build gate is separate from the unimplemented native Java release gate.

T03 implementation record (2026-08-31):

- `semantic_profile_registry.rs` compiles the exact Java entry, revision-3
  root, parameter envelope and nine payloads; `java_profile.rs` validates
  finite selection/path/name/signature rules. Explicit candidate validation
  proves every revision-2 entry is byte-identical. Compiled support alone does
  not install a registry or register a frontend tuple.
- `java_source_artifacts.rs`, `successor_source_artifacts.rs`, `safety_check.rs`
  and the private structural projection in `successor_vc.rs` admit only the
  Java scalar operations, exact check lists, linked and ordered shift helpers,
  closed unsigned intermediates, typed normalized contracts, dense identifiers,
  acyclic CFG and source calls. Public artifact and VC contexts remain Java;
  no new source axiom or checker rule is added.
- Source maps require original UTF-8 ranges, method-owned paths and identical
  shift-helper origins, with no synthetic permission. Captured input hashes
  remain private validation state and must match the manifest's exact selected
  input set; rehashing different bytes cannot reuse an old validated map.
  Manifest selection must account for every closure method and source file.
- `java_profile_vectors.rs` executes the frozen registry/identity/payload/hash
  cases and additional repaired-hash, type, conversion, check, shift, flow,
  call, map, manifest and VC mutations. Raw sidecar parsing/attachment remains
  T05; compiler/capture and emitted-source conformance remain T04-T06. Test
  artifacts are hand-authored and do not claim Java source execution.
- The shift-order review finding was reproduced by a failing regression, then
  fixed by validating the exact helper sequence. The repeated full-diff review
  has no remaining findings.
- Verification passed: `cargo test -p mpk-vc --test java_profile_vectors
  --test java_profile_spec --test semantic_profile_registry_runtime
  --test successor_source_artifacts --test successor_vc --offline`
  (12 + 10 + 4 + 4 + 5 tests), `cargo test -p mpk-cli --test java_build_inputs
  --offline` (four ordinary tests), and `cargo test -p mpk-api
  compiled_java_candidate_cannot_start_an_installed_api_session --offline`
  (one test). `cargo check --workspace --all-targets --offline` and
  `cargo clippy --workspace --all-targets --offline -- -D warnings` passed.
  All 23 frozen vector raw-byte digests still match their manifest records.
- `CARGO_NET_OFFLINE=true ./scripts/check-fast.sh` passed the complete local
  gate: formatting, obsolete-interface checks, Clippy, workspace tests, CLI
  build and certificate acceptance/rejection smoke checks. The gate, including
  its Git file enumeration, ran outside the sandbox.
- The ignored Java two-build/export test was not rerun in T03: the T02 project,
  build recipe, fixed JDK and measured candidate inventories are unchanged.
  The complete native Linux release gate was not run on this ARM development
  host; installed JVM enforcement and Java release acceptance remain T07/T09/T10.

T04 implementation record (2026-08-31):

- `CapturedSnapshot`, `Selection` and `SourceText` enforce immutable selected
  file/contract bytes, no-follow file types and inventory, strict UTF-8 with no
  Unicode escape preprocessing, and original UTF-8 positions. The shared
  native capture recognizes Java inputs and preserves unlisted entries for
  rejection. The Java second pass requires the native parent's private
  read-only snapshot; its path metadata checks do not replace host capture.
- `CompilerSession`, `ClosedFileManager` and `TreeInventory` use fresh public
  parse/analyze APIs, exact options and locale, immutable raw syntax facts,
  all syntactic branches and bounded pre/post traversals. Captured immutable
  character-sequence identity binds compiler unit wrappers without internal
  APIs; diagnostic provenance still requires the original source object.
  Post-attribution comparison applies only after T05's raw admission gates.
- `CompilerDiagnostics`, the closed Java codebook and failure serializer
  count before retention, never request compiler prose, validate byte spans,
  sort deterministically and publish no partial artifact. Rust independently
  validates the complete Java diagnostic code/status/phase/message/exit set.
- `java_frontend_vectors.rs` owns the separately compiled private Java harness.
  It executes all 20 compiler observations, the 17 T04 rejection cases, all 15
  file-manager boundary checks, capture/source/diagnostic/counter regressions,
  planted-dependency positive controls and original byte-coordinate checks.
  The 32 frozen numeric definitions have inclusive/plus-one tests; T05/T06
  still own their subset/contract/lowering counter consumers. Test-only code,
  planted classes and processor services are excluded from the candidate JAR.
- The `precedence_cases` harness executes six current capture/encoding/parse/
  attribution and operational-failure cases. Its explicit
  `follow_on_precedence` inventory preserves six downstream obligations:
  release preflight (T07), subset before sidecars and raw excluded class/var
  gates (T05), sidecars before lowering (T05/T06), and map failure before
  publication (T06). Their closed failure transport and raw-tree deferral are
  tested in T04; final semantic outcomes require those stages and all must
  execute before T09. These are downstream contributions to the T04 harness,
  not a claim that T05-T07 have been implemented.
- T04 found a T01 fixture transcription defect: `utf16-tab-bmp-nonbmp` had
  `??` in five source/spelling strings although its recorded coordinates
  described the original `é😀` comment. `CompilerProbe.java` retains that
  exact input. Only those five strings were restored; all measured offsets,
  tree/type/element facts, compiler settings, semantic payloads and immutable
  IDs remain unchanged. The Java vector raw SHA-256 changed from
  `598d066c2e707e1302f16560f9e4efe69558f5153ae29a870d85ef99e7dea26d` to
  `6d5b467efd44cdf044f21e34adf53a87c9c06ede78f664107b5e246794e1aea0`.
  The manifest records the correction and T04 owner. A no-JDK fixture test
  now recomputes every recorded spelling and UTF-16/UTF-8 range, and the
  pinned compiler reproduces the complete repaired observation.
- Two isolated offline builds produced the same 28 classes and 108,747-byte
  JAR, SHA-256
  `a08106c84aa784f37b0f88484c8d30dee2eaf060d62c591575582b420677d8ea`.
  Build/source/class inventories and independent build-test goldens were
  updated together. The T02 main/provider-check class bytes, installed
  registries and four Go/Rust/C# tuples are unchanged.
- The review also found that an invalid path could compete with the entry
  budget in filesystem iteration order. Capture now completes bounded name
  collection and sorting before path validation. Opposite creation-order
  regressions produce the same limit outcome. The repeated task review has
  no remaining findings.
- Verification passed: the fixed-JDK harness (20 observations, 17 owned
  rejections, 15 file-manager boundaries, 32 limit definitions and 301 Java
  assertions), both explicitly enabled build/export and frontend-vector
  integration tests, the ordinary Java build/protocol/capture tests, the
  10 specification and 12 artifact/profile tests, and the complete
  `CARGO_NET_OFFLINE=true ./scripts/check-fast.sh` local gate. The latter
  includes formatting, obsolete-interface checks, Clippy, workspace tests,
  CLI build and certificate smoke checks and runs outside the sandbox for
  Git enumeration. The final code was rebuilt and rechecked after review fixes.
- The native Linux installed release gate was not run on the ARM development
  host. These JDK runs use Linux amd64 emulation. T07/T09/T10 still own native
  JVM enforcement, complete downstream conformance and release acceptance;
  the packaged main remains version-only and Java is not activated.

T05 implementation record (2026-08-31):

- `JavaSubset`, `SourceTokens` and `ScalarType` enforce the exact raw interface,
  method, parameter/local, statement, literal, operation and conversion rules.
  Initialization is inert by source shape, signatures count explicit JVM
  descriptor units, and admitted variable reads retain exact source spelling
  and symbol bindings. All syntactic branches are checked, including dead
  branches; every path returns and every selected declaration is accounted for.
- Calls resolve to the captured `ExecutableElement` identities. Selection roots
  close over all syntactic calls, reject cycles or unused methods/files and
  produce deterministic callee-first order with canonical-ID tie breaking.
  The internal immutable closure preserves raw origins, types, variable
  bindings and call targets for T06, without implementing CFGs or lowering.
- `StrictJson` performs an iterative validation pass over every selected sidecar
  before retaining a contract expression model. It rejects duplicate keys at
  every level, invalid UTF-8/Unicode/numbers, null and malformed JSON. Parsing
  then enforces closed shapes and inclusive clause/node/depth bounds. Complete
  attachment precedes type interpretation; type failures follow selected-file
  order, while successful attached contracts follow callee-first method order.
- `JavaContracts` resolves only method parameters and the ensures-only result,
  enforces signed scalar types and the closed operator set, preserves clause/
  operand order and computes the canonical sidecar and common normalized hashes.
  The frozen 430-byte sidecar and 975-byte normalized hash payload reproduce
  their exact T01 hashes. Raw input bytes retain a distinct hash; a closure from
  different source bytes or a different selection cannot receive the sidecars.
- `JavaAdmission` sequences compiler analysis, source admission and contracts,
  closes the compiler session and normalizes operational failures without
  returning partial results. It has no public launcher. The packaged main is
  still version-only; no successful source artifact, proof verdict or installed
  Java tuple was exposed at T05 completion; T06 was then the next task.
- `java_subset.rs` and `java_contracts.rs` own the private fixed-JDK executor:
  61 source refusals, 14 contract refusals, source admission for 49 accepted
  vectors, all 34 matrix rows, all 35 conversion rules, and real exact/plus-one
  consumers for six subset/contract limits. Python independently checks source
  bindings, call order, raw and canonical hashes and normalization; Rust checks
  the actual failure envelopes, Java context and normalized contract hashes.
  The vector manifest adds those owners; frozen vector bytes and identities
  are unchanged. Missing-sidecar fixtures retain a reachable helper's sidecar
  so the required nonempty selection is valid and capture completes first.
  The 256-slot boundary case calls the oversized declaration from a valid
  zero-parameter root, since the parent already rejects oversized selected
  signatures. All 301 generated selections pass the independent Rust validator.
- Review found javac sharing raw modifier/type objects in multi-declarators.
  T04's inventory now counts those objects once and marks sharing for deferred
  accepted-tree comparison, so T05's raw multi-declarator gate yields the frozen
  subset refusal. Sharing in an admitted subtree still fails closed. Other
  regressions cover raw identifier/symbol agreement, JSON validation across a
  whole batch, attachment-before-type precedence, operator tags distinct from
  atom tags and selected-file error order independent of emission order.
- T05 contributes three executable precedence outcomes to T04's owning harness:
  subset before missing sidecars, excluded class before synthesized-constructor
  comparison, and excluded `var` before inferred-type comparison. At T05
  completion, three remained pending: contracts before lowering and map
  failures (T06), and release preflight before source (T07). None was claimed
  complete by a transport-only check.
- Two isolated offline builds reproduced all 21 project-file records, 53
  classes and the same 202,319-byte JAR, SHA-256
  `9ec1e1c639dca558365820108359dc97d1b7613b11c9f19025430dd68379f82c`.
  Source/class/JAR inventories and independent build-test goldens agree.
  The version-only `Main` and `BuildIdentity` class bytes are unchanged.
- Verification passed: the fixed-JDK admission harness (301 cases and 833
  Java assertions), explicitly enabled offline build/export, T04 compiler/
  diagnostic, T05 source-subset and contract integration tests, and the full
  `CARGO_NET_OFFLINE=true ./scripts/check-fast.sh` gate. The final gate includes
  formatting, obsolete-interface checks, Clippy, workspace tests, CLI build
  and certificate smoke checks. The boundary fixture fix was rerun through
  both T05 owning integration tests, the T04 regression and the full fast gate.
  The repeated final task review has no remaining findings.
- Native installed Linux release enforcement was not run on the ARM development
  host. Fixed-JDK tests use Linux amd64 emulation; T07/T09/T10 still own native
  JVM isolation, full downstream conformance and release acceptance. The active
  registry remains revision 2 with the existing four Go/Rust/C# tuples.

T06 implementation record (2026-08-31):

- `JavaIr`, `JavaLowering` and `JavaLoweringValidation` construct and validate
  the complete acyclic CFG from T05's admitted source model. Functions retain
  callee-first order; blocks receive false-before-true BFS IDs only after graph
  construction, followed by dense block-parameter and instruction IDs. Source
  locals keep declaration order, and a separate topological pass verifies
  definite assignment. Values live across expression branches travel through
  explicit block parameters and edge arguments without generated source locals.
- Operations preserve left-to-right evaluation, branch-local checks and source
  syntax without constant folding. Wrapping arithmetic, MIN/-1 division and
  remainder, exact `divisor_nonzero` lists, widening/truncation and adjacent
  masked shift patterns follow the Java profile. Unsigned shift carriers have
  only their required uses; calls bind the earlier callee's exact contract hash.
- `JavaSourceMaps` emits one original UTF-8 byte origin per function,
  instruction and terminator, in canonical reference order. Original source
  and tree identity, UTF-16 boundaries, method containment and source-node
  roles are checked; block parameters receive no synthetic map entries.
  `JavaEmission` rechecks raw source/sidecar and selection bindings, then
  assembles complete VIR, map, frontend-stage manifest and success response.
  `CanonicalJson` counts canonical bytes before allocation and before each
  write, includes the final stdout LF, and uses the common domain-NUL hashes.
  `JavaFrontend` returns either the complete response or an artifact-free
  normalized failure. No partial success bytes are published.
- The private fixed-JDK harness has 99 source fixtures: all 49 accepted
  vectors, 27 operation mappings, six symbolic CFG goldens, 13 additional
  evaluation/origin regressions, two raw-input linkage cases, one actual
  method-block overflow and one contracts-before-lowering case. Fresh compiler
  sessions must produce identical complete response bytes. Python separately
  checks canonical artifacts and 125 mathematical Bool/BV evaluations; the
  Rust owning tests import real responses and captured bytes through T03's
  validators and reject rehashed semantic, map and manifest mutations.
- All seven frozen lowering refusals are included in 25 producer-model
  mutations; all seven source-map vectors execute. All nine T06 counters
  have inclusive/plus-one checks through production consumers, including
  verifying that excess additions do not change either method or closure
  totals. These counter checks do not allocate maximal artifacts or claim
  native process-resource enforcement. The private producer has no normal
  stderr output; T07 still owns the native parent stream limits.
- T06 supplies the two executable precedence contributions recorded by T04:
  invalid contracts prevent lowering, and invalid maps prevent publication.
  Only release preflight before source remains pending for T07. Review fixes
  distinguish the release registry's schema from its ID, allow T07 to inject
  the complete distribution digest independently of the JDK archive digest,
  enforce source-origin ownership and roles, reject stray call metadata, and
  avoid repeated scans when resolving local declarations.
- Two isolated offline builds reproduced 28 project-file records, 82 classes
  and the same 313,051-byte JAR, SHA-256
  `125ef66b3de047ca5ff8c659c1d38e8c225f1cf2975db5fb4d4b4e9c8d67c2ff`.
  Source/class/JAR inventories and independent build-test goldens agree.
  All 53 predecessor class records, including `Main` and `BuildIdentity`,
  are unchanged. The harness explicitly
  supplies test bundle identities, a zero registry digest, the measured JAR
  hash and a test distribution digest; these do not register a release tuple.
- Verification passed: all eight explicitly enabled offline integration tests for
  `java_build_inputs`, `java_frontend_vectors`, `java_subset`, `java_contracts`,
  `java_lowering` and `java_source_maps`, plus
  `CARGO_NET_OFFLINE=true ./scripts/check-fast.sh`. The full fast gate includes
  formatting, obsolete-interface checks, Clippy, workspace tests, CLI build
  and certificate smoke checks. The strict obsolete-interface check was also
  rerun with every new file staged. The repeated full task review has no
  remaining findings.
- Native installed Linux release enforcement was not run on the ARM
  development host. Fixed-JDK tests use Linux amd64 emulation; T07 owns the
  registered native runner, T09 the JDK differential/fuzz corpus and release
  rehearsal, and T10 activation. Frozen Java/revision-3 vector bytes, the
  installed revision-2 registry and all four Go/Rust/C# tuples are unchanged.

T07 implementation record (2026-08-31; native acceptance pending):

- `java_release_bundles.py` and `build-java-candidate.sh` reconstruct a private
  Java candidate and registry from the frozen inputs and measured frontend.
  They live under `release/build-inputs/java/`, outside the active registry.
  The private registry digest is
  `3208209c96bedce5ce94b26a824a9164c281f5bf8bc76e03f56497a85d564269`;
  the complete toolchain distribution digest is
  `8f6c540278984d0a8f94f3d288ab94fa84fe165a023c2a376b26bfa955d0e8e1`.
  The frontend contains its JAR and project notice. The toolchain has 405
  regular files: 399 JDK files and six frozen native files. All 205 archive
  legal links become independent regular copies of their frozen target bytes.
  No host JDK, native library, dependency restore or image pull is used.
- Assembly requires a new absolute destination and an explicit Linux x86-64
  owning test executable. Two isolated offline builds precede copying the
  exact JDK/native closure. The final check verifies every file, directory,
  byte hash, executable bit, sealed mode and unique inode. Links, missing or
  extra content, aliases and writable inputs reject. A failed assembly is not
  a publishable image; this private assembler does not activate a release.
- `java_release.rs` constructs only the frozen 27-token JVM prefix, six-entry
  environment and ordered frontend grammar, including actual candidate
  identities. The shared descriptor validator requires the exact Java
  candidate, host and layout even after outer hashes are repaired. Java's
  `mpk.host.linux-x86_64-gnu.java25.v0` and corresponding runtime layout are
  distinct from the existing Go/Rust/C# contracts; those budgets and IDs are
  unchanged. No environment flag or caller-selected executable opens Java.
- `java_frontend_runner.rs` is included only by its owning test executable.
  It locates registries beside the actual installed `bin/mpk` inode, uses the
  shared no-follow loader to snapshot registered bundles, and prepares the
  Java host before accessing selected source. It checks captured path/kind/
  count/size agreement before materialization, imports the complete response
  through the shared validators, and binds successful manifests to the exact
  frontend, distribution, component and release-registry identities.
- Java's sandbox branch uses atomic cgroup placement and the existing pidfd,
  bounded-stream and descendant/backing cleanup machinery with 1 GiB memory,
  zero swap, 128 PIDs, 16 GiB address space, 1,024 descriptors, zero core bytes,
  64 MiB `noswap`/noexec tmpfs, a 120-second timeout, 256 MiB stdout and 2 MiB
  stderr limits. It maps only UID/GID 65534, clears supplementary groups and
  all capability sets, mounts a private read-only PID proc view, and installs
  a finite x86-64 syscall policy. Only exact pthread clone flags pass;
  clone3 receives ENOSYS for the glibc fallback. Socket and namespace
  challenges must fail in the installed policy before source is exposed.
  This describes the implemented boundary, not measured native acceptance.
- `FrontendArguments`, `JavaRelease` and `RuntimePreflight` connect packaged
  `Main` to the existing admission/lowering/emission pipeline. Before capture,
  the child checks exact argv/environment, compiler/JAR/JDK bytes, code source,
  read-only roots, PID/UID/GID/capability state and the fixed JVM settings.
  Malformed invocation has no stdout and exits 2. Valid-request operational
  failures use artifact-free successor envelopes. The private lowering
  harness retains its explicit test identities; real component projections
  now describe the Java executable, complete JDK content and native content.
- The owning test checks real descriptor validation, repaired-hash mutations,
  ordered multi-file launcher expansion and the closed isolation inventory.
  `RunnerTests.java` adds 118 fixed-JDK assertions covering truncated/reordered
  arguments, forbidden overrides, byte/link/size changes and packaged-Main
  metadata failure before absent source capture. T04's precedence ledger now
  names that executable contribution; native installed precedence remains
  required separately. The image gate rejects ten real JAR/JDK/native/mode/
  link/registry mutations without claiming kernel enforcement.
- `check-java-runner.sh --native` requires a native x86-64 Linux host, writable
  initial cgroup-v2 hierarchy, root and strace. It runs installed Java cases,
  demands identical hostile-environment output, rejects an undelegated launch,
  and exercises OOM/PID/timeout/stdout/stderr/tmpfs faults with fixed source-free
  payloads through the same resource supervisor. It also runs the ten image
  mutations through installed preflight. Its receipt binds the runner and
  registry and records JVM-attributed clone flags, clone3 fallback, pre-exec
  socket denials and trace hash. A transport-only parser regression rejects
  parent-only or incomplete traces; it is not substituted for native evidence.
- Review fixed the candidate inventory schema, installed success status and
  component linkage, constrained the shared .NET address-space exception to
  its actual bootstrap slot, and restored an exhaustive Linux-only C# test
  match without changing its Go/Rust cases. It also corrected stale source/
  notice/status descriptions and prevented another process's trace events
  from satisfying JVM evidence requirements. Final source records cover 31
  project files and 87 classes; two isolated builds reproduce the 332,137-byte
  JAR, SHA-256
  `333a050128cddc206474c9bdcca244276c08b246f2a5ba11f55983537cf7cd75`.
- Verification passed: the eight explicitly enabled offline Java integration
  tests from T02/T04/T05/T06; `check-java-frontend.sh --run-runner`; private
  candidate assembly and `check-java-runner.sh --image`; the owning Rust test;
  and `CARGO_NET_OFFLINE=true ./scripts/check-fast.sh`. The full gate includes
  formatting, obsolete-interface checks, workspace Clippy/tests, CLI build
  and certificate smoke checks. Fixed offline Linux amd64 builds also passed
  all-target Clippy, owning-runner compilation and the sandbox unit target,
  including the seccomp logic test. Those builds/tests ran under ARM-host CPU
  emulation and do not establish native kernel isolation.
  The final standard gate was repeated with all new files staged. The repeated
  task diff review has no remaining findings; native acceptance remains
  separate as recorded below.
- The actual native command was attempted with the assembled image and
  refused this ARM macOS host with exit 69, `JAVA_NATIVE_HOST_REQUIRED`.
  No native host was available: JVM/syscall compatibility, installed privilege/
  filesystem/network/resource failures and cleanup therefore remain unrun.
  **T07 native acceptance is pending; run and review that gate before accepting
  T07 or starting T08.** T09 still owns complete release rehearsal and T10
  public activation. Frozen Java/revision-3 vector bytes, active revision-2
  descriptors and the existing four Go/Rust/C# tuples remain unchanged.

A task is complete only after its exact deliverables, required verification
(or explicitly recorded unrun reason), fix/review loop, commit and push are
finished. T02 does not claim T03 validators or T10 activation. An unknown
compiler/host/schema behavior must be resolved through reviewed freeze changes,
not guessed by an implementer or admitted under an immutable existing ID.
