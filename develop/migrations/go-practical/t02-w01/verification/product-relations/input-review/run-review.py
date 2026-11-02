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
base = "27490ecb06ff728162bb9499c7b5773c23b83514"
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
module = types.ModuleType("mpk_product_relations_input_scanner")
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
eofs = []
observations = json.loads((out / "raw-formatter-observations.json").read_text())["observations"]
assert observations
assert all(Path(p).is_file() and row["kind"] == "exact_original_libtest_stdout_EOF" for p, row in observations.items())
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
        exact = observations.get(name)
        if exact and exact["sha256"] == record["sha256"] and exact["bytes"] == record["bytes"] and exact["kind"] == "exact_original_libtest_stdout_EOF" and all(line.endswith(": new blank line at EOF.") for line in message.splitlines() if line):
            eofs.append({**record, "exact_raw_stdout_EOF_preserved": True, "git_whitespace_output": message})
        else:
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
assert {row["path"] for row in eofs} == set(observations)
record = {
    "status": "passed" if not (findings or whitespace or git_errors or missing) else "failed",
    "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "source_base_commit": base,
    "selection_reason": "Actual typed relation-witness products and generic equivalence/update/map/comparator laws plus every16 arbitrary mixed-width word-pair equality/Int.Equivalent bridge and arbitrary balanced-Int representative shifts. Preserve all78 product/3489 signed-equality identities and complete old raw hashes, original compiler E0282 and receipt subtype-observer source/raw failure epochs, all84 actual same-whole-byte current Rust/Go outcomes, four zero-axiom positives and80 genuine type rejections. Four selected Rust roots, Clippy and format pass, all actual handles terminal, all255 frozen inputs unchanged. The unchanged65-arity layout matrix and prior93600 leaf obligations remain retained without an unrelated rerun or new all65 relation-consumer claim. Official127 rules/4 scopes/39 seals remain unchanged; only exact actual per-file hash/length original libtest stdout EOF observations. No source/JSON/directory or blanket whitespace exclusion. Full56 source/carrier/nominal/type/operation/storage/lifetime/state/program foundation, complete compiler-derived roots/private descriptor/registry/context/retained policies/immutable original package/export/selected-closure replay/all169 application demands/T02-T11 remain required; R06 pending. No CSharp/agents/Actions/intermediate full gate or public/source/application/Linux release claim.",
    "scanner": {**seal(script), "matchers": len(matchers), "scope_count": len(module.SCOPES), "definitions_changed": False},
    "inputs": inputs, "selected_files": len(inputs), "findings": findings, "raw_stdout_EOF_observations": eofs,
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
