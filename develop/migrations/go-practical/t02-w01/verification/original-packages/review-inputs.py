"""Review full changed files and every retained original input; outside sandbox."""
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

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
BASE = "ed67a4c388defe8babdbc55f29851552562121d9"
REPORT = OUT / "input-review.json"


def git(*args):
    return subprocess.check_output(["/usr/bin/git", *args])


def seal(path, raw=None):
    raw = Path(path).read_bytes() if raw is None else raw
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


assert git("rev-parse", "HEAD").decode().strip() == BASE
scanner = Path("scripts/check-no-active-gir.sh")
code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0].rsplit("raise SystemExit(main())", 1)[0]
module = types.ModuleType("mpk_original_package_input_review")
module.__file__ = str(ROOT / scanner)
sys.modules[module.__name__] = module
sys.argv = [str(scanner), str(ROOT), "strict"]
exec(compile(code, str(scanner), "exec"), module.__dict__)
matchers = module.load_matchers()
manifest, scanner_files, fixtures, _ = module.validate_manifest()
module.run_fixture_self_test(manifest, matchers)
assert (len(matchers), len(module.SCOPES), len(manifest["sealed_records"])) == (127, 4, 39)
configured = scanner_files | fixtures | module.active_retained_record_exclusions(manifest)
findings = []
decoded = []
cache = {}
path_rules = [m for m in matchers if m["scope"] == "exact-path"]


def scan(name, raw):
    key = (hashlib.sha256(raw).hexdigest(), tuple(module.path_matches(name, m["path"]) for m in path_rules))
    if key not in cache:
        cache[key] = [vars(f) for f in module.detect(name, raw.decode(errors="ignore"), matchers)]
    findings.extend({**finding, "path": name} for finding in cache[key])


def payload(name, text):
    raw = base64.b64decode(text, validate=True)
    scan(name, raw)
    decoded.append(seal(name, raw))
    return raw


selected = git("diff", "--name-only", "-z", "HEAD", "--") + git("ls-files", "--others", "--exclude-standard", "-z")
paths = sorted(set(selected.decode().split("\0")) - {"", str(REPORT.relative_to(ROOT))})
inputs = []
links = []
for name in paths:
    path = ROOT / name
    assert path.is_file() and not path.is_symlink(), name
    raw = path.read_bytes()
    inputs.append(seal(name, raw))
    if name not in configured:
        scan(name, raw)
    if path.suffix == ".md":
        for target in re.findall(r"(?<!!)\[[^\]]*\]\(([^\n)]+)\)", raw.decode()):
            target = target.strip().strip("<>")
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target) or target.startswith("#"):
                continue
            clean = urllib.parse.unquote(target.split("#", 1)[0])
            links.append({"document": name, "target": target, "exists": (path.parent / clean).resolve().exists()})

external = Path("/private/tmp/mpk-go-practical-original-packages-integrated-replay")
objects = external / "evidence/original-source-objects"
originals = []
for path in sorted(objects.glob("*.json")):
    raw = path.with_suffix(".raw").read_bytes()
    obj = json.loads(path.read_bytes())
    assert obj["id"] == path.stem and obj["schema"] == "mpk.go.source_object.v1"
    assert len(raw) == obj["bytes"] and hashlib.sha256(raw).hexdigest() == obj["raw_sha256"]
    data = {key: value for key, value in obj.items() if key != "id"}
    canonical = json.dumps(data, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0" + canonical).hexdigest() == obj["id"]
    scan(str(path), path.read_bytes())
    scan(str(path.with_suffix(".raw")), raw)
    originals.append({"metadata": seal(path), "raw": seal(path.with_suffix(".raw")), "object": obj})
assert len(originals) == 5674
for path in sorted((external / "evidence").glob("*.json")):
    raw = path.read_bytes()
    scan(str(path), raw)
    data = json.loads(raw)
    if path.name == "replay-native-original-export.json":
        for key in ["module_output", "stdout", "stderr"]:
            payload(str(path) + "#" + key, data[key])
    elif path.name == "replay-original-application.json":
        for owner, files in data["original_files"].items():
            for name, value in files.items():
                payload(str(path) + "#" + owner + "/" + name, value)
    else:
        raise AssertionError(path)
owned_path = OUT / "integrated/selected/original-packages-owned.json"
owned = json.loads(owned_path.read_bytes())
for key in ["manifest", "policy", "helper", "modules", "packages"]:
    payload(str(owned_path) + "#" + key, owned[key])
for pkg in owned["inventory"]:
    for source in pkg["sources"]:
        raw = payload(str(owned_path) + "#" + source["object"]["id"], source["raw"])
        assert hashlib.sha256(raw).hexdigest() == source["object"]["raw_sha256"] and len(raw) == source["object"]["bytes"]
    if pkg["original_archive"] is not None:
        raw = payload(str(owned_path) + "#" + pkg["export_object"]["id"], pkg["original_archive"])
        assert hashlib.sha256(raw).hexdigest() == pkg["export_object"]["raw_sha256"] and len(raw) == pkg["export_object"]["bytes"]
    else:
        assert pkg["intrinsic"] and pkg["import_path"] == "unsafe"
frozen = json.loads(Path("develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json").read_bytes())["inputs"]
assert len(frozen) == 255
assert all(seal(row["path"]) == row for row in frozen)
source_inputs = json.loads((OUT / "integrated/inputs.json").read_bytes())["sources"]
assert all(seal(row["path"])["sha256"] == row["sha256"] for row in source_inputs)
missing = [row for row in links if not row["exists"]]
unchanged = all(seal(row["path"]) == row for row in inputs)
record = {"status": "passed" if not findings and not missing and unchanged else "failed",
          "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "basis_commit": BASE,
          "inputs": inputs, "source_inputs_unchanged": unchanged,
          "findings": findings, "missing_links": missing, "links_checked": len(links),
          "complete_original_source_export_object_count": len(originals), "complete_original_objects": originals,
          "complete_decoded_payloads": decoded, "frozen_inputs_unchanged": 255,
          "matcher_scope_seal_counts": [127, 4, 39], "new_source_JSON_directory_or_blanket_exclusions": False,
          "native_release_acceptance": False, "full_T_gate_passed": False}
REPORT.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({k: v for k, v in record.items() if k not in {"inputs", "complete_original_objects", "complete_decoded_payloads"}}))
raise SystemExit(record["status"] != "passed")
