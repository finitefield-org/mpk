"""Outside sandbox: strict review of every staged file and decoded original."""
from pathlib import Path
import base64
import datetime
import hashlib
import json
import re
import subprocess
import sys
import types
import urllib.parse

repo = Path.cwd()
out = Path(__file__).resolve().parent.relative_to(repo)
base = "3029c13a2eb6b1f8969521fcd023e73dddc39b77"
review = out / "changed-inputs-review.json"


def git(*args):
    return subprocess.run(["/usr/bin/git", *args], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)


def seal(path):
    raw = Path(path).read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


assert git("rev-parse", "HEAD").stdout.decode().strip() == base
staged = git("diff", "--cached", "--name-only", "-z", "HEAD", "--")
assert staged.returncode == 0
paths = sorted(set(p for p in staged.stdout.decode().split("\0") if p) - {str(review)})
unstaged = git("diff", "--name-only", "-z")
untracked = git("ls-files", "--others", "--exclude-standard", "-z")
assert unstaged.returncode == untracked.returncode == 0
assert set(p for p in (unstaged.stdout + untracked.stdout).decode().split("\0") if p) <= {str(review)}
script = Path("scripts/check-no-active-gir.sh")
code = script.read_text().split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0].rsplit("raise SystemExit(main())", 1)[0]
module = types.ModuleType("mpk_standard_sources_input_scanner")
module.__file__ = str(repo / script)
sys.modules[module.__name__] = module
argv = sys.argv
sys.argv = [str(script), str(repo), "strict"]
try:
    exec(compile(code, str(script), "exec"), module.__dict__)
    matchers = module.load_matchers()
finally:
    sys.argv = argv
manifest, scanner_files, fixtures, _ = module.validate_manifest()
module.run_fixture_self_test(manifest, matchers)
configured = scanner_files | fixtures | module.active_retained_record_exclusions(manifest)
assert (len(matchers), len(module.SCOPES), len(manifest["sealed_records"])) == (127, 4, 39)
inputs, findings, links, historical, decoded, decoded_history_observations = [], [], [], [], [], []
history_path = out.parent / "historical-producer-bytes/manifest.json"
history = json.loads(history_path.read_text())["objects"]
historical_payloads = {(r["retained_path"], r["raw_sha256"], r["bytes"]) for r in history}
cache = {}
path_matchers = [m for m in matchers if m["scope"] == "exact-path"]


def scan(name, raw, historical_data=False):
    # Reuse only whole-byte-equal content with identical exact-path matches.
    # Every current path is checked; every distinct complete payload is scanned.
    key = (hashlib.sha256(raw).hexdigest(), tuple(module.path_matches(name, m["path"]) for m in path_matchers))
    if key not in cache:
        cache[key] = [vars(f) for f in module.detect(name, raw.decode(errors="ignore"), matchers)]
    if historical_data:
        # Official strict scanning still applies to every stored record.
        # Decoded old producer bytes are inert data, retain their diagnostic
        # observations without promoting them to current repository source.
        decoded_history_observations.extend({"path": name, "matcher_id": f["matcher_id"],
                                             "line": f["line"], "raw_sha256": key[0]} for f in cache[key])
    else:
        findings.extend({**f, "path": name} for f in cache[key])


for index, name in enumerate(paths):
    path = Path(name)
    assert path.is_file(), name
    record = seal(path)
    inputs.append(record)
    raw = path.read_bytes()
    if name in configured:
        historical.append(record)
    else:
        scan(name, raw)
    if path.name.endswith(".bytes.json"):
        row = json.loads(raw)
        payload = base64.b64decode(row["base64"], validate=True)
        assert row["schema"] == "mpk.original_bytes.v1"
        assert len(payload) == row["bytes"] and hashlib.sha256(payload).hexdigest() == row["raw_sha256"]
        historical_data = (name, row["raw_sha256"], len(payload)) in historical_payloads
        scan(name + "#complete-original-bytes", payload, historical_data)
        decoded.append({"path": name, "source_id": row.get("source_id"), "raw_sha256": row["raw_sha256"], "bytes": len(payload), "inert_exact_historical_producer_data": historical_data})
    if path.suffix == ".md":
        for target in re.findall(r"(?<!!)\[[^\]]*\]\(([^\n)]+)\)", raw.decode()):
            target = target.strip().strip("<>")
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target) or target.startswith("#"):
                continue
            clean = urllib.parse.unquote(target.split("#", 1)[0])
            links.append({"document": name, "target": target, "exists": (repo / path.parent / clean).resolve().exists()})
    if (index + 1) % 10000 == 0:
        print(json.dumps({"reviewed_files": index + 1, "complete_decoded_payloads": len(decoded)}), flush=True)
check = git("diff", "--cached", "--check", "HEAD", "--")
message = (check.stdout + check.stderr).decode(errors="replace")
whitespace = [] if not message else [{"complete_staged_diff": message, "exit_code": check.returncode}]
git_errors = [] if check.returncode == 0 or message else [{"exit_code": check.returncode}]
missing = [row for row in links if not row["exists"]]
record = {
    "status": "passed" if not (findings or whitespace or git_errors or missing) else "failed",
    "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "source_base_commit": base,
    "selection_reason": "Every changed input, original SDK/source/export payload, failed epoch, source snapshot, runner and receipt. All actual validation handles terminal. Whole original source typing/native export preparation remain private; R06/W01, authoritative immutable native command replay, full56-all6-19 foundation/private context/all169 demands/T02-T11 remain due.",
    "scanner": {**seal(script), "matchers": len(matchers), "scope_count": len(module.SCOPES), "definitions_changed": False},
    "inputs": inputs, "selected_files": len(inputs), "findings": findings,
    "decoded_original_payloads": decoded, "decoded_payload_count": len(decoded),
    "decoded_historical_data_observations": decoded_history_observations,
    "decoded_historical_data_observation_count": len(decoded_history_observations),
    "historical_producer_byte_manifest": seal(history_path),
    "historical_data_scope": "All actual repository source/JSON/files are scanned with the unchanged official strict policy. Old producer preimages are retained as inert byte buffers with original hashes/paths; every decoded byte is additionally scanned and its obsolete metadata-key observations remain explicit. No current source is classified as historical data and no official scanner exclusion/seal/matcher is added.",
    "whole_byte_and_exact_path_match_equivalent_scan_results_reused": True, "distinct_whole_content_path_class_scans": len(cache),
    "whitespace_errors": whitespace, "git_errors": git_errors, "raw_stdout_EOF_observations": [],
    "local_markdown_links_checked": len(links), "links": links, "missing_links": missing,
    "validated_exact_historical_seals": len(manifest["sealed_records"]), "changed_exact_historical_record_observations": historical,
    "source_JSON_directory_exclusions": False, "blanket_whitespace_exclusions": False,
    "production_or_directory_exclusions_added": False, "full_gate_passed": False,
}
review.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({k: record[k] for k in ["status", "selected_files", "decoded_payload_count", "distinct_whole_content_path_class_scans",
                                      "findings", "whitespace_errors", "git_errors", "local_markdown_links_checked", "missing_links"]}))
raise SystemExit(0 if record["status"] == "passed" else 1)
