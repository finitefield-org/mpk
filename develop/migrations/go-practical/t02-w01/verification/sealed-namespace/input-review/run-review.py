"""Review every changed input with unchanged official strict scanner policy."""
from pathlib import Path
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
base = "7405cf5fd5c88f0e31ca96076618e296869ad095"
review = out / "changed-inputs-review.json"


def git(*args):
    return subprocess.run(["/usr/bin/git", *args], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


assert git("rev-parse", "HEAD").stdout.decode().strip() == base
changed = git("diff", "--name-only", "-z", "HEAD", "--")
untracked = git("ls-files", "--others", "--exclude-standard", "-z")
assert changed.returncode == untracked.returncode == 0
paths = sorted(set(p for p in (changed.stdout + untracked.stdout).decode().split("\0") if p) - {str(review)})
script = Path("scripts/check-no-active-gir.sh")
raw = script.read_text()
code = raw.split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0]
assert code.rstrip().endswith("raise SystemExit(main())")
code = code.rsplit("raise SystemExit(main())", 1)[0]
module = types.ModuleType("mpk_sealed_namespace_input_scanner")
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
inputs, findings, whitespace, git_errors, links, historical = [], [], [], [], [], []
for name in paths:
    path = Path(name)
    assert path.is_file(), name
    record = seal(path)
    inputs.append(record)
    content = path.read_text(errors="ignore")
    if name in configured:
        historical.append(record)
    else:
        findings.extend(vars(f) for f in module.detect(name, content, matchers))
    run = git("diff", "--no-index", "--check", "--", "/dev/null", name)
    message = (run.stdout + run.stderr).decode(errors="replace")
    if message:
        whitespace.append({"path": name, "exit_code": run.returncode, "message": message})
    elif run.returncode not in (0, 1):
        git_errors.append({"path": name, "exit_code": run.returncode})
    if path.suffix == ".md":
        for target in re.findall(r"(?<!!)\[[^\]]*\]\(([^\n)]+)\)", content):
            target = target.strip().strip("<>")
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target) or target.startswith("#"):
                continue
            clean = urllib.parse.unquote(target.split("#", 1)[0])
            links.append({"document": name, "target": target, "exists": (repo / path.parent / clean).resolve().exists()})
missing = [row for row in links if not row["exists"]]
record = {
    "status": "passed" if not (findings or whitespace or git_errors or missing) else "failed",
    "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "source_base_commit": base,
    "selection_reason": "All changed production capture/snapshot code, six new snapshot roots, original failed collector alias sources/raw output, complete original/final source snapshots,34-root101-subtest current and7-root11-subtest legacy evidence, current vet/format, two actual fresh-cache170-input frontend builds,17 independent complete private preimages/99 full source identities,255 unchanged frozen inputs and current requirements/ledger/README pointers. Review exact bytes with unchanged official127rules/four scopes/39seals and all local Markdown links. No source/JSON/directory/blanket whitespace exclusions. All169 original application demands, full56-family foundation/private context and T02-T11 remain due; CSharp tests/intermediate full gate remain skipped.",
    "scanner": {**seal(script), "matchers": len(matchers), "scope_count": len(module.SCOPES), "definitions_changed": False},
    "inputs": inputs, "selected_files": len(inputs), "findings": findings,
    "whitespace_errors": whitespace, "git_errors": git_errors,
    "local_markdown_links_checked": len(links), "links": links, "missing_links": missing,
    "validated_exact_historical_seals": len(manifest["sealed_records"]),
    "changed_exact_historical_record_observations": historical,
    "source_JSON_directory_exclusions": False, "blanket_whitespace_exclusions": False,
    "production_or_directory_exclusions_added": False, "full_gate_passed": False,
}
review.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ["status", "selected_files", "findings", "whitespace_errors", "git_errors", "local_markdown_links_checked", "missing_links"]}))
raise SystemExit(0 if record["status"] == "passed" else 1)
