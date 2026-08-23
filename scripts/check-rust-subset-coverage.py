#!/usr/bin/env python3
"""Audit the Rust v0 negative/adversarial rule and vector coverage manifests."""

from __future__ import annotations

import json
import re
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
ROOT_MANIFEST = Path("fixtures/rust-basic/manifest.json")
CHILD_MANIFESTS = (
    Path("fixtures/rust-basic/negative/manifest.json"),
    Path("fixtures/rust-basic/adversarial/manifest.json"),
)
DESIGN_PATH = Path("develop/docs/05_rust_frontend_design.md")
VECTOR_PATH = Path("develop/specs/vectors/rust-subset-v0.json")
EXECUTION_VECTOR_PATH = Path("rust-tools/rust2vir/testdata/rust-subset-v0.json")
EXPECTED_VECTOR_GROUPS = {
    "rejected_cases": (73, "case.expect"),
    "same_phase_precedence": (5, "case.primary"),
    "limit_boundaries": (35, "case.at+case.above"),
}

EXPECTED_RULES: dict[int, tuple[str, ...]] = {
    1: ("reference", "raw_pointer", "unsafe", "ffi"),
    2: ("heap_allocation", "vector", "slice", "string"),
    3: ("trait", "generic", "method", "closure", "function_pointer"),
    4: (
        "extern_crate",
        "use",
        "import_alias",
        "reexport",
        "restricted_visibility",
        "non_same_crate_path",
    ),
    5: ("async", "loop", "recursion", "match", "enum", "tuple"),
    6: ("float", "integer_128", "cast"),
    7: ("out_of_range_typed_integer_literal",),
    8: ("static_state", "explicit_panic", "assert_macro", "drop_type"),
    9: (
        "build_script",
        "external_dependency",
        "proc_macro",
        "macro_expansion",
        "cfg",
        "lint_level_attribute",
    ),
    10: (
        "rust_toolchain_file",
        "non_library_crate_type",
        "unapproved_lint_control_argument",
    ),
    11: (
        "module_missing",
        "module_ambiguous",
        "module_cycle",
        "module_path_attribute",
        "module_root_escape",
        "path_nonportable",
        "path_reserved",
        "path_case_collision",
        "source_inventory_disagreement",
    ),
    12: (
        "mutable_parameter",
        "field_mutation",
        "index_mutation",
        "non_usize_array_index",
        "projected_move",
        "partial_move",
    ),
    13: (
        "malformed_contract",
        "unresolved_contract",
        "callstatic_contract_hash_mismatch",
    ),
    14: (
        "preflight_before_source",
        "source_before_subset",
        "typecheck_before_subset",
        "subset_before_contract",
        "toolchain_before_mir",
        "mir_before_semantics",
        "later_diagnostics_excluded",
    ),
    15: (
        "missing_target",
        "unsupported_pointer_width",
        "stale_lockfile",
        "compiler_commit_mismatch",
        "semantic_profile_missing",
        "semantic_profile_unknown",
        "semantic_profile_language_mismatch",
        "crossed_strategy_profile",
        "crossed_axiom_profile",
    ),
    16: (
        "checker_profile_not_active",
        "axiom_profile_not_active",
        "checker_profile_not_permitted",
        "axiom_profile_not_permitted",
    ),
    17: (
        "unknown_mir_statement",
        "unknown_mir_rvalue",
        "unknown_mir_projection",
        "unknown_mir_terminator",
        "unknown_mir_assertion",
        "changed_checked_operation_pattern",
    ),
    18: (
        "non_regular_source",
        "source_symlink",
        "source_reparse_point",
        "source_root_escape",
    ),
    19: (
        "cargo_manifest_bytes",
        "lockfile_bytes",
        "contract_bytes",
        "contract_count",
        "contract_expression_depth",
        "source_set",
        "aggregate_mir",
        "diagnostic_set",
        "vir_bytes",
        "source_map_bytes",
        "source_manifest_bytes",
        "cargo_child_stdout",
        "cargo_child_stderr",
        "frontend_stdout",
        "frontend_stderr",
        "diagnostic_truncation_status",
    ),
    20: (
        "descriptor_oversized",
        "descriptor_noncanonical",
        "inventory_limit",
        "graph_limit",
        "path_limit",
        "per_file_limit",
        "aggregate_cache_limit",
        "declared_size_overflow",
        "declared_size_mismatch",
        "mount_before_validation",
        "execute_before_validation",
    ),
    21: (
        "vc_member_limit",
        "vc_assumption_limit",
        "vc_node_limit",
        "vc_depth_limit",
        "vc_json_limit",
        "skeleton_json_limit",
        "generated_proof_depth_limit",
        "generated_certificate_limit",
        "policy_json_limit",
        "policy_markdown_limit",
        "no_partial_evidence",
        "no_source_status_reclassification",
    ),
    22: ("missing_required_safety_check", "extra_noncanonical_safety_check"),
    23: (
        "theorem_member_missing",
        "theorem_member_duplicate",
        "theorem_member_wrong_partition",
        "dependency_missing",
        "dependency_reversed",
        "dependency_extra",
        "checked_conjunct_without_declaration",
    ),
    24: (
        "frontend_descriptor_missing",
        "frontend_descriptor_unregistered",
        "main_bytes_mismatch",
        "helper_bytes_mismatch",
        "duplicate_helper_name",
        "unknown_helper_name",
        "raw_executable_path",
        "frontend_bundle_id_omitted",
        "frontend_bundle_id_unknown",
        "frontend_bundle_id_incompatible",
        "toolchain_bundle_id_omitted",
        "toolchain_bundle_id_unknown",
        "toolchain_bundle_id_incompatible",
        "driver_digest_mismatch",
        "toolchain_descriptor_missing",
        "toolchain_descriptor_mismatch",
        "toolchain_component_missing",
        "toolchain_component_mismatch",
        "executable_outside_bundle",
        "subordinate_manifest_mismatch",
    ),
    25: (
        "registry_missing",
        "registry_oversized",
        "registry_noncanonical",
        "registry_unknown_field",
        "registry_duplicate_tuple",
        "registry_hash_mismatch",
        "runner_embedded_registry_identity_mismatch",
        "policy_registry_assertion_omitted",
        "policy_registry_assertion_wrong",
        "source_manifest_registry_mismatch",
        "policy_registry_identity_mismatch",
    ),
    26: (
        "v0_free_form_command",
        "scan_recipe_missing",
        "verify_recipe_missing",
        "scan_recipe_duplicate",
        "verify_recipe_duplicate",
        "working_directory_role_wrong",
        "source_root_not_dot",
        "argv_order_noncanonical",
        "contract_order_noncanonical",
        "caller_selected_output_path",
        "absolute_recipe_argument",
    ),
    27: (
        "driver_request_missing",
        "driver_request_mutable",
        "driver_request_oversized",
        "driver_request_noncanonical",
        "driver_request_duplicate_key",
        "driver_request_identity_mismatch",
        "driver_artifact_missing",
        "driver_artifact_partial",
        "driver_artifact_duplicate",
        "driver_artifact_oversized",
        "driver_artifact_noncanonical",
        "driver_artifact_identity_mismatch",
    ),
    28: (
        "status_exit_disagreement",
        "json_missing",
        "json_truncated",
        "json_noncanonical",
        "extra_stdout",
        "partial_artifact",
        "main_frontend_digest_mismatch",
        "repeated_identity_mismatch",
        "source_selection_incorrect",
        "unit_mapping_incorrect",
        "source_map_reference_invalid",
        "source_map_range_invalid",
        "source_map_hash_invalid",
        "vc_identity_mismatch",
        "vir_identity_mismatch",
        "input_set_mismatch",
        "verification_limit_mismatch",
        "final_manifest_mutation",
        "skeleton_source_vc_hash_mismatch",
        "response_vir_hash_mismatch",
        "response_source_manifest_hash_mismatch",
        "downstream_vc_self_hash_mismatch",
    ),
}

MANIFEST_KEYS = {
    "schema",
    "kind",
    "design",
    "expectation_profiles",
    "tests",
    "rule_groups",
    "vector_groups",
}
DESIGN_KEYS = {"path", "section", "line_start", "line_end"}
EXPECTATION_KEYS = {"status", "phase", "code", "exit", "artifacts"}
TEST_KEYS = {"id", "path", "function"}
RULE_GROUP_KEYS = {"bullet", "rules", "expectation", "tests"}
VECTOR_GROUP_KEYS = {
    "path",
    "group",
    "case_ids",
    "expectation_source",
    "tests",
}
STATUS_EXITS = {
    "rejected": 3,
    "source-error": 4,
    "frontend-error": 1,
    "no-json": 2,
    "gate-error": 1,
    "verification-error": 1,
    "validation-error": 1,
    "policy-error": 1,
}
FORBIDDEN_SUCCESS_STATUSES = {"accepted", "ir-lowered", "ready", "verified"}
CODE_PATTERN = re.compile(r"^[A-Z][A-Z0-9_]*$")
ID_PATTERN = re.compile(r"^[a-z][a-z0-9_.-]*$")
RULE_PART_PATTERN = re.compile(r"^[a-z][a-z0-9_]*$")
PORTABLE_PATH_PATTERN = re.compile(r"^[A-Za-z0-9._/-]+$")


class CoverageError(Exception):
    pass


class StrictJsonError(ValueError):
    pass


def fail(message: str) -> None:
    raise CoverageError(message)


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, child in pairs:
        if key in value:
            raise StrictJsonError(f"duplicate object key {key!r}")
        value[key] = child
    return value


def reject_nonfinite_number(value: str) -> None:
    raise StrictJsonError(f"non-finite number {value!r}")


def load_json(relative: Path) -> dict[str, Any]:
    path = ROOT / relative
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=reject_duplicate_keys,
            parse_constant=reject_nonfinite_number,
        )
    except (OSError, UnicodeError, json.JSONDecodeError, StrictJsonError) as error:
        fail(f"{relative}: cannot read strict JSON: {error}")
    if not isinstance(value, dict):
        fail(f"{relative}: root must be an object")
    return value


def exact_keys(value: dict[str, Any], expected: set[str], owner: str) -> None:
    actual = set(value)
    if actual != expected:
        fail(
            f"{owner}: keys differ; missing={sorted(expected - actual)} "
            f"unknown={sorted(actual - expected)}"
        )


def require_nonempty_strings(value: Any, owner: str) -> list[str]:
    if (
        not isinstance(value, list)
        or not value
        or any(not isinstance(item, str) or not item for item in value)
    ):
        fail(f"{owner}: expected a nonempty string array")
    if len(value) != len(set(value)):
        fail(f"{owner}: values must be unique")
    return value


def validate_root_links() -> None:
    manifest = load_json(ROOT_MANIFEST)
    coverage = manifest.get("negative_coverage")
    if not isinstance(coverage, dict):
        fail(f"{ROOT_MANIFEST}: missing negative_coverage object")
    exact_keys(coverage, {"schema", "manifests"}, f"{ROOT_MANIFEST}.negative_coverage")
    if coverage["schema"] != "mpk.rust.negative_coverage.index.v0":
        fail(f"{ROOT_MANIFEST}: wrong negative coverage index schema")
    expected = [str(path.relative_to("fixtures/rust-basic")) for path in CHILD_MANIFESTS]
    if coverage["manifests"] != expected:
        fail(f"{ROOT_MANIFEST}: coverage manifests must be exactly {expected}")


def validate_design_anchor(manifest: dict[str, Any], relative: Path, index: int) -> None:
    design = manifest.get("design")
    if not isinstance(design, dict):
        fail(f"{relative}.design: expected object")
    exact_keys(design, DESIGN_KEYS, f"{relative}.design")
    if design["path"] != str(DESIGN_PATH) or design["section"] != "19.2":
        fail(f"{relative}.design: must point at design section 19.2")
    expected_lines = (2727, 2759) if index == 0 else (2760, 2801)
    if (design["line_start"], design["line_end"]) != expected_lines:
        fail(f"{relative}.design: stale section line anchors")
    lines = (ROOT / DESIGN_PATH).read_text(encoding="utf-8").splitlines()
    if lines[2724].strip() != "### 19.2 Negative and adversarial corpus":
        fail(f"{DESIGN_PATH}: section 19.2 heading moved; review the inventory")
    if "At minimum, fixtures must deterministically reject:" not in lines[2726]:
        fail(f"{DESIGN_PATH}: section 19.2 normative lead-in changed")


def validate_expectations(
    manifest: dict[str, Any], relative: Path
) -> dict[str, dict[str, Any]]:
    profiles = manifest.get("expectation_profiles")
    if not isinstance(profiles, dict) or not profiles:
        fail(f"{relative}.expectation_profiles: expected nonempty object")
    for name, expectation in profiles.items():
        if not isinstance(name, str) or not RULE_PART_PATTERN.fullmatch(name):
            fail(f"{relative}.expectation_profiles: invalid profile {name!r}")
        if not isinstance(expectation, dict):
            fail(f"{relative}.expectation_profiles.{name}: expected object")
        exact_keys(expectation, EXPECTATION_KEYS, f"{relative}.expectation_profiles.{name}")
        status = expectation["status"]
        phase = expectation["phase"]
        code = expectation["code"]
        exit_value = expectation["exit"]
        if not isinstance(status, str) or status in FORBIDDEN_SUCCESS_STATUSES:
            fail(f"{relative}.{name}: invalid negative status {status!r}")
        if not isinstance(phase, str) or not RULE_PART_PATTERN.fullmatch(phase.replace("-", "_")):
            fail(f"{relative}.{name}: invalid phase {phase!r}")
        if expectation["artifacts"] != []:
            fail(f"{relative}.{name}: every negative outcome must be artifact-free")
        if status == "preserve-owning-status":
            if exit_value != "preserve-owning-exit":
                fail(f"{relative}.{name}: preserving status must preserve exit")
        elif (
            status not in STATUS_EXITS
            or type(exit_value) is not int
            or exit_value != STATUS_EXITS[status]
        ):
            fail(f"{relative}.{name}: status/exit pair is not frozen")
        if status == "no-json" and code is None:
            continue
        if not isinstance(code, str) or not CODE_PATTERN.fullmatch(code):
            fail(f"{relative}.{name}: invalid stable code {code!r}")
    return profiles


def test_function_exists(path: Path, function: str) -> bool:
    try:
        source = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError):
        return False
    pattern = re.compile(
        rf"#\[test\](?:\s*#\[[^\]]+\])*\s*(?:async\s+)?fn\s+{re.escape(function)}\s*\("
    )
    return pattern.search(source) is not None


def collect_tests(
    manifest: dict[str, Any], relative: Path, catalog: dict[str, tuple[str, str]]
) -> None:
    tests = manifest.get("tests")
    if not isinstance(tests, list) or not tests:
        fail(f"{relative}.tests: expected nonempty array")
    local_ids: set[str] = set()
    for index, test in enumerate(tests):
        owner = f"{relative}.tests[{index}]"
        if not isinstance(test, dict):
            fail(f"{owner}: expected object")
        exact_keys(test, TEST_KEYS, owner)
        test_id, path_text, function = test["id"], test["path"], test["function"]
        if not isinstance(test_id, str) or not ID_PATTERN.fullmatch(test_id):
            fail(f"{owner}: invalid test ID {test_id!r}")
        if test_id in local_ids:
            fail(f"{owner}: duplicate test ID {test_id}")
        local_ids.add(test_id)
        if (
            not isinstance(path_text, str)
            or not PORTABLE_PATH_PATTERN.fullmatch(path_text)
            or path_text.startswith("/")
            or ".." in Path(path_text).parts
        ):
            fail(f"{owner}: test path must be portable and root-relative")
        if not isinstance(function, str) or not RULE_PART_PATTERN.fullmatch(function):
            fail(f"{owner}: invalid Rust test function {function!r}")
        definition = (path_text, function)
        if test_id in catalog:
            fail(f"{owner}: duplicate global test ID {test_id}")
        catalog[test_id] = definition


def collect_rules(
    manifest: dict[str, Any],
    relative: Path,
    profiles: dict[str, dict[str, Any]],
    known_tests: set[str],
    allowed_bullets: range,
    rules: dict[str, tuple[str, tuple[str, ...]]],
    referenced_tests: set[str],
) -> None:
    groups = manifest.get("rule_groups")
    if not isinstance(groups, list) or not groups:
        fail(f"{relative}.rule_groups: expected nonempty array")
    used_profiles: set[str] = set()
    for index, group in enumerate(groups):
        owner = f"{relative}.rule_groups[{index}]"
        if not isinstance(group, dict):
            fail(f"{owner}: expected object")
        exact_keys(group, RULE_GROUP_KEYS, owner)
        bullet = group["bullet"]
        if type(bullet) is not int or bullet not in allowed_bullets:
            fail(f"{owner}: bullet {bullet!r} is outside this child manifest")
        parts = require_nonempty_strings(group["rules"], f"{owner}.rules")
        for part in parts:
            if not RULE_PART_PATTERN.fullmatch(part):
                fail(f"{owner}: invalid atomic rule name {part!r}")
        profile = group["expectation"]
        if profile not in profiles:
            fail(f"{owner}: unknown expectation profile {profile!r}")
        used_profiles.add(profile)
        test_ids = tuple(require_nonempty_strings(group["tests"], f"{owner}.tests"))
        unknown_tests = set(test_ids) - known_tests
        if unknown_tests:
            fail(f"{owner}: unknown tests {sorted(unknown_tests)}")
        referenced_tests.update(test_ids)
        for part in parts:
            rule_id = f"rust.19.2.b{bullet:02d}.{part}"
            if rule_id in rules:
                fail(f"{owner}: duplicate rule {rule_id}")
            rules[rule_id] = (profile, test_ids)
    unused_profiles = set(profiles) - used_profiles
    if unused_profiles:
        fail(f"{relative}: unreferenced expectation profiles {sorted(unused_profiles)}")


def validate_test_catalog(catalog: dict[str, tuple[str, str]]) -> None:
    for test_id, (path_text, function) in sorted(catalog.items()):
        path = ROOT / path_text
        if not path.is_file():
            fail(f"test {test_id}: missing file {path_text}")
        if not test_function_exists(path, function):
            fail(f"test {test_id}: missing #[test] fn {function} in {path_text}")


def validate_rule_inventory(
    rules: dict[str, tuple[str, tuple[str, ...]]],
    profiles_by_rule: dict[str, dict[str, Any]],
) -> None:
    expected = {
        f"rust.19.2.b{bullet:02d}.{part}"
        for bullet, parts in EXPECTED_RULES.items()
        for part in parts
    }
    actual = set(rules)
    if actual != expected:
        fail(
            "section 19.2 atomic coverage differs; "
            f"missing={sorted(expected - actual)} unknown={sorted(actual - expected)}"
        )
    code_owners: dict[str, set[str]] = defaultdict(set)
    for rule_id, (profile, test_ids) in rules.items():
        expectation = profiles_by_rule[rule_id]
        if not test_ids:
            fail(f"{rule_id}: rule has no real test owner")
        code = expectation["code"]
        if code is not None:
            code_owners[code].add(rule_id)
    if not code_owners:
        fail("coverage has no normative stable-code mapping")


def validate_authoritative_codes(
    manifests: list[tuple[Path, dict[str, Any], dict[str, dict[str, Any]]]],
) -> None:
    authoritative_text = "\n".join(
        path.read_text(encoding="utf-8")
        for path in sorted((ROOT / "develop/specs").rglob("*"))
        if path.is_file() and path.suffix in {".json", ".md"}
    )
    registered = set(re.findall(r"\b[A-Z][A-Z0-9_]{2,}\b", authoritative_text))
    for relative, _, profiles in manifests:
        for name, expectation in profiles.items():
            code = expectation["code"]
            if code is not None and code not in registered:
                fail(
                    f"{relative}.expectation_profiles.{name}: "
                    f"code {code} is absent from the normative specifications"
                )


def collect_vector_groups(
    manifest: dict[str, Any],
    relative: Path,
    known_tests: set[str],
    mappings: dict[str, dict[str, Any]],
    referenced_tests: set[str],
) -> None:
    groups = manifest.get("vector_groups")
    if not isinstance(groups, list) or not groups:
        fail(f"{relative}.vector_groups: expected nonempty array")
    for index, group in enumerate(groups):
        owner = f"{relative}.vector_groups[{index}]"
        if not isinstance(group, dict):
            fail(f"{owner}: expected object")
        exact_keys(group, VECTOR_GROUP_KEYS, owner)
        if group["path"] != str(VECTOR_PATH):
            fail(f"{owner}: only the normative Rust subset vector is owned here")
        name = group["group"]
        if name not in EXPECTED_VECTOR_GROUPS:
            fail(f"{owner}: unknown normative vector group {name!r}")
        if name in mappings:
            fail(f"{owner}: duplicate vector group {name}")
        case_ids = require_nonempty_strings(group["case_ids"], f"{owner}.case_ids")
        expected_count, source = EXPECTED_VECTOR_GROUPS[name]
        if len(case_ids) != expected_count:
            fail(f"{owner}: expected {expected_count} frozen case IDs")
        if group["expectation_source"] != source:
            fail(f"{owner}: expectation source must be {source!r}")
        test_ids = require_nonempty_strings(group["tests"], f"{owner}.tests")
        unknown_tests = set(test_ids) - known_tests
        if unknown_tests:
            fail(f"{owner}: unknown tests {sorted(unknown_tests)}")
        referenced_tests.update(test_ids)
        mappings[name] = group


def validate_vector_groups(mappings: dict[str, dict[str, Any]]) -> None:
    if set(mappings) != set(EXPECTED_VECTOR_GROUPS):
        fail(
            "Rust subset vector coverage differs; "
            f"missing={sorted(set(EXPECTED_VECTOR_GROUPS) - set(mappings))} "
            f"unknown={sorted(set(mappings) - set(EXPECTED_VECTOR_GROUPS))}"
        )
    try:
        normative_bytes = (ROOT / VECTOR_PATH).read_bytes()
        execution_bytes = (ROOT / EXECUTION_VECTOR_PATH).read_bytes()
    except OSError as error:
        fail(f"Rust subset vector mirror cannot be read: {error}")
    if execution_bytes != normative_bytes:
        fail(
            f"{EXECUTION_VECTOR_PATH}: hermetic test vector differs byte-for-byte "
            f"from {VECTOR_PATH}"
        )
    vector = load_json(VECTOR_PATH)
    if vector.get("schema") != "mpk.rust.subset.conformance.v0":
        fail(f"{VECTOR_PATH}: wrong schema")
    all_ids: set[str] = set()
    for name, (expected_count, _) in EXPECTED_VECTOR_GROUPS.items():
        cases = vector.get(name)
        if not isinstance(cases, list) or len(cases) != expected_count:
            fail(f"{VECTOR_PATH}.{name}: expected {expected_count} cases")
        vector_ids: list[str] = []
        for index, case in enumerate(cases):
            owner = f"{VECTOR_PATH}.{name}[{index}]"
            if not isinstance(case, dict):
                fail(f"{owner}: expected object")
            case_id = case.get("id")
            if not isinstance(case_id, str) or not ID_PATTERN.fullmatch(case_id):
                fail(f"{owner}: invalid case ID {case_id!r}")
            if case_id in all_ids:
                fail(f"{owner}: duplicate global vector case ID {case_id}")
            all_ids.add(case_id)
            vector_ids.append(case_id)
            if name == "rejected_cases":
                expect = case.get("expect")
                if not isinstance(expect, dict) or set(expect) != {"status", "phase", "code"}:
                    fail(f"{owner}.expect: expected exact status/phase/code")
                status = expect["status"]
                if status not in {"rejected", "source-error", "frontend-error"}:
                    fail(f"{owner}: invalid negative status {status!r}")
                if not isinstance(expect["phase"], str) or not expect["phase"]:
                    fail(f"{owner}: missing exact phase")
                if not isinstance(expect["code"], str) or not CODE_PATTERN.fullmatch(expect["code"]):
                    fail(f"{owner}: invalid stable code")
                if STATUS_EXITS[status] not in {1, 3, 4}:
                    fail(f"{owner}: status does not have a frozen non-usage exit")
            elif name == "same_phase_precedence":
                findings = case.get("findings")
                primary = case.get("primary")
                if (
                    not isinstance(findings, list)
                    or len(findings) < 2
                    or len(findings) != len(set(findings))
                    or primary not in findings
                    or primary != findings[0]
                ):
                    fail(f"{owner}: precedence must freeze the first of unique findings")
            else:
                if case.get("at") != "accept":
                    fail(f"{owner}: exact boundary must accept")
                above = case.get("above", case.get("expect"))
                if not isinstance(above, str) or not above:
                    fail(f"{owner}: boundary+1 must freeze a rejection")
        if mappings[name]["case_ids"] != vector_ids:
            manifest_ids = mappings[name]["case_ids"]
            fail(
                f"{name}: manifest/vector IDs differ; "
                f"missing={sorted(set(vector_ids) - set(manifest_ids))} "
                f"unknown={sorted(set(manifest_ids) - set(vector_ids))}"
            )


def main() -> int:
    validate_root_links()
    catalog: dict[str, tuple[str, str]] = {}
    manifests: list[tuple[Path, dict[str, Any], dict[str, dict[str, Any]]]] = []
    for index, relative in enumerate(CHILD_MANIFESTS):
        manifest = load_json(relative)
        exact_keys(manifest, MANIFEST_KEYS, str(relative))
        expected_schema = (
            "mpk.rust.negative_coverage.v0"
            if index == 0
            else "mpk.rust.adversarial_coverage.v0"
        )
        expected_kind = "negative" if index == 0 else "adversarial"
        if manifest["schema"] != expected_schema or manifest["kind"] != expected_kind:
            fail(f"{relative}: wrong schema/kind pair")
        validate_design_anchor(manifest, relative, index)
        profiles = validate_expectations(manifest, relative)
        collect_tests(manifest, relative, catalog)
        manifests.append((relative, manifest, profiles))

    rules: dict[str, tuple[str, tuple[str, ...]]] = {}
    profiles_by_rule: dict[str, dict[str, Any]] = {}
    vector_mappings: dict[str, dict[str, Any]] = {}
    referenced_tests: set[str] = set()
    for index, (relative, manifest, profiles) in enumerate(manifests):
        before = set(rules)
        allowed = range(1, 19) if index == 0 else range(19, 29)
        local_tests = {test["id"] for test in manifest["tests"]}
        collect_rules(
            manifest,
            relative,
            profiles,
            local_tests,
            allowed,
            rules,
            referenced_tests,
        )
        for rule_id in set(rules) - before:
            profiles_by_rule[rule_id] = profiles[rules[rule_id][0]]
        collect_vector_groups(
            manifest,
            relative,
            local_tests,
            vector_mappings,
            referenced_tests,
        )

    validate_rule_inventory(rules, profiles_by_rule)
    validate_authoritative_codes(manifests)
    validate_vector_groups(vector_mappings)
    unreferenced_tests = set(catalog) - referenced_tests
    if unreferenced_tests:
        fail(f"unreferenced named tests: {sorted(unreferenced_tests)}")
    validate_test_catalog(catalog)

    print(
        "Rust subset negative coverage: "
        f"{len(rules)} atomic rules, "
        f"{sum(count for count, _ in EXPECTED_VECTOR_GROUPS.values())} vector cases, "
        f"{len(catalog)} named tests"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except CoverageError as error:
        print(f"rust subset coverage error: {error}", file=sys.stderr)
        raise SystemExit(1)
