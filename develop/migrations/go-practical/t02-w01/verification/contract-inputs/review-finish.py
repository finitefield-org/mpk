"""Incremental finish review after the complete archived-input review."""
from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys
import types
import urllib.parse

REPO = Path.cwd()
ROOT = Path(__file__).resolve().parent
sys.path = [p for p in sys.path if Path(p or ".").resolve() != ROOT]


def git(*args):
    return subprocess.check_output(["/usr/bin/git", *args])


def seal(path):
    raw = path.read_bytes()
    return {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(), "mode": path.stat().st_mode & 0o777}


def logical(path):
    p = Path(path)
    return str(p.relative_to(REPO)) if p.is_absolute() else str(p)


def main():
    previous_path = ROOT / "input-review.json"
    previous = json.loads(previous_path.read_bytes())
    assert previous["status"] == "passed" and previous["new_findings"] == previous["unchanged_basis_findings"] == []
    assert git("rev-parse", "HEAD").decode().strip() == previous["basis_commit"]
    scanner = REPO / "scripts/check-no-active-gir.sh"
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0].rsplit("raise SystemExit(main())", 1)[0]
    module = types.ModuleType("contract_input_finish_review")
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), "strict"]
    exec(compile(code, str(scanner), "exec"), module.__dict__)
    matchers = module.load_matchers()
    manifest, configured, fixtures, _ = module.validate_manifest()
    assert (len(matchers), len(module.SCOPES), len(manifest["sealed_records"]), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
    for name in configured | fixtures:
        assert (REPO / name).read_bytes() == git("show", previous["basis_commit"] + ":" + name)
    known = {}
    for row in previous["current_inputs"]:
        assert row == seal(Path(row["path"])), "Previously reviewed current files must stay exact"
        known[logical(row["path"])] = row
    output_path = ROOT / "finish-review.json"
    names = set((git("diff", "--name-only", "-z", "HEAD") + git("ls-files", "--others", "--exclude-standard", "-z")).decode().split("\0")) - {"", logical(output_path)}
    assert set(known) <= names
    additional, findings, links = [], [], []
    for name in sorted(names - set(known)):
        path = REPO / name
        assert path.is_file() and not path.is_symlink()
        raw = path.read_bytes()
        additional.append(seal(path))
        findings.extend(vars(row) for row in module.detect(name, raw.decode(errors="ignore"), matchers))
        if path.suffix == ".md":
            for target in re.findall(r"\]\(([^)]+)\)", raw.decode()):
                target = target.strip("<>").split("#", 1)[0]
                if not target or re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target):
                    continue
                resolved = path.parent / re.sub(r":\d+$", "", urllib.parse.unquote(target))
                links.append({"source": name, "target": target, "exists": resolved.exists()})
    assert not findings and all(row["exists"] for row in links)
    raw_outputs = {}
    def retain(row):
        assert row == seal(Path(row["path"]))
        raw_outputs[logical(row["path"])] = row
    for path in ROOT.rglob("result.json"):
        value = json.loads(path.read_bytes())
        for field in ["stdout", "stderr", "native_stdout", "native_stderr"]:
            if field in value:
                retain(value[field])
    whitespace = []
    for kind in ["--cached", None]:
        args = ["/usr/bin/git", "diff"] + ([kind] if kind else []) + ["--check", "--", "."]
        args.extend(":(exclude)" + path for path in sorted(raw_outputs))
        run = subprocess.run(args, capture_output=True, text=True, check=False)
        whitespace.append({"command": args, "exit_code": run.returncode, "stdout": run.stdout, "stderr": run.stderr})
        assert run.returncode == 0, whitespace[-1]
    report = {"status": "passed", "driver": seal(Path(__file__).resolve()), "complete_input_review": seal(previous_path),
              "all_previously_reviewed_current_files_unchanged": True, "previous_current_files": len(known),
              "additional_inputs": additional, "additional_findings": findings, "links": links,
              "matcher_count": 127, "scopes": 4, "seals": 39, "exclusions": 51,
              "raw_producer_outputs_preserved_by_exact_receipt": list(raw_outputs.values()), "whitespace_checks": whitespace,
              "scope": "Reuse exact complete review inputs; scan every additional current file under the same rules. Preserve exact producer stdout/stderr, including original final blank lines, and check whitespace on all other staged/unstaged changes.",
              "full_gate": "Deferred to final T02-W04: ./scripts/check-fast.sh"}
    with output_path.open("x") as out:
        json.dump(report, out, indent=2)
        out.write("\n")
    print(json.dumps({"status": "passed", "previous_files_exact": len(known), "additional_inputs": len(additional), "additional_findings": len(findings), "raw_outputs_preserved": len(raw_outputs)}))


if __name__ == "__main__":
    main()
