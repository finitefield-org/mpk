# JAVA-03 Implementation Traceability Ledger

Status: `JAVA-03-T01` and `JAVA-03-T02` complete (2026-08-31);
`JAVA-03-T03` is next and T03 through T10 are pending. The freeze and offline
build candidate do not establish Java source processing, native Linux release
isolation, or Java activation. The active release remains Go/Rust/C# at registry revision 2.

This is an execution plan subordinate to `../specs/JAVA_PROFILE_V0.md`, the
exact Java/revision-3 vectors, and `SEMANTIC_PROFILE_REGISTRY_V1.md`. It does
not change immutable contracts or checker acceptance. Each requirement and
vector family below has one primary implementation owner; downstream users
consume that owner's result without redefining it.

## 1. Serial task graph and bounded deliverables

`CSHARP-02-T20 -> JAVA-03-T01 -> T02 -> T03 -> T04 -> T05 -> T06 -> T07 -> T08
-> T09 -> T10 -> DART-04`. Every task requires its predecessor's completed,
reviewed result. T01 froze the profile; T02 added the inactive offline build.
T03 begins the inactive profile and artifact validators.

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

A task is complete only after its exact deliverables, required verification
(or explicitly recorded unrun reason), fix/review loop, commit and push are
finished. T02 does not claim T03 validators or T10 activation. An unknown
compiler/host/schema behavior must be resolved through reviewed freeze changes,
not guessed by an implementer or admitted under an immutable existing ID.
