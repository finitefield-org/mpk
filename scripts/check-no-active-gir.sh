#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
  --audit)
    mode="audit"
    ;;
  --strict)
    mode="strict"
    ;;
  *)
    echo "usage: $0 --audit|--strict" >&2
    exit 2
    ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

exec python3 - "$repo_root" "$mode" <<'PY'
from __future__ import annotations

import json
import os
import re
import stat
import subprocess
import sys
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any


REPO_ROOT = Path(sys.argv[1])
MODE = sys.argv[2]
METADATA_PATH = "develop/migrations/gir-to-vir-obsolete-terms.txt"
MANIFEST_PATH = "develop/migrations/gir-to-vir-search-fixtures/manifest.json"
FIXTURE_ROOT = "develop/migrations/gir-to-vir-search-fixtures"
SCRIPT_PATH = "scripts/check-no-active-gir.sh"
MANIFEST_SCHEMA = "mpk.gir_to_vir.search_fixtures.v0"
SCOPES = {
    "exact-global-token",
    "exact-path",
    "schema-qualified-json-field",
    "type/variant-qualified-code-symbol",
}
DISPOSITIONS = {"remove", "rename/regenerate"}
EXPECTED_SCANNER_FILES = {METADATA_PATH, SCRIPT_PATH}
EXPECTED_HISTORICAL = {
    "develop/specs/GIR_V0.md": "always",
    "develop/specs/GO_SUBSET_V0.md": "always",
    "develop/specs/AI_API_V0.md": "always",
    "develop/specs/CERT_V0.md": "always",
    "develop/specs/TRUST_BOUNDARY_V0.md": "always",
    "develop/specs/UNSAFE_POLICY_V0.md": "always",
    "develop/migrations/gir-to-vir-inventory.md": "always",
    "develop/migrations/go-gir-semantic-baseline.json": "always",
    "develop/docs/05_rust_frontend_design.md": "design_status_marker",
    "develop/docs/05_rust_frontend_design-todo.md": "design_status_marker",
}
DESIGN_PATHS = {
    path for path, activation in EXPECTED_HISTORICAL.items()
    if activation == "design_status_marker"
}


class GateError(Exception):
    pass


@dataclass(frozen=True)
class Finding:
    matcher_id: str
    owner: str
    path: str
    line: int
    excerpt: str


def fail(message: str) -> None:
    raise GateError(message)


def strict_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            fail(f"duplicate JSON key {key!r}")
        result[key] = value
    return result


def read_strict_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=strict_object)
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        fail(f"cannot read strict JSON {path.relative_to(REPO_ROOT)}: {error}")
    if not isinstance(value, dict):
        fail(f"{path.relative_to(REPO_ROOT)} must contain one JSON object")
    return value


def normalized_relative_path(value: Any, context: str) -> str:
    if not isinstance(value, str) or not value:
        fail(f"{context} must be a nonempty string")
    if "\\" in value or value.endswith("/"):
        fail(f"{context} must be a normalized POSIX file path: {value!r}")
    parsed = PurePosixPath(value)
    if parsed.is_absolute() or value != parsed.as_posix() or ".." in parsed.parts:
        fail(f"{context} must be repository-relative and normalized: {value!r}")
    return value


def validate_regular_file(relative: str, context: str) -> Path:
    relative = normalized_relative_path(relative, context)
    current = REPO_ROOT
    for component in PurePosixPath(relative).parts:
        current = current / component
        try:
            info = current.lstat()
        except FileNotFoundError:
            fail(f"{context} is missing: {relative}")
        if stat.S_ISLNK(info.st_mode):
            fail(f"{context} must not contain a symlink component: {relative}")
    if not current.is_file():
        fail(f"{context} must name one exact regular file, not a directory: {relative}")
    return current


def require_exact_keys(value: dict[str, Any], required: set[str], optional: set[str], context: str) -> None:
    missing = required - value.keys()
    unknown = value.keys() - required - optional
    if missing:
        fail(f"{context} is missing keys: {sorted(missing)}")
    if unknown:
        fail(f"{context} has unknown keys: {sorted(unknown)}")


def load_matchers() -> list[dict[str, Any]]:
    path = validate_regular_file(METADATA_PATH, "scanner metadata")
    matchers: list[dict[str, Any]] = []
    ids: set[str] = set()
    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        try:
            matcher = json.loads(line, object_pairs_hook=strict_object)
        except json.JSONDecodeError as error:
            fail(f"{METADATA_PATH}:{line_number}: invalid JSON: {error}")
        if not isinstance(matcher, dict):
            fail(f"{METADATA_PATH}:{line_number}: matcher must be an object")
        scope = matcher.get("scope")
        common = {"id", "scope", "disposition", "removal_owner"}
        if scope == "exact-global-token":
            require_exact_keys(matcher, common | {"token"}, set(), f"matcher line {line_number}")
            if not isinstance(matcher["token"], str) or not matcher["token"]:
                fail(f"{METADATA_PATH}:{line_number}: token must be nonempty")
        elif scope == "exact-path":
            require_exact_keys(matcher, common | {"path"}, set(), f"matcher line {line_number}")
            matcher["path"] = normalized_relative_path(matcher["path"], f"matcher line {line_number} path")
        elif scope == "schema-qualified-json-field":
            require_exact_keys(
                matcher,
                common | {"schemas", "field"},
                set(),
                f"matcher line {line_number}",
            )
            schemas = matcher["schemas"]
            if (
                not isinstance(schemas, list)
                or not schemas
                or any(not isinstance(item, str) or not item for item in schemas)
                or len(schemas) != len(set(schemas))
            ):
                fail(f"{METADATA_PATH}:{line_number}: schemas must be unique nonempty strings")
            if not isinstance(matcher["field"], str) or not matcher["field"]:
                fail(f"{METADATA_PATH}:{line_number}: field must be nonempty")
        elif scope == "type/variant-qualified-code-symbol":
            require_exact_keys(matcher, common | {"symbol"}, set(), f"matcher line {line_number}")
            symbol = matcher["symbol"]
            if not isinstance(symbol, str) or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+", symbol):
                fail(f"{METADATA_PATH}:{line_number}: symbol must be fully qualified")
        else:
            fail(f"{METADATA_PATH}:{line_number}: unknown matcher scope {scope!r}")

        matcher_id = matcher.get("id")
        if not isinstance(matcher_id, str) or not re.fullmatch(r"[a-z0-9][a-z0-9-]*", matcher_id):
            fail(f"{METADATA_PATH}:{line_number}: invalid matcher id {matcher_id!r}")
        if matcher_id in ids:
            fail(f"{METADATA_PATH}:{line_number}: duplicate matcher id {matcher_id!r}")
        ids.add(matcher_id)
        if matcher.get("disposition") not in DISPOSITIONS:
            fail(f"{METADATA_PATH}:{line_number}: invalid disposition")
        if not re.fullmatch(r"GO-VIR-02-T12", str(matcher.get("removal_owner", ""))):
            fail(f"{METADATA_PATH}:{line_number}: every removal must be owned by GO-VIR-02-T12")
        matchers.append(matcher)
    if not matchers:
        fail(f"{METADATA_PATH} contains no matchers")
    if {matcher["scope"] for matcher in matchers} != SCOPES:
        fail("matcher metadata must exercise exactly the four frozen matcher scopes")
    return matchers


def validate_manifest() -> tuple[dict[str, Any], set[str], set[str], set[str]]:
    path = validate_regular_file(MANIFEST_PATH, "fixture manifest")
    manifest = read_strict_json(path)
    require_exact_keys(
        manifest,
        {"schema", "design_status_marker", "scanner_files", "historical_records", "fixtures"},
        set(),
        "fixture manifest",
    )
    if manifest["schema"] != MANIFEST_SCHEMA:
        fail(f"fixture manifest schema must be {MANIFEST_SCHEMA!r}")
    marker = manifest["design_status_marker"]
    if not isinstance(marker, str) or not marker:
        fail("design_status_marker must be nonempty")

    scanner_values = manifest["scanner_files"]
    if not isinstance(scanner_values, list):
        fail("scanner_files must be an array")
    scanner_files = {
        normalized_relative_path(value, "scanner_files entry") for value in scanner_values
    }
    if len(scanner_files) != len(scanner_values):
        fail("scanner_files contains a duplicate")
    if scanner_files != EXPECTED_SCANNER_FILES:
        fail(
            "scanner_files must be the exact frozen scanner implementation/metadata set; "
            f"got {sorted(scanner_files)}"
        )
    for relative in scanner_files:
        validate_regular_file(relative, "scanner exclusion")

    historical_values = manifest["historical_records"]
    if not isinstance(historical_values, list):
        fail("historical_records must be an array")
    historical: dict[str, str] = {}
    for index, item in enumerate(historical_values):
        if not isinstance(item, dict):
            fail(f"historical_records[{index}] must be an object")
        require_exact_keys(item, {"path", "activation"}, set(), f"historical_records[{index}]")
        relative = normalized_relative_path(item["path"], f"historical_records[{index}].path")
        activation = item["activation"]
        if activation not in {"always", "design_status_marker"}:
            fail(f"historical_records[{index}] has unknown activation {activation!r}")
        if relative in historical:
            fail(f"historical_records contains duplicate path {relative!r}")
        historical[relative] = activation
        validate_regular_file(relative, "historical record")
    if historical != EXPECTED_HISTORICAL:
        fail("historical_records must equal the frozen exact-file allowlist")

    fixture_values = manifest["fixtures"]
    if not isinstance(fixture_values, list):
        fail("fixtures must be an array")
    fixtures: set[str] = set()
    manifest_roles = 0
    for index, item in enumerate(fixture_values):
        if not isinstance(item, dict):
            fail(f"fixtures[{index}] must be an object")
        role = item.get("role")
        if role == "manifest":
            require_exact_keys(item, {"path", "role"}, set(), f"fixtures[{index}]")
            manifest_roles += 1
        elif role == "positive":
            require_exact_keys(
                item,
                {"path", "role", "expected_matchers"},
                {"virtual_path"},
                f"fixtures[{index}]",
            )
        elif role == "negative":
            require_exact_keys(
                item,
                {"path", "role"},
                {"virtual_path", "guards"},
                f"fixtures[{index}]",
            )
        else:
            fail(f"fixtures[{index}] has unknown role {role!r}")
        relative = normalized_relative_path(item["path"], f"fixtures[{index}].path")
        if relative in fixtures:
            fail(f"fixtures contains duplicate path {relative!r}")
        if not relative.startswith(FIXTURE_ROOT + "/"):
            fail(f"fixture is outside exact fixture root: {relative}")
        fixtures.add(relative)
        validate_regular_file(relative, "fixture exclusion")
        if "virtual_path" in item:
            normalized_relative_path(item["virtual_path"], f"fixtures[{index}].virtual_path")
    if manifest_roles != 1 or MANIFEST_PATH not in fixtures:
        fail("fixtures must enumerate the manifest itself exactly once")

    actual_fixture_files: set[str] = set()
    root = REPO_ROOT / FIXTURE_ROOT
    for current, directory_names, file_names in os.walk(root, followlinks=False):
        current_path = Path(current)
        for name in directory_names:
            candidate = current_path / name
            if candidate.is_symlink():
                fail(f"fixture root contains symlinked directory: {candidate.relative_to(REPO_ROOT)}")
        for name in file_names:
            candidate = current_path / name
            relative = candidate.relative_to(REPO_ROOT).as_posix()
            validate_regular_file(relative, "fixture-root file")
            actual_fixture_files.add(relative)
    if fixtures != actual_fixture_files:
        fail(
            "fixture manifest must enumerate every fixture-root regular file exactly once; "
            f"missing={sorted(actual_fixture_files - fixtures)}, unknown={sorted(fixtures - actual_fixture_files)}"
        )

    historical_files = set(historical)
    classes = {
        "scanner": scanner_files,
        "fixtures": fixtures,
        "historical": historical_files,
    }
    names = list(classes)
    for index, left_name in enumerate(names):
        for right_name in names[index + 1:]:
            overlap = classes[left_name] & classes[right_name]
            if overlap:
                fail(f"exclusion classes {left_name}/{right_name} overlap: {sorted(overlap)}")
    return manifest, scanner_files, fixtures, historical_files


def token_pattern(token: str) -> re.Pattern[str]:
    prefix = r"(?<![A-Za-z0-9_])" if re.match(r"[A-Za-z0-9_]", token[0]) else ""
    suffix = r"(?![A-Za-z0-9_])" if re.match(r"[A-Za-z0-9_]", token[-1]) else ""
    return re.compile(prefix + re.escape(token) + suffix)


def symbol_pattern(symbol: str) -> re.Pattern[str]:
    components = symbol.split("::")
    body = r"\s*::\s*".join(re.escape(component) for component in components)
    return re.compile(r"(?<![A-Za-z0-9_])" + body + r"(?![A-Za-z0-9_])")


def path_matches(candidate: str, obsolete: str) -> bool:
    return candidate == obsolete or candidate.startswith(obsolete + "/")


def line_for_offset(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def excerpt_for_line(text: str, line: int) -> str:
    if line <= 0:
        return "path match"
    lines = text.splitlines()
    if line > len(lines):
        return "content match"
    value = " ".join(lines[line - 1].strip().split())
    return value[:160]


def object_has_qualified_field(value: Any, schemas: set[str], field: str, inherited: set[str] | None = None) -> bool:
    active = set() if inherited is None else set(inherited)
    if isinstance(value, dict):
        for schema_key in ("schema", "schema_version"):
            schema_value = value.get(schema_key)
            if isinstance(schema_value, str):
                active.add(schema_value)
        if field in value and active & schemas:
            return True
        return any(object_has_qualified_field(child, schemas, field, active) for child in value.values())
    if isinstance(value, list):
        return any(object_has_qualified_field(child, schemas, field, active) for child in value)
    return False


def detect(relative: str, text: str, matchers: list[dict[str, Any]]) -> list[Finding]:
    findings: list[Finding] = []
    parsed_json: Any = None
    parsed_json_available = False
    stripped = text.lstrip()
    if stripped.startswith("{") or stripped.startswith("["):
        try:
            parsed_json = json.loads(text)
            parsed_json_available = True
        except json.JSONDecodeError:
            pass

    for matcher in matchers:
        matcher_id = matcher["id"]
        owner = matcher["removal_owner"]
        scope = matcher["scope"]
        offsets: list[int] = []
        if scope == "exact-global-token":
            offsets = [match.start() for match in token_pattern(matcher["token"]).finditer(text)]
        elif scope == "exact-path":
            obsolete = matcher["path"]
            if path_matches(relative, obsolete):
                findings.append(Finding(matcher_id, owner, relative, 0, "path match"))
            content_pattern = re.compile(
                r"(?<![A-Za-z0-9._/-])" + re.escape(obsolete) + r"(?![A-Za-z0-9._/-])"
            )
            offsets = [match.start() for match in content_pattern.finditer(text)]
        elif scope == "type/variant-qualified-code-symbol":
            offsets = [match.start() for match in symbol_pattern(matcher["symbol"]).finditer(text)]
        elif scope == "schema-qualified-json-field":
            schemas = set(matcher["schemas"])
            field = matcher["field"]
            qualified = (
                parsed_json_available
                and object_has_qualified_field(parsed_json, schemas, field)
            )
            if not parsed_json_available:
                qualified = any(token_pattern(schema).search(text) for schema in schemas) and bool(
                    token_pattern(field).search(text)
                )
            if qualified:
                field_match = token_pattern(field).search(text)
                offsets = [field_match.start() if field_match else 0]
        for offset in offsets:
            line = line_for_offset(text, offset)
            findings.append(
                Finding(matcher_id, owner, relative, line, excerpt_for_line(text, line))
            )
    return findings


def run_fixture_self_test(manifest: dict[str, Any], matchers: list[dict[str, Any]]) -> None:
    matcher_by_id = {matcher["id"]: matcher for matcher in matchers}
    positive_scopes: set[str] = set()
    positive_context_ids: set[str] = set()
    negative_context_ids: set[str] = set()
    for index, fixture in enumerate(manifest["fixtures"]):
        role = fixture["role"]
        if role == "manifest":
            continue
        relative = fixture["path"]
        virtual_path = fixture.get("virtual_path", relative)
        text = (REPO_ROOT / relative).read_text(encoding="utf-8")
        found = {finding.matcher_id for finding in detect(virtual_path, text, matchers)}
        if role == "positive":
            expected_values = fixture["expected_matchers"]
            if (
                not isinstance(expected_values, list)
                or not expected_values
                or any(not isinstance(value, str) for value in expected_values)
                or len(expected_values) != len(set(expected_values))
            ):
                fail(f"fixtures[{index}].expected_matchers must be unique matcher ids")
            expected = set(expected_values)
            unknown = expected - matcher_by_id.keys()
            if unknown:
                fail(f"positive fixture {relative} references unknown matchers: {sorted(unknown)}")
            if found != expected:
                fail(
                    f"positive fixture {relative} mismatch: expected={sorted(expected)}, found={sorted(found)}"
                )
            positive_scopes.update(matcher_by_id[matcher_id]["scope"] for matcher_id in expected)
            positive_context_ids.update(
                matcher_id for matcher_id in expected
                if matcher_by_id[matcher_id]["scope"]
                in {"schema-qualified-json-field", "type/variant-qualified-code-symbol"}
            )
        else:
            if found:
                fail(f"negative fixture {relative} was detected by {sorted(found)}")
            guards = fixture.get("guards", [])
            if (
                not isinstance(guards, list)
                or any(not isinstance(value, str) for value in guards)
                or len(guards) != len(set(guards))
            ):
                fail(f"negative fixture {relative} guards must be unique matcher ids")
            unknown = set(guards) - matcher_by_id.keys()
            if unknown:
                fail(f"negative fixture {relative} guards unknown matchers: {sorted(unknown)}")
            for matcher_id in guards:
                if matcher_by_id[matcher_id]["scope"] not in {
                    "schema-qualified-json-field",
                    "type/variant-qualified-code-symbol",
                }:
                    fail(f"negative guard {matcher_id} is not context-qualified")
            negative_context_ids.update(guards)
    if positive_scopes != SCOPES:
        fail(f"positive fixtures must detect every matcher scope; got {sorted(positive_scopes)}")
    contextual = {
        matcher["id"] for matcher in matchers
        if matcher["scope"]
        in {"schema-qualified-json-field", "type/variant-qualified-code-symbol"}
    }
    if positive_context_ids != contextual or negative_context_ids != contextual:
        fail(
            "every context-qualified matcher must have positive and retained-context negative coverage; "
            f"positive_missing={sorted(contextual - positive_context_ids)}, "
            f"negative_missing={sorted(contextual - negative_context_ids)}"
        )


def active_historical_allowlist(manifest: dict[str, Any]) -> set[str]:
    marker = manifest["design_status_marker"]
    design_ready = all(marker in (REPO_ROOT / path).read_text(encoding="utf-8") for path in DESIGN_PATHS)
    active: set[str] = set()
    for item in manifest["historical_records"]:
        if item["activation"] == "always" or design_ready:
            active.add(item["path"])
    return active


def repository_files() -> list[str]:
    command = [
        "/usr/bin/git",
        "-C",
        str(REPO_ROOT),
        "ls-files",
        "--cached",
        "--others",
        "--exclude-standard",
        "-z",
    ]
    try:
        result = subprocess.run(command, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    except (OSError, subprocess.CalledProcessError) as error:
        fail(f"cannot enumerate repository files with /usr/bin/git: {error}")
    try:
        paths = [entry.decode("utf-8") for entry in result.stdout.split(b"\0") if entry]
    except UnicodeDecodeError as error:
        fail(f"git returned a non-UTF-8 repository path: {error}")
    if len(paths) != len(set(paths)):
        fail("git file enumeration returned duplicate paths")
    return sorted(paths)


def scan_repository(
    matchers: list[dict[str, Any]], excluded: set[str]
) -> tuple[list[Finding], int]:
    findings: list[Finding] = []
    scanned = 0
    for relative in repository_files():
        relative = normalized_relative_path(relative, "git-enumerated path")
        if relative in excluded:
            continue
        try:
            (REPO_ROOT / relative).lstat()
        except FileNotFoundError:
            # The index still lists a tracked file deleted by an unstaged
            # cutover change. A deleted file is not active scan input.
            continue
        path = validate_regular_file(relative, "active repository file")
        scanned += 1
        try:
            raw = path.read_bytes()
        except OSError as error:
            fail(f"cannot read active repository file {relative}: {error}")
        text = raw.decode("utf-8", errors="ignore")
        findings.extend(detect(relative, text, matchers))
    return findings, scanned


def report(findings: list[Finding], scanned: int, historical_count: int) -> int:
    print(
        f"GIR obsolete-interface self-test: ok "
        f"({scanned} active files scanned; {historical_count} historical files allowed)"
    )
    if not findings:
        print("No active obsolete GIR interfaces found.")
        return 0

    grouped: dict[tuple[str, str, str], list[Finding]] = defaultdict(list)
    for finding in findings:
        grouped[(finding.matcher_id, finding.owner, finding.path)].append(finding)
    print(
        f"Found {len(findings)} obsolete-interface occurrences in "
        f"{len({finding.path for finding in findings})} active files."
    )
    for (matcher_id, owner, path), values in sorted(grouped.items()):
        first = min(values, key=lambda value: value.line)
        location = path if first.line == 0 else f"{path}:{first.line}"
        print(
            f"{matcher_id}\towner={owner}\tcount={len(values)}\t{location}\t{first.excerpt}"
        )
    if MODE == "audit":
        print("Audit mode records findings without failing; strict mode must fail before cutover.")
        return 0
    print("Strict mode rejects active obsolete GIR interfaces.", file=sys.stderr)
    return 1


def main() -> int:
    try:
        matchers = load_matchers()
        manifest, scanner_files, fixtures, _historical = validate_manifest()
        run_fixture_self_test(manifest, matchers)
        historical = active_historical_allowlist(manifest)
        excluded = scanner_files | fixtures | historical
        findings, scanned = scan_repository(matchers, excluded)
        return report(findings, scanned, len(historical))
    except GateError as error:
        print(f"GIR obsolete-interface gate configuration error: {error}", file=sys.stderr)
        return 2


raise SystemExit(main())
PY
